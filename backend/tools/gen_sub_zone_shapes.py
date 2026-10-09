"""Builds migration/src/sub_zone_shapes.rs from the team's map file.

Usage: python3 -I gen_shapes.py <kri_map_data.js> <create_zones migration> <out.rs>
"""
import json
import re
import sys

src, zones_migration, out = sys.argv[1:4]
text = open(src, encoding="utf-8").read()
data = json.loads(text[text.index("{"): text.rindex("}") + 1])
slug = lambda name: name.lower().replace(" ", "-")

# What the zones migration seeded: (zone slug, sub-zone slug).
seeded = set()
rust = open(zones_migration, encoding="utf-8").read()
for block in re.finditer(r'ZoneSeed \{\s*name: "([^"]+)",.*?sub_zones: &\[(.*?)\],\s*\}', rust, re.S):
    for sub in re.finditer(r'\("([^"]+)",', block.group(2)):
        seeded.add((slug(block.group(1)), slug(sub.group(1))))

rows, seen, points, largest = [], set(), 0, 0
for feature in data["subdistricts"]["features"]:
    p = feature["properties"]
    if p["gov"] == "Context":
        continue
    key = (slug(p["dist"]), slug(p["en"]))
    assert key not in seen, key
    seen.add(key)
    geometry = feature["geometry"]
    polygons = [geometry["coordinates"]] if geometry["type"] == "Polygon" else geometry["coordinates"]
    rings = []
    for polygon in polygons:
        assert len(polygon) == 1, ("hole", key)
        ring = polygon[0]
        if ring[0] == ring[-1]:
            ring = ring[:-1]
        assert len(ring) >= 3
        rings.append([[round(lon, 5), round(lat, 5)] for lon, lat in ring])
    count = sum(len(r) for r in rings)
    points += count
    largest = max(largest, count)
    rows.append((key[0], key[1], json.dumps(rings, separators=(",", ":"))))

print("shapes", len(rows), "seeded", len(seeded), "points", points, "largest", largest)
print("in map, not seeded:", sorted(seen - seeded))
print("seeded, not in map:", sorted(seeded - seen))

rows.sort()
with open(out, "w", encoding="utf-8") as f:
    f.write("// Generated from web/map_demo/kri_map_data.js (Iraq CSO 2019 sub-districts via\n")
    f.write("// geoBoundaries, regrouped into KRG districts). Do not edit by hand.\n")
    f.write("//\n")
    f.write("// Each row is (zone slug, sub-zone slug, outline). The outline is a JSON array\n")
    f.write("// of rings, each ring an array of [lon, lat] in WGS84 degrees with five\n")
    f.write("// decimals, not repeating its first corner at the end. No ring is a hole.\n")
    f.write("#[rustfmt::skip]\n")
    f.write(f"pub const SUB_ZONE_SHAPES: [(&str, &str, &str); {len(rows)}] = [\n")
    for zone, sub, rings in rows:
        f.write(f'    ("{zone}", "{sub}", "{rings}"),\n')
    f.write("];\n")
