"""Download NMME (+ C3S ECMWF SEAS5) seasonal precipitation hindcasts/forecasts from the IRI Data Library.

Keyless: IRI requires sign-in for its web pages, but download endpoints
(OPeNDAP .dods, data.tsv, data.nc) still work anonymously (as of 2026-10).

For each model and start month (Sep, Oct, Nov, Dec) one request returns the
ensemble-mean precipitation (mm/day) averaged over the 1-degree grid points
inside the Kurdistan box (lat 34.3-37.3N -> 35,36,37; lon 42.5-46.2E -> 43..46),
leads 0.5..8.5 months, for every year the dataset holds that start.

Raw OPeNDAP binary files are saved as-is (parse with seasonal_models_test.py).

Usage: python3 -I fetch_seasonal_models.py <out_dir> [model ...] [lmax=8.5] [months=9,10,11,12]
  (lmax/months let a second process fetch e.g. Nov/Dec starts with 3 leads only, to save server time)
"""
import os
import sys
import time
import urllib.parse
import urllib.request

ROOT = "https://iridl.ldeo.columbia.edu/SOURCES/"
N = ".Models/.NMME/"
C3S = ".EU/.Copernicus/.CDS/.C3S/"
UA = {"User-Agent": "curl/8.7.1"}  # IRI rejects the default Python-urllib agent

# model -> list of (part, dataset path, first start "YYYY-MM", last start "YYYY-MM")
DATASETS = {
    "CanSIPS-IC4": [("HINDCAST", N + ".CanSIPS-IC4/.HINDCAST/.MONTHLY/.prec", "1990-01", "2024-06"),
                    ("FORECAST", N + ".CanSIPS-IC4/.FORECAST/.MONTHLY/.prec", "2024-07", "2026-10")],
    "NCEP-CFSv2": [("HINDCAST", N + ".NCEP-CFSv2/.HINDCAST/.MONTHLY/.prec", "1982-01", "2010-12"),
                   ("FORECAST", N + ".NCEP-CFSv2/.FORECAST/.EARLY_MONTH_SAMPLES/.MONTHLY/.prec", "2011-03", "2026-10")],
    "GFDL-SPEAR": [("HINDCAST", N + ".GFDL-SPEAR/.HINDCAST/.MONTHLY/.prec", "1991-01", "2020-12"),
                   ("FORECAST", N + ".GFDL-SPEAR/.FORECAST/.MONTHLY/.prec", "2021-01", "2026-10")],
    "NASA-GEOSS2S": [("HINDCAST", N + ".NASA-GEOSS2S/.HINDCAST/.MONTHLY/.prec", "1981-02", "2017-01"),
                     ("FORECAST", N + ".NASA-GEOSS2S/.FORECAST/.MONTHLY/.prec", "2017-02", "2026-10")],
    "COLA-RSMAS-CCSM4": [("ALL", N + ".COLA-RSMAS-CCSM4/.MONTHLY/.prec", "1982-01", "2026-10")],
    "COLA-RSMAS-CESM1": [("ALL", N + ".COLA-RSMAS-CESM1/.MONTHLY/.prec", "1982-01", "2026-10")],
    # Copernicus C3S ECMWF SEAS5 mirrored on IRI (6 leads only; prcp in m/s).
    # SEAS5.1 hindcast 1981-2016, SEAS5 forecasts Sep 2017-Oct 2022, SEAS5.1 forecasts Nov 2022 on.
    "ECMWF-SEAS5": [("HINDCAST", C3S + ".ECMWF/.SEAS51/.hindcast/.prcp", "1981-01", "2016-12"),
                    ("FORECAST5", C3S + ".ECMWF/.SEAS5/.forecast/.prcp", "2017-09", "2022-10"),
                    ("FORECAST51", C3S + ".ECMWF/.SEAS51/.forecast/.prcp", "2022-11", "2026-10")],
}
START_MONTHS = {9: "Sep", 10: "Oct", 11: "Nov", 12: "Dec"}
FIRST_YEAR = 1990  # need starts from Sep 1990 onward (season 1990/91 is outside the ERA5 file; harmless)


def ym(s):
    y, m = s.split("-")
    return int(y), int(m)


def build_url(path, dates, lmax):
    vals = "/".join("(1 %s %d)" % (START_MONTHS[m], y) for y, m in dates)
    ops = ("S/%s/VALUES/L/0.5/%.1f/RANGEEDGES/Y/34.3/37.3/RANGE/X/42.5/46.2/RANGE/"
           "[X Y]average/[M]average/dods.dods" % (vals, lmax))
    return ROOT + path + "/" + urllib.parse.quote(ops, safe="/()")


def fetch(url, dest, tries=3):
    for k in range(tries):
        try:
            t0 = time.time()
            with urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=900) as r:
                b = r.read()
            if b"Data:\n" not in b:
                raise RuntimeError("no Data: section (%d bytes)" % len(b))
            with open(dest, "wb") as f:
                f.write(b)
            if os.path.exists(dest + ".part"):
                os.remove(dest + ".part")
            return len(b), time.time() - t0
        except Exception as e:  # noqa: BLE001
            print("   retry %d: %s" % (k + 1, e), flush=True)
            time.sleep(10)
    raise RuntimeError("failed: " + url)


def main():
    out = sys.argv[1]
    opts = dict(a.split("=", 1) for a in sys.argv[2:] if "=" in a)
    models = [a for a in sys.argv[2:] if "=" not in a] or list(DATASETS)
    lmax_opt = float(opts.get("lmax", 8.5))
    months_opt = [int(x) for x in opts.get("months", "9,10,11,12").split(",")]
    os.makedirs(out, exist_ok=True)
    for model in models:
        for part, path, first, last in DATASETS[model]:
            y0, m0 = ym(first)
            y1, m1 = ym(last)
            lmax = lmax_opt
            for m in months_opt:
                dates = [(y, m) for y in range(max(y0, FIRST_YEAR), y1 + 1)
                         if (y, m) >= (y0, m0) and (y, m) <= (y1, m1)]
                if not dates:
                    continue
                dest = os.path.join(out, "%s_%s_S%s.dods" % (model, part, START_MONTHS[m]))
                if os.path.exists(dest) or os.path.exists(dest + ".part"):
                    print("skip", dest, flush=True)
                    continue
                url = build_url(path, dates, lmax)
                open(dest + ".part", "w").close()  # claim this file so a parallel process skips it
                n, dt = fetch(url, dest)
                print("%s %s %s: %d starts %d-%d, %d bytes, %.0fs" % (
                    model, part, START_MONTHS[m], len(dates), dates[0][0], dates[-1][0], n, dt), flush=True)
                with open(os.path.join(out, "urls.txt"), "a") as f:
                    f.write("%s\t%s\n" % (os.path.basename(dest), url))


if __name__ == "__main__":
    main()
