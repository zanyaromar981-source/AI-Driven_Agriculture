"""Do seasonal forecast models (NMME + ECMWF SEAS5) add skill over ONI for Kurdistan winter rain?

Inputs (all passed as arguments, nothing is read from the script folder):
  models_dir   raw OPeNDAP .dods files from fetch_seasonal_models.py (box-mean, ensemble-mean precip)
  openmeteo_dir ERA5 daily precipitation per zone (<Zone>.json, Open-Meteo archive format)
  zones_json   zone list (key "zones", each with "name")
  oni_txt      NOAA CPC oni.ascii.txt

Usage:
  python3 -I seasonal_models_test.py <models_dir> <openmeteo_dir> <zones_json> <oni_txt>

Method
  * Truth = mean over the 16 zones of ERA5 monthly precipitation totals.
  * Model value = ensemble-mean precip over the 1-degree grid points inside the box
    (35-37N, 43-46E), converted to monthly totals (rate x days) and summed over the target months.
    Anomaly = minus the model's own mean for that start/lead over the test years (bias removal).
  * Start months Sep, Oct, Nov, Dec; target = the 3 months from the start month (leads 0.5-2.5).
    Sep and Oct starts are also tested against the full Oct-May season (model sum over the
    Oct-May months it covers: ECMWF only has 6 leads, so Oct-Feb / Oct-Mar).
  * ONI baseline = the latest 3-month ONI known at issue time (Sep start -> JJA, Oct -> JAS,
    Nov -> ASO, Dec -> SON of the same year).
  * Seasons 1991/92 .. 2025/26 (n = 35 where the target is complete).
  * Leave-one-year-out (LOO) linear regression: ONI alone, model alone, ONI + model.
    Reported as cross-validated correlation (r_cv) and MSSS (1 - MSE / MSE of LOO climatology).
  * Dry seasons = bottom 25% of observed target totals. "Called below normal" = forecast in the
    lowest third of that predictor's own values over the test years.
"""
import json
import math
import os
import re
import struct
import sys

MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
START_MONTHS = [9, 10, 11, 12]
ONI_AT_ISSUE = {9: "JJA", 10: "JAS", 11: "ASO", 12: "SON"}
YEARS = list(range(1991, 2026))  # season years (start year of Oct-May season)
NMME = ["CanSIPS-IC4", "NCEP-CFSv2", "GFDL-SPEAR", "NASA-GEOSS2S", "COLA-RSMAS-CCSM4", "COLA-RSMAS-CESM1"]
ALL_MODELS = NMME + ["ECMWF-SEAS5"]


# ----------------------------------------------------------------------------- readers
DECL = re.compile(r"(Float32|Float64|Int32)\s+(\w+)((?:\[[^\]]*\])+);")


def read_dods(path):
    """Minimal OPeNDAP binary (.dods) reader: returns {name: (dims, values)}."""
    b = open(path, "rb").read()
    k = b.index(b"\nData:\n")
    dds = b[:k].decode()
    pos = k + 7
    out = {}
    for typ, name, dimstr in DECL.findall(dds):
        dims = [(d, int(n)) for d, n in re.findall(r"\[(\w+) = (\d+)\]", dimstr)]
        n1, n2 = struct.unpack(">II", b[pos:pos + 8])
        pos += 8
        code, size = {"Float32": ("f", 4), "Float64": ("d", 8), "Int32": ("i", 4)}[typ]
        vals = struct.unpack(">%d%s" % (n1, code), b[pos:pos + n1 * size])
        pos += n1 * size
        if name not in out:
            out[name] = (dims, vals)
    return out


def ym_from_s(s):
    m = int(round(s))
    return 1960 + m // 12, m % 12 + 1


def days_in(y, m):
    if m == 2:
        return 29 if (y % 4 == 0 and (y % 100 != 0 or y % 400 == 0)) else 28
    return 30 if m in (4, 6, 9, 11) else 31


def add_months(y, m, k):
    t = y * 12 + (m - 1) + k
    return t // 12, t % 12 + 1


