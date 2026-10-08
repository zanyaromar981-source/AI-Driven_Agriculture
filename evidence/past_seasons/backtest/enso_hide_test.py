"""Hide each winter, make the September call from the Pacific state only, then compare with what really happened.
Usage: python3 -I enso_hide_test.py ../backtest_data/enso/oni.ascii.txt ../season_rain_summary.json out_late/results.json"""
import sys, json, statistics as st
ONI={}
for line in open(sys.argv[1]).read().splitlines()[1:]:
    seas,yr,tot,anom=line.split(); ONI[(seas,int(yr))]=float(anom)
rain=json.load(open(sys.argv[2])); R={int(r['season'][:4]):r['full_pct_normal'] for r in rain}
sat={t['season'][:4]:t for t in json.load(open(sys.argv[3]))['Dec']['season_table']}
S=[s for s in range(1991,2026) if s in R and ('JAS',s) in ONI]
allv=sorted(R[s] for s in S); DROUGHT=allv[len(allv)//4]
def phase(v): return 'El Niño' if v>=0.5 else ('La Niña' if v<=-0.5 else 'neutral')
def linfit(xs,ys):
    mx,my=st.mean(xs),st.mean(ys); b=sum((x-mx)*(y-my) for x,y in zip(xs,ys))/sum((x-mx)**2 for x in xs); return my-b*mx,b
print(f'September call, winter hidden, {len(S)} winters. Drought = season rain ≤ {DROUGHT}% of normal.\n')
print(f'{"Winter":9s} {"ONI Sep":>7s} {"State":8s} | {"Call made in September":34s} | {"Predicted rain":>14s} | {"Actual rain":>11s} | Result')
calls=0; right=0; strong_right=0; errs=[]; base=[]
for s in S:
    o=ONI[('JAS',s)]; ph=phase(o)
    others=[t for t in S if t!=s]; a,b=linfit([ONI[('JAS',t)] for t in others],[R[t] for t in others]); pred=a+b*o
    act=R[s]; errs.append(abs(pred-act)); base.append(abs(100-act))
    satmark=''
    if str(s) in sat: satmark=' [satellite: '+('BAD season' if sat[str(s)]['bad'] else 'normal')+']'
    if ph=='El Niño':
        call='Wet or normal winter, no drought'; ok=act>DROUGHT; strong=act>=100
    elif ph=='La Niña':
        call='Dry-leaning winter, below normal'; ok=act<100; strong=act<=DROUGHT
    else:
        call='No call (neutral), wait for December'; ok=None; strong=None
    if ok is not None:
        calls+=1; right+=ok; strong_right+=bool(strong)
    res='–' if ok is None else ('RIGHT' if ok else 'WRONG')
    if ok is not None and ph=='El Niño': res+=' (above normal ✓)' if strong else ' (normal, not above)'
    if ok is not None and ph=='La Niña': res+=' (drought ✓)' if strong else (' (below normal, not drought)' if ok else '')
    print(f'{s}/{str(s+1)[2:]:3s} {o:+7.2f} {ph:8s} | {call:34s} | {pred:13.0f}% | {act:10.0f}% | {res}{satmark}')
print(f'\nCalls made: {calls} of {len(S)} winters (the rest were neutral → no call).')
print(f'Right: {right} of {calls} ({right/calls:.0%}). Strong version right (El Niño→above normal / La Niña→drought): {strong_right} of {calls}.')
print(f'Predicted rain % (from ONI alone, winter hidden): average error {st.mean(errs):.0f} points vs {st.mean(base):.0f} points if you always guess "normal".')
ne=[s for s in S if phase(ONI[('JAS',s)])=='neutral']; nd=sum(1 for s in ne if R[s]<=DROUGHT)
print(f'Neutral winters: {len(ne)}, of which {nd} were droughts → in a neutral year the September call cannot help; the December warning takes over.')
print(f'\n2026/27: ONI JAS 2026 = {ONI[("JAS",2026)]:+.2f} ({phase(ONI[("JAS",2026)])}) → predicted season rain ≈ {linfit([ONI[("JAS",t)] for t in S],[R[t] for t in S])[0]+linfit([ONI[("JAS",t)] for t in S],[R[t] for t in S])[1]*ONI[("JAS",2026)]:.0f}% of normal')
