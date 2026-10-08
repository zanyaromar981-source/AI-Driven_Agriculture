"""Collect the record of every paper on AI in agriculture from Semantic Scholar (bulk search, keyless), 2000-2026.
Query = an AI term AND a farming term in the title/abstract (see QUERY). Output: metadata/papers.jsonl (one paper per line).
Resumable: keeps the paging token in metadata/_token.txt. Usage: python3 -I harvest_papers.py"""
import os, json, time, urllib.request, urllib.parse, urllib.error
HERE = os.path.dirname(os.path.abspath(__file__)); OUT = os.path.join(HERE, 'metadata')
QUERY = ('("artificial intelligence" | "machine learning" | "deep learning" | "neural network" | "computer vision" | '
         '"large language model" | "random forest") + (agriculture | agricultural | crop | crops | farm | farmer | farmers | '
         'farming | wheat | irrigation | agronomy | livestock)')
FIELDS = 'title,year,publicationDate,abstract,citationCount,influentialCitationCount,openAccessPdf,externalIds,venue,publicationTypes,fieldsOfStudy,authors,url'
API = 'https://api.semanticscholar.org/graph/v1/paper/search/bulk'

def get(params):
    url = API + '?' + urllib.parse.urlencode(params)
    for i in range(30):
        try:
            return json.load(urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'}), timeout=180))
        except urllib.error.HTTPError as e:
            if e.code not in (429, 500, 502, 503, 504): raise
        except Exception:
            pass
        time.sleep(min(60, 4 + 4 * i))
    raise SystemExit('gave up after retries')

def main():
    tokf = os.path.join(OUT, '_token.txt'); outf = os.path.join(OUT, 'papers.jsonl')
    token = open(tokf).read().strip() if os.path.exists(tokf) else None
    n = sum(1 for _ in open(outf)) if os.path.exists(outf) else 0
    if n and not token: print('already complete:', n); return
    while True:
        p = {'query': QUERY, 'fields': FIELDS, 'year': '2000-2026'}
        if token: p['token'] = token
        d = get(p)
        with open(outf, 'a') as f:
            for paper in d.get('data', []):
                a = paper.get('authors') or []
                paper['authors'] = [x.get('name') for x in a[:8]] + (['et al.'] if len(a) > 8 else [])
                f.write(json.dumps(paper, ensure_ascii=False) + '\n'); n += 1
        token = d.get('token')
        print(f"{time.strftime('%H:%M:%S')} {n} / {d.get('total')} papers", flush=True)
        if token: open(tokf, 'w').write(token)
        else:
            if os.path.exists(tokf): os.remove(tokf)
            break
        time.sleep(1.2)
    print('DONE', n)

if __name__ == '__main__':
    main()
