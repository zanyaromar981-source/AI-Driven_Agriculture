# Jutyar (farmer app, Flutter)

Built one screen at a time against a fake server. Design source: `../design/jutyar_app.pen`. Data contract: `../BACKEND.md`.

## Run it

```sh
export PATH="$HOME/development/flutter/bin:$PATH"
flutter pub get
flutter run            # pick the phone or emulator
flutter test           # grid math + fake server tests
```

## What works

- Sign in: phone number, SMS code, my farms (Sorani right-to-left, English toggle)
- Add farm: walk the corners with GPS, paint crops on the 10 m satellite grid, name it, save
- Map styles on every map: Satellite with Kurdish place names, Map, Terrain
- Walk mode: walk the edge and the app places the dots (at most 50, bends kept)
- Exact area: farm area is the area inside the border; edge cells are cut along it
- Offline: stays signed in; the border being marked is saved after every dot; Save goes to an outbox on the phone and uploads when there is internet

## Things to know

- `lib/config.dart`: `kTestMode = true` skips input checks and lets you tap the map to place corners. Set it to `false` before release.
- `lib/api/fake_api.dart` is the fake server. The real one will be a new class that implements `lib/api/api.dart`; swap it in `lib/main.dart`.
- Fake server rules: any phone and any code work; a phone ending in 0 starts with no farms; it needs internet like a real server (Wi-Fi off = "offline"), and keeps new farms in a file on the phone.
- Phone storage (`lib/store/`): `session` (sign-in), `draft_edge` (border being marked), `outbox` (farms waiting to upload), `farms_cache` (last list), all JSON in the app's private folder.
- Grid: UTM zone 38N, cell = floor(easting/10), floor(northing/10) (`lib/geo.dart`, checked against pyproj).
- Map styles (`lib/widgets/farm_map.dart`): Satellite = Esri World Imagery + Kurdish place names from OpenStreetMap via the Overpass API (`lib/widgets/place_names.dart`); Map = OpenStreetMap tiles; Terrain = Esri World Topo. All need internet and show their credit line. These are free public services with fair-use rules: for a real release, cache names on our server and get a tile licence.
- Installing on a Samsung phone with `adb install`: add `--user 0`, or the app also lands in the Dual App profile and that copy has no permissions.