def load_models(models_dir):
    """fc[model][(year, start_month)] = {lead_index: monthly total mm}."""
    fc = {}
    for fn in sorted(os.listdir(models_dir)):
        if not fn.endswith(".dods"):
            continue
        model = fn.split("_")[0]
        d = read_dods(os.path.join(models_dir, fn))
        var = "prec" if "prec" in d else "prcp"
        dims, vals = d[var]
        scale = 86400000.0 if var == "prcp" else 1.0  # ECMWF m/s -> mm/day
        names = [n for n, _ in dims]
        shape = dict(dims)
        S = d["S"][1]
        L = d["L"][1]
        for i, s in enumerate(S):
            y, m = ym_from_s(s)
            for j, lead in enumerate(L):
                idx = (i * shape["L"] + j) if names == ["S", "L"] else (j * shape["S"] + i)
                v = vals[idx]
                if v != v:  # NaN
                    continue
                li = int(round(lead - 0.5))
                ty, tm = add_months(y, m, li)
                fc.setdefault(model, {}).setdefault((y, m), {})[li] = v * scale * days_in(ty, tm)
    return fc


def load_era5(openmeteo_dir, zones_json):
    zones = [z["name"] for z in json.load(open(zones_json))["zones"]]
    per_zone = []
    for z in zones:
        d = json.load(open(os.path.join(openmeteo_dir, z + ".json")))["daily"]
        mon, cnt = {}, {}
        for t, p in zip(d["time"], d["precipitation_sum"]):
            key = (int(t[:4]), int(t[5:7]))
            mon[key] = mon.get(key, 0.0) + (p or 0.0)
            cnt[key] = cnt.get(key, 0) + 1
        per_zone.append({k: v for k, v in mon.items() if cnt[k] == days_in(*k)})  # complete months only
    keys = set.intersection(*[set(z) for z in per_zone])
    return {k: sum(z[k] for z in per_zone) / len(per_zone) for k in keys}, len(zones)


def load_oni(path):
    oni = {}
    for line in open(path):
        p = line.split()
        if len(p) == 4 and p[1].isdigit():
            oni[(p[0], int(p[1]))] = float(p[3])
    return oni


# ----------------------------------------------------------------------------- stats
def mean(a):
    return sum(a) / len(a)


def corr(a, b):
    ma, mb = mean(a), mean(b)
    sab = sum((x - ma) * (y - mb) for x, y in zip(a, b))
    saa = sum((x - ma) ** 2 for x in a)
    sbb = sum((y - mb) ** 2 for y in b)
    return sab / math.sqrt(saa * sbb) if saa > 0 and sbb > 0 else float("nan")


def partial_corr(y, x, z):
    """corr(y, x | z)."""
    ryx, ryz, rxz = corr(y, x), corr(y, z), corr(x, z)
    return (ryx - ryz * rxz) / math.sqrt((1 - ryz ** 2) * (1 - rxz ** 2))


def ols_fit(X, y):
    """X: list of rows (without intercept). Returns coefficients [b0, b1, ...]."""
    rows = [[1.0] + list(r) for r in X]
    p = len(rows[0])
    A = [[sum(r[i] * r[j] for r in rows) for j in range(p)] for i in range(p)]
    v = [sum(r[i] * t for r, t in zip(rows, y)) for i in range(p)]
    for c in range(p):  # Gauss-Jordan
        piv = max(range(c, p), key=lambda r: abs(A[r][c]))
        A[c], A[piv], v[c], v[piv] = A[piv], A[c], v[piv], v[c]
        for r in range(p):
            if r != c:
                f = A[r][c] / A[c][c]
                A[r] = [a - f * b for a, b in zip(A[r], A[c])]
                v[r] -= f * v[c]
    return [v[i] / A[i][i] for i in range(p)]


def predict(beta, row):
    return beta[0] + sum(b * x for b, x in zip(beta[1:], row))


