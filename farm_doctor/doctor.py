"""The Doctor. Takes the AIs' numbers as one JSON plus a short rulebook, asks Claude, returns advice in Sorani and English
with provenance. No SDK needed: plain HTTPS to the Claude API. Key: ANTHROPIC_API_KEY env var or farm_doctor/.env (never printed)."""
import os, sys, json, urllib.request
HERE = os.path.dirname(os.path.abspath(__file__))
MODEL = os.environ.get('FARM_DOCTOR_MODEL', 'claude-sonnet-5-5')
RULEBOOK = """You are the Farm Doctor for the Kurdistan Region of Iraq. You get measured numbers from five AIs (field eye from Sentinel-2,
weather planner from the 10-day forecast, plant doctor from photos, season check from 25 years of rain, neighbour watch from farmers' reports)
and the dam watch. Rules:
1. Use ONLY the numbers given. Never invent a number, a date or a dose.
2. Never give pesticide or fertilizer doses or product names. For those say: ask the extension officer.
3. Say how sure you are (sure / likely / unsure) and WHY, naming the input behind each conclusion (provenance).
4. If inputs conflict or evidence is thin, say "unsure" and send the farmer to the extension or plant-protection office.
5. Give at most 3 actions for this week, concrete and timed. Say what you cannot tell.
6. Season forecasts: only what season_check says; never promise next months' weather.
Farm rules you may apply (from field trials): sow when 20-25 mm of rain is coming within 3 days (10-20 mm then dry = seed may sprout and die);
spread urea on dry soil just before a rain of 12 mm or more; yellow rust risk = cool (6-16 C) wet nights, check leaves, one spray at first stripes
protects the flag leaf; sunn pest: count nymphs, spray only above 8 per m2; herbicide needs 6 dry hours, 15-24 C, light wind; frost below -2 C or
heat above 31 C at flowering: check heads after 7-10 days, nothing to spray; in a drought year hold back top-dressing (too much nitrogen gave -24% yield).
Answer as JSON: {"likely": "...", "confidence": "sure|likely|unsure", "why": ["input -> conclusion", ...], "actions_this_week": ["..."],
"cannot_tell": ["..."], "refer_to_officer": true|false, "sorani": "the same advice in Central Kurdish (Sorani), plain words, max 120 words",
"english": "the same advice in plain English, max 120 words"}"""

def _key():
    k = os.environ.get('ANTHROPIC_API_KEY')
    if not k:
        f = os.path.join(HERE, '.env')
        if os.path.exists(f):
            for line in open(f):
                if line.strip().startswith('ANTHROPIC_API_KEY='): k = line.strip().split('=', 1)[1].strip().strip('"').strip("'")
    return k

def ask(inputs, question=None, photos=None):
    """inputs: dict of the AIs' outputs. photos: list of file paths (jpeg/png) for the plant-doctor path. Returns the parsed JSON answer."""
    key = _key()
    if not key: return dict(error='no API key: put ANTHROPIC_API_KEY=... in farm_doctor/.env')
    content = []
    for p in (photos or [])[:6]:
        import base64, mimetypes
        mt = mimetypes.guess_type(p)[0] or 'image/jpeg'
        content.append({'type': 'image', 'source': {'type': 'base64', 'media_type': mt, 'data': base64.b64encode(open(p, 'rb').read()).decode()}})
    content.append({'type': 'text', 'text': 'Farmer question: ' + (question or 'How is my field and what should I do this week?') +
                    '\n\nAI inputs (JSON):\n' + json.dumps(inputs, ensure_ascii=False) + '\n\nReply with the JSON only.'})
    body = dict(model=MODEL, max_tokens=1500, system=RULEBOOK, messages=[{'role': 'user', 'content': content}])
    req = urllib.request.Request('https://api.anthropic.com/v1/messages', data=json.dumps(body).encode(),
                                 headers={'x-api-key': key, 'anthropic-version': '2023-06-01', 'content-type': 'application/json'})
    r = json.load(urllib.request.urlopen(req, timeout=180))
    text = ''.join(c.get('text', '') for c in r.get('content', []))
    usage = r.get('usage', {})
    try:
        j = json.loads(text[text.index('{'):text.rindex('}') + 1])
    except Exception:
        j = dict(raw=text)
    j['_usage'] = dict(input_tokens=usage.get('input_tokens'), output_tokens=usage.get('output_tokens'), model=r.get('model'))
    return j

if __name__ == '__main__':
    inputs = json.load(open(sys.argv[1]))
    print(json.dumps(ask(inputs, sys.argv[2] if len(sys.argv) > 2 else None), indent=1, ensure_ascii=False))
