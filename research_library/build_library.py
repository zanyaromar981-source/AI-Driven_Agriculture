"""Turn metadata/papers.jsonl into a searchable SQLite library (library.sqlite, full-text search on title+abstract)
and tag each paper with the topics that matter for the SmartSuli project.
Usage: python3 -I build_library.py
Search afterwards, e.g.:  sqlite3 library.sqlite "select year,citations,title from papers where rowid in
   (select rowid from fts where fts match 'wheat AND drought AND satellite') order by citations desc limit 20"
"""
import os, json, re, sqlite3
HERE = os.path.dirname(os.path.abspath(__file__))
TOPICS = {   # tag -> regex on title+abstract (lower case)
 'wheat': r'\bwheat|\bbarley|cereal', 'drought': r'drought|water stress|dry spell|aridity|rainfed|rain-fed|dryland',
 'satellite': r'satellite|sentinel|landsat|modis|remote sensing|ndvi|multispectral|hyperspectral',
 'yield_forecast': r'yield (prediction|forecast|estimat)|crop yield', 'disease_pest': r'disease|pest|rust|blight|fung|insect|weed',
 'irrigation': r'irrigation|evapotranspiration|soil moisture', 'weather_climate': r'weather|climate|rainfall|precipitation|temperature',
 'advisory': r'advisor|extension|decision support|recommendation system|sms|mobile phone|smartphone app|farmer.{0,20}(advice|information)',
 'llm_chatbot': r'large language model|\bllm|chatgpt|gpt-4|chatbot|conversational', 'smallholder': r'smallholder|small-scale|developing countr|africa|india',
 'middle_east': r'\biraq|kurdistan|\biran\b|turkey|türkiye|syria|jordan|morocco|tunisia|egypt|middle east|mena\b|west asia|mediterranean',
 'impact_evidence': r'randomi[sz]ed|field trial|impact evaluation|adoption|farmers\' (income|yield)|cost[- ]benefit',
 'review': r'\breview\b|survey|systematic|meta-analysis|state of the art',
 'livestock': r'livestock|cattle|dairy|poultry|sheep|goat|pig\b', 'robotics_hardware': r'robot|drone|uav|iot|sensor network|tractor'}
RELEVANT = ['wheat', 'drought', 'satellite', 'advisory', 'llm_chatbot', 'smallholder', 'middle_east', 'impact_evidence', 'disease_pest', 'yield_forecast']

def main():
    db = os.path.join(HERE, 'library.sqlite')
    if os.path.exists(db): os.remove(db)
    con = sqlite3.connect(db); c = con.cursor()
    c.execute('create table papers (id text primary key, title text, abstract text, year int, date text, citations int, influential int, venue text, types text, fields text, authors text, doi text, arxiv text, url text, oa_pdf text, tags text, relevance int)')
    c.execute("create virtual table fts using fts5(title, abstract, content='papers', content_rowid='rowid')")
    seen = set(); n = 0
    for line in open(os.path.join(HERE, 'metadata', 'papers.jsonl')):
        p = json.loads(line)
        if p['paperId'] in seen: continue
        seen.add(p['paperId'])
        text = ((p.get('title') or '') + ' ' + (p.get('abstract') or '')).lower()
        tags = [t for t, rx in TOPICS.items() if re.search(rx, text)]
        rel = sum(2 if t in ('wheat', 'drought', 'middle_east', 'advisory', 'impact_evidence') else 1 for t in tags if t in RELEVANT)
        ex = p.get('externalIds') or {}
        c.execute('insert into papers values (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)', (p['paperId'], p.get('title'), p.get('abstract'), p.get('year'), p.get('publicationDate'),
                  p.get('citationCount') or 0, p.get('influentialCitationCount') or 0, p.get('venue'), ','.join(p.get('publicationTypes') or []),
                  ','.join(p.get('fieldsOfStudy') or []), '; '.join(a for a in (p.get('authors') or []) if a), ex.get('DOI'), ex.get('ArXiv'), p.get('url'),
                  (p.get('openAccessPdf') or {}).get('url') or None, ','.join(tags), rel))
        n += 1
    c.execute("insert into fts(rowid, title, abstract) select rowid, title, coalesce(abstract,'') from papers")
    con.commit()
    print(n, 'papers;', c.execute("select count(*) from papers where oa_pdf is not null and oa_pdf != ''").fetchone()[0], 'with a free PDF link;',
          c.execute('select count(*) from papers where abstract is not null').fetchone()[0], 'with abstract')
    for t in TOPICS:
        print(f"  {t:18s} {c.execute('select count(*) from papers where tags like ?', ('%' + t + '%',)).fetchone()[0]}")

if __name__ == '__main__':
    main()
