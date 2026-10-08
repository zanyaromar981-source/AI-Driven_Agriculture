"""Usage: python3 -I write_results_md.py out_late out_peak zones8.json BACKTEST_RESULTS.md"""
import json, sys
L=json.load(open(sys.argv[1]+'/results.json')); P=json.load(open(sys.argv[2]+'/results.json'))
info=json.load(open(sys.argv[1]+'/zone_info.json')); zones=[z['name'] for z in json.load(open(sys.argv[3]))['zones']]
def pct(x): return '–' if x is None else f'{round(x*100)}%'
out=[]; w=out.append
w('# Backtest results: would the AI have warned before the bad harvests? (2026-10-06)\n')
w(f'Zones: {", ".join(zones)} (8 of 16; cut on user request to shorten the satellite download). Seasons 1999/00–2024/25 (26). Rows per cutoff: {L["Dec"]["n"]} zone-seasons.\n')
w('**Inputs known at the cutoff date:** rain since 1 Oct vs. the zone\'s 1991–2020 normal, soil moisture 0–100 cm (7-day mean) vs. normal, mean temperature anomaly, ET0 vs. normal, last season\'s Oct–May rain vs. normal (all ERA5 via Open-Meteo), and from February the current MODIS greenness vs. the zone median.')
w('**Outcome ("bad" zone-season):** MODIS 250 m NDVI on cropland-like pixels in an 8 km box per zone (pixels whose median spring peak is 0.30–0.75). Main definition = mean NDVI 22 Mar–9 May (grain filling) in the zone\'s bottom 25% of 2000–2025. Second definition = spring peak NDVI (6 Mar–9 May), bottom 25%.')
w('**Model:** logistic regression, features standardised, L2. Leave-one-season-out: all zones of a season are held out together. "Warned" = probability ≥ 0.5 (also shown at 0.35). Baselines: always-normal, same-as-last-year, rain < 75% of normal.\n')
w('## Season level (what the pitch says)\nA season counts as bad when at least half of the zones were bad. Caught = the model warned (≥ half of zones ≥ threshold).\n')
w('| Warning made at | Outcome = late-spring greenness | | Outcome = spring peak | |')
w('|---|---|---|---|---|')
w('| | caught (thr 0.5) | false alarms | caught (thr 0.5) | false alarms |')
for c in ['Dec','Jan','Feb','Mar']:
    a=L[c]; b=P[c]
    w(f'| End of {c} | {a["season_caught"]}/{a["season_bad"]} (thr 0.35: {a["season_caught_lo"]}/{a["season_bad"]}) | {a["season_false"]}/{a["season_normal"]} (0.35: {a["season_false_lo"]}/{a["season_normal"]}) | {b["season_caught"]}/{b["season_bad"]} (0.35: {b["season_caught_lo"]}/{b["season_bad"]}) | {b["season_false"]}/{b["season_normal"]} (0.35: {b["season_false_lo"]}/{b["season_normal"]}) |')
w('\n**Plain-language accuracy (late-spring outcome, threshold 0.5):**\n')
for c in ['Dec','Jan','Feb','Mar']:
    a=L[c]; tot=a['season_bad']+a['season_normal']; right=a['season_caught']+(a['season_normal']-a['season_false'])
    z=a['zone']; bal=((z['recall'] or 0)+(1-(z['far'] or 0)))/2
    w(f'- End of {c}: {right} of {tot} seasons judged right ({round(right/tot*100)}%). Per zone: {pct(z["recall"])} of bad zone-seasons caught, {pct(z["far"])} false alarms, balanced accuracy {round(bal*100)}%.')
