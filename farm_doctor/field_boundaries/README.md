# Field boundary test (2026-10-08)

Can AI draw farm boundaries from a free Sentinel-2 picture (10 m)? Two methods on three 3.3 × 2.7 km windows.

| Window | Picture | Classic (NDVI edges + watershed) | SAM (Meta Segment Anything, ViT-B, 4× upsampled false colour) |
|---|---|---|---|
| Erbil plain, 17 Apr 2026, 0% cloud, flat regular fields | `erbil_apr2026_rgb.png` | 543 segments, median 0.3 ha: over-split | **249 field masks, median 3.4 ha, outlines follow the real fields** (`erbil_apr2026_sam.png`); 117 agree with a classic segment |
| Koya, 17 Apr 2026, hilly, irregular fields | `koya_apr2026_rgb.png` | 413 segments, over-split | 127 masks, median 2.1 ha, only 48% of the window covered: weaker on hills |
| Koya, 16 Mar 2025, drought, bare | `koya_mar2025_rgb.png` | 509 segments | not run |

Verdict: on the plains (Erbil, Makhmour, Sharazur, Garmiyan) SAM draws field boundaries well enough for "tap your field". On hilly, small, irregular fields it misses about half; there the farmer draws the field by hand. SAM masks overlap (nested masks), so the app must keep one mask per tap (the smallest mask containing the tap point).
Run time: about 2–3 minutes per window on the CPU of this Mac; the SAM checkpoint (375 MB) is in the session scratchpad `fields/sam_vit_b_01ec64.pth` (download again from Meta's release if needed).
Scripts: `fetch_s2.py` (full-res Sentinel-2 window via Planetary Computer, keyless) and `boundaries.py` (both methods, draws overlays). Needs a venv with numpy, scipy, scikit-image, pillow, torch, segment-anything, opencv-python-headless.
