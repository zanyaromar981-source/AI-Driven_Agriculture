"""The Doctor. Takes the AIs' numbers as one JSON plus a short rulebook, asks the model, returns advice in Sorani and English
with provenance. Provider is swappable: Gemini (default, decided 2026-10-08) or Claude. Plain HTTPS, no SDK.
Keys: GEMINI_API_KEY and/or ANTHROPIC_API_KEY in the environment or in farm_doctor/.env (git-ignored, never printed).
Settings: FARM_DOCTOR_PROVIDER = gemini | claude; FARM_DOCTOR_MODEL = model name (default gemini-2.5-flash / claude-sonnet-5-5)."""
import os, sys, json, base64, mimetypes, urllib.request
HERE = os.path.dirname(os.path.abspath(__file__))
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
Answer as JSON only: {"likely": "...", "confidence": "sure|likely|unsure", "why": ["input -> conclusion", ...], "actions_this_week": ["..."] (max 3),
"cannot_tell": ["..."], "refer_to_officer": true|false, "sorani": "the same advice in Central Kurdish (Sorani), plain words, max 120 words",
"english": "the same advice in plain English, max 120 words"}"""


def _env(name):
    v = os.environ.get(name)
    if not v:
        f = os.path.join(HERE, '.env')
        if os.path.exists(f):
            for line in open(f):
                if line.strip().startswith(name + '='):
                    v = line.strip().split('=', 1)[1].strip().strip('"').strip("'")
    return v


def _provider():
    p = (_env('FARM_DOCTOR_PROVIDER') or 'gemini').lower()
    return p if p in ('gemini', 'claude') else 'gemini'


def _parse(text):
    try:
        return json.loads(text[text.index('{'):text.rindex('}') + 1])
    except Exception:
        return dict(raw=text)


def _user_text(inputs, question):
    return ('Farmer question: ' + (question or 'How is my field and what should I do this week?') +
            '\n\nAI inputs (JSON):\n' + json.dumps(inputs, ensure_ascii=False) + '\n\nReply with the JSON only.')


def _images(photos):
    out = []
    for p in (photos or [])[:6]:
        mt = mimetypes.guess_type(p)[0] or 'image/jpeg'
        out.append((mt, base64.b64encode(open(p, 'rb').read()).decode()))
    return out


def ask_gemini(inputs, question=None, photos=None):
    key = _env('GEMINI_API_KEY')
    if not key:
        return dict(error='no API key: put GEMINI_API_KEY=... in farm_doctor/.env (Google AI Studio)')
    model = _env('FARM_DOCTOR_MODEL') or 'gemini-2.5-flash'
    parts = [{'inline_data': {'mime_type': mt, 'data': b64}} for mt, b64 in _images(photos)]
    parts.append({'text': _user_text(inputs, question)})
    body = {'system_instruction': {'parts': [{'text': RULEBOOK}]},
            'contents': [{'role': 'user', 'parts': parts}],
            'generationConfig': {'responseMimeType': 'application/json', 'maxOutputTokens': 1500, 'temperature': 0.2}}
    url = f'https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent'
    req = urllib.request.Request(url, data=json.dumps(body).encode(),
                                 headers={'x-goog-api-key': key, 'content-type': 'application/json'})
    try:
        r = json.load(urllib.request.urlopen(req, timeout=180))
    except urllib.error.HTTPError as e:
        return dict(error=f'gemini http {e.code}: ' + e.read()[:300].decode('utf-8', 'replace'))
    try:
        text = ''.join(p.get('text', '') for p in r['candidates'][0]['content']['parts'])
    except Exception:
        return dict(error='gemini: no candidates', raw=str(r)[:400])
    j = _parse(text)
    u = r.get('usageMetadata', {})
    j['_usage'] = dict(input_tokens=u.get('promptTokenCount'), output_tokens=u.get('candidatesTokenCount'), model=r.get('modelVersion', model), provider='gemini')
    return j


def ask_claude(inputs, question=None, photos=None):
    key = _env('ANTHROPIC_API_KEY')
    if not key:
        return dict(error='no API key: put ANTHROPIC_API_KEY=... in farm_doctor/.env')
    model = _env('FARM_DOCTOR_MODEL') or 'claude-sonnet-5-5'
    content = [{'type': 'image', 'source': {'type': 'base64', 'media_type': mt, 'data': b64}} for mt, b64 in _images(photos)]
    content.append({'type': 'text', 'text': _user_text(inputs, question)})
    body = dict(model=model, max_tokens=1500, system=RULEBOOK, messages=[{'role': 'user', 'content': content}])
    req = urllib.request.Request('https://api.anthropic.com/v1/messages', data=json.dumps(body).encode(),
                                 headers={'x-api-key': key, 'anthropic-version': '2023-06-01', 'content-type': 'application/json'})
    try:
        r = json.load(urllib.request.urlopen(req, timeout=180))
    except urllib.error.HTTPError as e:
        return dict(error=f'claude http {e.code}: ' + e.read()[:300].decode('utf-8', 'replace'))
    text = ''.join(c.get('text', '') for c in r.get('content', []))
    j = _parse(text)
    u = r.get('usage', {})
    j['_usage'] = dict(input_tokens=u.get('input_tokens'), output_tokens=u.get('output_tokens'), model=r.get('model'), provider='claude')
    return j


def ask(inputs, question=None, photos=None):
    """inputs: dict of the AIs' outputs. photos: list of file paths. Returns the parsed JSON answer from the chosen provider."""
    return ask_gemini(inputs, question, photos) if _provider() == 'gemini' else ask_claude(inputs, question, photos)


if __name__ == '__main__':
    inputs = json.load(open(sys.argv[1]))
    print(json.dumps(ask(inputs, sys.argv[2] if len(sys.argv) > 2 else None), indent=1, ensure_ascii=False))
