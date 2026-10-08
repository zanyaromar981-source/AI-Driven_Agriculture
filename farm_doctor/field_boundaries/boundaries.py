"""Field-boundary test on a Sentinel-2 window: (A) Segment Anything automatic masks on an upsampled false-colour image,
(B) classic NDVI-edge + watershed. Draws both over the true-colour picture and prints counts/sizes.
Usage: venv/bin/python boundaries.py <name> [sam_checkpoint]"""
import sys, json, struct, ast, numpy as np
from PIL import Image, ImageDraw
from scipy import ndimage as ndi
from skimage import filters, segmentation, measure, morphology, exposure, color

name = sys.argv[1]; ck = sys.argv[2] if len(sys.argv) > 2 else 'sam_vit_b_01ec64.pth'
b = open(f'{name}.npy', 'rb').read()
hl = struct.unpack('<H', b[8:10])[0] if b[6] == 1 else struct.unpack('<I', b[8:12])[0]; off = 10 if b[6] == 1 else 12
h = ast.literal_eval(b[off:off + hl].decode('latin1')); arr = np.frombuffer(b[off + hl:], dtype=np.dtype(h['descr'])).reshape(h['shape']).astype(np.float32)
B02, B03, B04, B08, SCL = arr[:5]; meta = json.load(open(f'{name}.json'))
offv = 1000.0 if meta['meta']['baseline'] >= 4.0 and np.median(B04[(SCL == 4) | (SCL == 5)]) > 1000 else 0.0
R, G, Bb, N = [(x - offv).clip(0, 10000) / 10000 for x in (B04, B03, B02, B08)]
ndvi = (N - R) / np.maximum(N + R, 1e-6)
H, W = R.shape; print(name, 'pixels', H, 'x', W, '=', round(H * W * 100 / 1e4), 'ha;', 'median NDVI', round(float(np.median(ndvi)), 2))

def stretch(x): lo, hi = np.percentile(x, (2, 98)); return ((x - lo) / max(hi - lo, 1e-6)).clip(0, 1)
rgb = np.dstack([stretch(R), stretch(G), stretch(Bb)]); fcc = np.dstack([stretch(N), stretch(R), stretch(G)])   # false colour: crops bright red
UP = 4
big = (Image.fromarray((fcc * 255).astype(np.uint8)).resize((W * UP, H * UP), Image.BICUBIC))
Image.fromarray((rgb * 255).astype(np.uint8)).resize((W * UP, H * UP), Image.BICUBIC).save(f'{name}_rgb.png')

# ---- (B) classic: edges of NDVI + watershed
sm = filters.gaussian(ndvi, 1.0); edges = filters.sobel(sm)
markers = measure.label(edges < np.percentile(edges, 55))
lab_ws = segmentation.watershed(edges, markers)
lab_ws = morphology.remove_small_objects(lab_ws, 12)           # drop < 1.2 ha
regs = [r for r in measure.regionprops(lab_ws) if r.area >= 12]
print(f'classic watershed: {len(regs)} segments, median {np.median([r.area for r in regs]) / 100:.1f} ha, largest {max(r.area for r in regs) / 100:.0f} ha')
ov = big.copy(); d = ImageDraw.Draw(ov)
bnd = segmentation.find_boundaries(lab_ws, mode='outer'); ys, xs = np.nonzero(bnd)
for y, x in zip(ys, xs): d.rectangle([x * UP, y * UP, x * UP + UP - 1, y * UP + UP - 1], fill=(255, 255, 0))
ov.save(f'{name}_classic.png')

# ---- (A) SAM automatic masks on the upsampled false-colour image
try:
    import torch
    from segment_anything import sam_model_registry, SamAutomaticMaskGenerator
    dev = 'cpu'   # MPS lacks float64 used inside SAM's mask generator
    sam = sam_model_registry['vit_b'](checkpoint=ck).to(dev)
    gen = SamAutomaticMaskGenerator(sam, points_per_side=48, pred_iou_thresh=0.80, stability_score_thresh=0.85, min_mask_region_area=int(1.0 * 100 * UP * UP))
    img = np.array(big)
    masks = gen.generate(img)
    masks = [m for m in masks if 1.0 * 100 * UP * UP <= m['area'] <= 60 * 100 * UP * UP]   # 1–60 ha
    masks.sort(key=lambda m: -m['area'])
    print(f'SAM ({dev}): {len(masks)} field-like masks, median {np.median([m["area"] for m in masks]) / (100 * UP * UP):.1f} ha, covering {100 * sum(m["area"] for m in masks) / (H * W * UP * UP):.0f}% of the window')
    ov = big.copy(); d = ImageDraw.Draw(ov)
    for m in masks:
        seg = m['segmentation']; bnd = segmentation.find_boundaries(seg, mode='outer'); ys, xs = np.nonzero(bnd)
        for y, x in zip(ys, xs): d.point((x, y), fill=(0, 255, 255))
    ov.save(f'{name}_sam.png')
    # how many SAM masks agree with a watershed segment (IoU > 0.5)?
    agree = 0
    for m in masks:
        seg = np.array(Image.fromarray(m['segmentation'].astype(np.uint8)).resize((W, H), Image.NEAREST)).astype(bool)
        best = 0
        for r in regs:
            rm = lab_ws == r.label; inter = np.logical_and(seg, rm).sum(); uni = np.logical_or(seg, rm).sum()
            best = max(best, inter / max(uni, 1))
        agree += best > 0.5
    print(f'SAM masks matching a classic segment (IoU>0.5): {agree}/{len(masks)}')
except Exception as e:
    print('SAM failed:', str(e)[:200])