def loo(X, y):
    """Leave-one-out predictions -> (r_cv, MSSS vs LOO climatology)."""
    n = len(y)
    pred, clim = [], []
    for i in range(n):
        Xt = [X[k] for k in range(n) if k != i]
        yt = [y[k] for k in range(n) if k != i]
        pred.append(predict(ols_fit(Xt, yt), X[i]))
        clim.append(mean(yt))
    mse = mean([(p - t) ** 2 for p, t in zip(pred, y)])
    msec = mean([(c - t) ** 2 for c, t in zip(clim, y)])
    return corr(pred, y), 1 - mse / msec


def lowest_third(vals):
    s = sorted(vals)
    thr = s[len(s) // 3 - 1] if len(s) >= 3 else s[0]
    return [v <= thr for v in vals]


def zscores(vals):
    m = mean(vals)
    sd = math.sqrt(mean([(v - m) ** 2 for v in vals]))
    return [(v - m) / sd for v in vals]


def zscores_opt(vals):
    have = [v for v in vals if v is not None]
    m = mean(have)
    sd = math.sqrt(mean([(v - m) ** 2 for v in have]))
    return [None if v is None else (v - m) / sd for v in vals]


def rcrit(n):  # two-sided 5% threshold for a correlation (Fisher z)
    return math.tanh(1.96 / math.sqrt(n - 3))


# ----------------------------------------------------------------------------- targets
def target_months(start_m, kind):
    if kind == "3mo":
        return [0, 1, 2]
    # Oct-May lead indices for this start month
    first = (10 - start_m) % 12
    return list(range(first, first + 8))


def target_label(start_m, kind, y):
    if kind == "3mo":
        a = MONTHS[start_m - 1]
        b = MONTHS[(start_m + 1) % 12]
        return "%s-%s" % (a, b)
    return "Oct-May"


def obs_total(era, y, start_m, leads):
    tot = 0.0
    for li in leads:
        key = add_months(y, start_m, li)
        if key not in era:
            return None
        tot += era[key]
    return tot


def model_total(f, leads):
    """Sum over the requested leads the model has; None if it has none of the first 3."""
    have = [li for li in leads if li in f]
    if not have or len(have) < min(3, len(leads)):
        return None, 0
    return sum(f[li] for li in have), len(have)


# ----------------------------------------------------------------------------- main
def main():
    models_dir, om_dir, zones_json, oni_txt = sys.argv[1:5]
    fc = load_models(models_dir)
    era, nz = load_era5(om_dir, zones_json)
    oni = load_oni(oni_txt)
    print("ERA5 region = mean of %d zones; complete months %d-%02d .. %d-%02d" % (
        nz, *min(era), *max(era)))
    print("Models loaded:", ", ".join("%s (%d starts)" % (m, len(fc[m])) for m in ALL_MODELS if m in fc))
    missing = [m for m in ALL_MODELS if m not in fc]
    if missing:
        print("MISSING models:", missing)
    models = [m for m in ALL_MODELS if m in fc]

    # Sanity: hindcast-era vs forecast-era mean of the Oct-start Oct-Dec forecast (mm)
    print("\nBias check, Oct-start Oct-Dec forecast mean (mm): 1991-2010 vs 2011-2025, obs %.0f vs %.0f" % (
        mean([obs_total(era, y, 10, [0, 1, 2]) for y in range(1991, 2011)]),
        mean([obs_total(era, y, 10, [0, 1, 2]) for y in range(2011, 2026)])))
    for m in models:
        a = [model_total(fc[m].get((y, 10), {}), [0, 1, 2])[0] for y in range(1991, 2011)]
        b = [model_total(fc[m].get((y, 10), {}), [0, 1, 2])[0] for y in range(2011, 2026)]
        if None in a or None in b:
            print("  %-17s incomplete" % m)
            continue
        print("  %-17s %6.0f  %6.0f" % (m, mean(a), mean(b)))

    results = []
    dry_rows = []
    preds_cache = {}
    for kind in ["3mo", "OctMay"]:
        for sm in START_MONTHS:
            if kind == "OctMay" and sm not in (9, 10):
                continue
            leads = target_months(sm, kind)
            label = target_label(sm, kind, 0)
            # observed target + ONI per year
            ys, obs, onis = [], [], []
            for y in YEARS:
                o = obs_total(era, y, sm, leads)
                k = (ONI_AT_ISSUE[sm], y)
                if o is None or k not in oni:
                    continue
                ys.append(y)
                obs.append(o)
                onis.append(oni[k])
            series = {}
            for m in models:
                vals = [model_total(fc[m].get((y, sm), {}), leads)[0] for y in ys]
                have = [v for v in vals if v is not None]
                if len(have) < 0.8 * len(ys):
                    continue
                mu = mean(have)
                # anomaly vs own mean (bias removed); None where the archive has no forecast
                series[m] = [None if v is None else v - mu for v in vals]
            # multi-model means of standardised anomalies (mean of the members present that year)
            for tag, members in (("NMME-mean", [m for m in NMME if m in series]),
                                 ("ALL-mean", [m for m in models if m in series])):
                if len(members) < 2 or (tag == "ALL-mean" and "ECMWF-SEAS5" not in members):
                    continue
                zs = [zscores_opt(series[m]) for m in members]
                series["%s(%d)" % (tag, len(members))] = [
                    mean([z[i] for z in zs if z[i] is not None]) for i in range(len(ys))]
            preds_cache[(kind, sm)] = (ys, obs, onis, series)

            n = len(ys)
            r_oni = corr(onis, obs)
            cv_oni = loo([[o] for o in onis], obs)
            # dry seasons
            k25 = int(math.ceil(0.25 * n))
            dry_idx = sorted(range(n), key=lambda i: obs[i])[:k25]
            low_oni = lowest_third(onis)
            dry_oni = sum(low_oni[i] for i in dry_idx)
            print("\n=== Start %s, target %s (obs), n=%d seasons %d/%02d-%d/%02d, |r|>=%.2f is 5%%-significant" % (
                MONTHS[sm - 1], label, n, ys[0], (ys[0] + 1) % 100, ys[-1], (ys[-1] + 1) % 100, rcrit(n)))
            print("  ONI(%s) alone: r=%+.2f  LOO r_cv=%+.2f MSSS=%+.2f  dry seasons called below-normal %d/%d" % (
                ONI_AT_ISSUE[sm], r_oni, cv_oni[0], cv_oni[1], dry_oni, k25))
            print("  %-17s %2s %6s %6s %8s | %-15s | %-15s | %-15s | %-15s | %s" % (
                "model", "n", "r", "r(ONI)", "partial", "ONI LOO cv/MSSS", "model LOO", "ONI+model LOO",
                "gain vs ONI", "dry hits"))
            for m, s in series.items():
                idx = [i for i in range(n) if s[i] is not None]
                so, oo, no = [s[i] for i in idx], [obs[i] for i in idx], [onis[i] for i in idx]
                cv_oni_s = loo([[o] for o in no], oo)  # ONI alone on exactly the same years
                r = corr(so, oo)
                cv_m = loo([[v] for v in so], oo)
                cv_both = loo([[o, v] for o, v in zip(no, so)], oo)
                pc = partial_corr(oo, so, no)
                low = dict(zip(idx, lowest_third(so)))
                dry_here = [i for i in dry_idx if i in low]
                hits = sum(low[i] for i in dry_here)
                calls = sum(low.values())
                print("  %-17s %2d %+6.2f %+6.2f %+8.2f | %+5.2f / %+5.2f   | %+5.2f / %+5.2f   | %+5.2f / %+5.2f   | r %+5.2f MSSS %+5.2f | %d/%d (calls %d)" % (
                    m, len(idx), r, corr(so, no), pc, cv_oni_s[0], cv_oni_s[1], cv_m[0], cv_m[1],
                    cv_both[0], cv_both[1], cv_both[0] - cv_oni_s[0], cv_both[1] - cv_oni_s[1],
                    hits, len(dry_here), calls))
                results.append(dict(kind=kind, start=MONTHS[sm - 1], target=label, model=m, n=len(idx), r=r,
                                    partial=pc, cv_model=cv_m, cv_oni=cv_oni_s, cv_both=cv_both,
                                    dry_hits=hits, dry_n=len(dry_here)))
            dry_rows.append((MONTHS[sm - 1], label, [(ys[i], round(obs[i])) for i in dry_idx]))

    print("\nDriest-25% seasons per target (season start year, obs mm):")
    for sm, label, rows in dry_rows:
        print("  start %s %-8s %s" % (sm, label, rows))

    # ------------------------------------------------------------------ 2026/27 forecasts
    print("\n=== Current forecasts for 2026/27 (box mean, ensemble mean) ===")
    print("  pct = forecast as % of the model's own 1991-2020 mean for the same start/lead;")
    print("  rank = share of the model's 1991-2025 forecasts below the 2026 value (0 = driest ever, 100 = wettest).")
    for sm in (9, 10):
        for kind in ("3mo", "OctMay"):
            leads = [1, 2, 3] if (kind == "3mo" and sm == 9) else target_months(sm, kind)
            label = "Oct-Dec" if (kind == "3mo" and sm == 9) else target_label(sm, kind, 0)
            print("  -- start %s 2026, target %s" % (MONTHS[sm - 1], label))
            zs_now = []
            for m in models:
                cur, nl = model_total(fc[m].get((2026, sm), {}), leads)
                if cur is None:
                    print("     %-17s no forecast available" % m)
                    continue
                hist_y = [(y, model_total(fc[m].get((y, sm), {}), leads)[0]) for y in range(1991, 2026)]
                hist = [h for _, h in hist_y if h is not None]
                clim = mean([h for y, h in hist_y if h is not None and y <= 2020])
                rank = 100.0 * sum(h < cur for h in hist) / len(hist)
                sd = math.sqrt(mean([(h - mean(hist)) ** 2 for h in hist]))
                z = (cur - mean(hist)) / sd
                if m in NMME:
                    zs_now.append(z)
                tag = " (months covered: %d of %d)" % (nl, len(leads)) if nl < len(leads) else ""
                print("     %-17s %5.0f mm  = %4.0f%% of own normal, rank %3.0f%%, z %+.2f%s" % (
                    m, cur, 100 * cur / clim, rank, z, tag))
            if zs_now:
                print("     NMME mean z = %+.2f (n=%d models)" % (mean(zs_now), len(zs_now)))

    # Regression outlook for 2026/27 (fit on all years, Oct start; ONI JAS 2026)
    print("\n=== Regression outlook for 2026/27 from Oct-2026 start (fit on 1991-2025) ===")
    oni_now = oni.get(("JAS", 2026))
    for kind in ("3mo", "OctMay"):
        ys, obs, onis, series = preds_cache[(kind, 10)]
        leads = target_months(10, kind)
        norm = mean(obs[:30])  # 1991-2020 observed normal
        b = ols_fit([[o] for o in onis], obs)
        p_oni = predict(b, [oni_now])
        line = "  %s: obs normal(1991-2020) %.0f mm; ONI-only (JAS 2026 = %+.2f) -> %.0f mm (%.0f%%)" % (
            target_label(10, kind, 0), norm, oni_now, p_oni, 100 * p_oni / norm)
        key = [k for k in series if k.startswith("NMME-mean")]
        if key:
            nm = [m for m in NMME if m in series]
            # current NMME-mean z for this target
            zc = []
            for m in nm:
                hist = [model_total(fc[m].get((y, 10), {}), leads)[0] for y in ys]
                hist = [h for h in hist if h is not None]
                cur = model_total(fc[m].get((2026, 10), {}), leads)[0]
                if cur is None:
                    continue
                mu = mean(hist)
                sd = math.sqrt(mean([(h - mu) ** 2 for h in hist]))
                zc.append((cur - mu) / sd)
            if zc:
                b2 = ols_fit([[o, v] for o, v in zip(onis, series[key[0]])], obs)
                p2 = predict(b2, [oni_now, mean(zc)])
                line += "; ONI+NMME-mean (z %+.2f, %d models) -> %.0f mm (%.0f%%)" % (
                    mean(zc), len(zc), p2, 100 * p2 / norm)
        print(line)
        print("     note: JAS 2026 ONI is above every JAS value in the fit (max %+.2f) -> extrapolation" % max(onis))


if __name__ == "__main__":
    main()