w('\n### Season by season (late-spring outcome). B = bad season; number = model\'s mean risk at that date\n')
w('| Season | Bad? | Dec | Jan | Feb | Mar | Zones bad |')
w('|---|---|---|---|---|---|---|')
tabs={c:{t['season']:t for t in L[c]['season_table']} for c in L}
for s in [t['season'] for t in L['Dec']['season_table']]:
    r=[tabs[c].get(s) for c in ['Dec','Jan','Feb','Mar']]
    bad=tabs['Dec'][s]['bad']
    w(f'| {s} | {"**BAD**" if bad else ""} | '+' | '.join('–' if x is None else (f'**{x["mean_p"]:.2f}**' if x['warned'] else f'{x["mean_p"]:.2f}') for x in r)+f' | {", ".join(tabs["Dec"][s]["bad_zones"]) or "–"} |')
w('\n## Zone level (every zone-season separately, late-spring outcome)\n')
w('| Warning made at | Recall (bad zone-seasons caught) | False-alarm rate | Precision | Accuracy | Always-normal acc. | Same-as-last-year recall / FAR | Rain<75% rule recall / FAR |')
w('|---|---|---|---|---|---|---|---|')
for c in ['Dec','Jan','Feb','Mar']:
    a=L[c]; z=a['zone']; bl=a['baseline_last_year']; br=a['baseline_rain_rule']; lo=a['zone_thr035']
    w(f'| End of {c} | {pct(z["recall"])} (thr 0.35: {pct(lo["recall"])}) | {pct(z["far"])} (0.35: {pct(lo["far"])}) | {pct(z["precision"])} | {pct(z["acc"])} | {pct(a["always_normal_acc"])} | {pct(bl["recall"])} / {pct(bl["far"])} | {pct(br["recall"])} / {pct(br["far"])} |')
w('\nModel weights (standardised; negative = more of this lowers the risk):\n')
for c in ['Dec','Jan','Feb','Mar']: w(f'- {c}: '+', '.join(f'{k} {v:+.2f}' for k,v in L[c]['weights'].items()))
w('\n## The 2025 test: trained on 1999/00–2023/24 only, shown data up to each date\n')
w('Risk per zone (* = the zone really had a bad 2025 by the late-spring outcome):\n')
w('| Zone | Rain Oct–Dec | Dec | Jan | Feb | Mar | Bad in 2025? |')
w('|---|---|---|---|---|---|---|')
t={c:{x['zone']:x for x in L[c]['test_2025']} for c in L}
for z in zones:
    r=[t[c].get(z) for c in ['Dec','Jan','Feb','Mar']]
    if not any(r): continue
    lab=next((x['label'] for x in r if x), None)
    w(f'| {z} | {next((str(x["rain_pct"])+"%" for x in r if x),"–")} | '+' | '.join('–' if x is None else (f'**{x["p"]:.2f}**' if x['p']>=0.5 else f'{x["p"]:.2f}') for x in r)+f' | {"**yes**" if lab==1 else ("no" if lab==0 else "?")} |')
w('\n## Sanity check on 2025/26 (the wet year, no label yet): mean risk per date\n')
for c in ['Dec','Jan','Feb','Mar']:
    v=[x['p'] for x in L[c]['test_2026']]
    w(f'- {c}: mean {sum(v)/len(v):.2f}, max {max(v):.2f}' if v else f'- {c}: –')
w('\n## Zone boxes\n')
for z in zones:
    i=info.get(z)
    if i: w(f'- {z}: {i["cropland_px"]} cropland-like pixels of {i["box_px"]}; median late-spring NDVI {i["peak_median"]}, bottom-25% cut {i["peak_q25"]}')
w('\n## Caveats\n- 8 zones, 26 seasons: small. Seasons are the real sample size (26), zones within a season move together.\n- ERA5 is a weather model, not gauges; MODIS 250 m mixes fields, roads and villages. Farmland points were picked by hand near each town.\n- The outcome is greenness, not yield or money. Greenness tracks the crop well in rainfed wheat/barley; it says nothing about irrigated summer crops.\n- Logistic regression at threshold 0.5 is cautious; the risk number itself is what the app shows.\n')
open(sys.argv[4],'w').write('\n'.join(out)); print('written',sys.argv[4])
