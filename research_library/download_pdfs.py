"""Download the free (open-access) PDFs listed in library.sqlite into pdfs/, most relevant to Jutyar first, then by citations.
Downloaded files are untrusted data: they are only saved (checked to start with %PDF), never opened or run here.
Resumable (skips files already saved; failures logged in pdfs/_failed.tsv). Usage: python3 -I download_pdfs.py [workers] [max]"""
import os, sys, sqlite3, time, urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
HERE = os.path.dirname(os.path.abspath(__file__)); PDF = os.path.join(HERE, 'pdfs'); os.makedirs(PDF, exist_ok=True)
UA = {'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125 Safari/537.36', 'Accept': 'application/pdf,*/*'}

def fetch(pid, url):
    path = os.path.join(PDF, pid + '.pdf')
    if os.path.exists(path): return pid, 'skip', 0
    try:
        data = urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=60).read(60_000_000)
    except Exception as e:
        return pid, 'error ' + str(e)[:80], 0
    if not data.startswith(b'%PDF'): return pid, 'not a pdf', 0
    tmp = path + '.part'; open(tmp, 'wb').write(data); os.replace(tmp, path)
    return pid, 'ok', len(data)

def main():
    workers = int(sys.argv[1]) if len(sys.argv) > 1 else 8; cap = int(sys.argv[2]) if len(sys.argv) > 2 else 10**9
    con = sqlite3.connect(os.path.join(HERE, 'library.sqlite'))
    rows = con.execute("select id, oa_pdf from papers where oa_pdf is not null and oa_pdf != '' order by relevance desc, citations desc").fetchall()[:cap]
    failed = set()
    ff = os.path.join(PDF, '_failed.tsv')
    if os.path.exists(ff): failed = {l.split('\t')[0] for l in open(ff)}
    todo = [(i, u) for i, u in rows if i not in failed and not os.path.exists(os.path.join(PDF, i + '.pdf'))]
    print(f'{len(rows)} free PDFs listed, {len(todo)} to fetch', flush=True)
    ok = bad = size = 0; t0 = time.time()
    with ThreadPoolExecutor(workers) as ex, open(ff, 'a') as flog:
        futs = [ex.submit(fetch, i, u) for i, u in todo]
        for k, f in enumerate(as_completed(futs), 1):
            pid, st, n = f.result()
            if st == 'ok': ok += 1; size += n
            elif st != 'skip': bad += 1; flog.write(f'{pid}\t{st}\n'); flog.flush()
            if k % 200 == 0 or k == len(todo):
                print(f"{time.strftime('%H:%M:%S')} {k}/{len(todo)} tried, {ok} saved ({size/1e9:.1f} GB), {bad} failed, {(time.time()-t0)/60:.0f} min", flush=True)
    print('DONE', ok, 'saved,', bad, 'failed')

if __name__ == '__main__':
    main()
