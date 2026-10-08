"""Pitch chart: for each cutoff month, the AI's average bad-season risk per season vs. what really happened.
Usage: python3 -I make_chart.py out_dir/results.json out_dir/backtest_chart.html"""
import json, sys
R=json.load(open(sys.argv[1])); out=sys.argv[2]
W,H,pl,pr,pt,pb=900,170,40,20,26,30
def panel(cut):
    T=R[cut]['season_table']; n=len(T); bw=(W-pl-pr)/n
    Y=lambda v: pt+(H-pt-pb)*(1-v)
    g=''.join(f'<line x1="{pl}" x2="{W-pr}" y1="{Y(v):.1f}" y2="{Y(v):.1f}" stroke="#ddd"/><text x="{pl-6}" y="{Y(v)+4:.1f}" font-size="11" text-anchor="end" fill="#777">{int(v*100)}%</text>' for v in (0.25,0.5,0.75))
    thr=f'<line x1="{pl}" x2="{W-pr}" y1="{Y(0.5):.1f}" y2="{Y(0.5):.1f}" stroke="#555" stroke-dasharray="4 3"/>'
    bars=''
    for i,t in enumerate(T):
        x=pl+i*bw+2; p=t['mean_p']; bad=t['bad']; warned=t['warned']
        col='#d03b3b' if bad else '#1d6c88'
        bars+=f'<rect x="{x:.1f}" y="{Y(p):.1f}" width="{bw-4:.1f}" height="{Y(0)-Y(p):.1f}" fill="{col}" fill-opacity="{1 if bad else .45}" rx="2"><title>{t["season"]}: AI risk {int(p*100)}%, {"BAD season" if bad else "normal"}, {int(t["frac_bad"]*100)}% of zones bad</title></rect>'
        if bad: bars+=f'<text x="{x+(bw-4)/2:.1f}" y="{H-pb+12}" font-size="10" text-anchor="middle" fill="#d03b3b" font-weight="700">{t["season"][2:4]}</text>'
        else: bars+=f'<text x="{x+(bw-4)/2:.1f}" y="{H-pb+12}" font-size="10" text-anchor="middle" fill="#777">{t["season"][2:4]}</text>'
        if t['season'].startswith('2024'): bars+=f'<circle cx="{x+(bw-4)/2:.1f}" cy="{Y(p)-10:.1f}" r="5" fill="none" stroke="#000" stroke-width="1.5"/>'
    s=R[cut]
    title=f'{cut}: caught {s["season_caught"]} of {s["season_bad"]} bad seasons, {s["season_false"]} false alarm(s) in {s["season_normal"]} normal seasons'
    return f'<h3 style="margin:18px 0 4px;font:600 14px system-ui">{title}</h3><svg viewBox="0 0 {W} {H}" style="width:100%;max-width:{W}px;display:block">{g}{thr}{bars}</svg>'
html=f'''<!doctype html><meta charset="utf-8"><title>Backtest: would the AI have warned?</title>
<body style="font-family:system-ui;max-width:940px;margin:24px auto;padding:0 16px;color:#111">
<h2 style="margin:0">Would the AI have warned before the bad harvests? (2000–2025, 16 zones)</h2>
<p style="color:#555;margin:6px 0 0">Bars = the model's average bad-season risk across the 16 zones, computed only from data known at that date (leave-one-season-out). Red = a season where at least half of the zones had a bottom-25% spring greenness (MODIS). Dashed line = warning threshold. Circle = 2024/25.</p>
{''.join(panel(c) for c in ['Dec','Jan','Feb','Mar'])}
<p style="color:#555;font-size:12px">Inputs: ERA5 rain, soil moisture, temperature, ET0 (Open-Meteo), last season's rain, and from February the current MODIS greenness. Outcome: MODIS 250 m spring peak NDVI on cropland-like pixels in an 8 km box per zone.</p></body>'''
open(out,'w').write(html); print('chart written',out)
