# Jutyar Control Room (website)

The real website that replaces the `web/admin_demo` demo. Two sides:

- **View** (`#/`): public, no login. Region map by district with a zoom into sub-districts, dams, fires, compare years, Alwa average prices, the news bar.
- **Admin** (`#/admin`): sign in at `#/login`. Overview, farmers and farms (create, edit, delete, support letter, government report), admins, crop register (and crop report), region data, alerts, inbox and news bar, the Doctor (AI), Alwa market, rules, app control, texts and languages, data jobs, settings.

Kurdish (Sorani, right to left) is the default language and English is the second. Every layout works on phones, tablets and desktops.

## Run it

```bash
cd web/control_room
npm install          # once
npm run dev          # http://127.0.0.1:5173
npm run build        # static site in dist/, works from any folder or host
```

Sample sign-in: `karwan.aziz@jutyar.krd` with the password `jutyar2026` (any active admin in the list uses the same password until Supabase is connected).

## Data

There is no backend yet. All records are sample data kept in the browser (`localStorage`), so edits survive a reload in that browser only. **Settings, Reset sample data** brings the starting data back.

- `src/data/types.ts`: every record. The future Supabase tables should follow it one to one.
- `src/data/store.ts`: the local database (a Map per collection: get, put and delete are O(1)).
- `src/data/api.ts`: the reads that combine collections (indexes and totals, built in one pass and cached until data changes) and the writes with their checks. Pages only use this and `db`, so moving to Supabase replaces `store.ts`, `db.ts` and `api.ts`, not the pages.
- `src/auth/auth.tsx`: sign-in. With Supabase it becomes `signInWithPassword`, and forgot-password becomes `resetPasswordForEmail`.
- `public/data/kri_map.json` and `src/data/places.json`: the real KRG borders and names, made from `web/map_demo/kri_map_data.js` by `npm run map`.

## Texts and translation

Every word on the site is in `src/i18n/en/<page>.json`. The Kurdish goes in `src/i18n/ku/<page>.json` with the same keys. A missing Kurdish text shows the English one until it is written.

```bash
npm run texts:export   # writes Desktop/Jutyar_Translation/jutyar_texts.xlsx (keeps Kurdish already written, backs up the old file)
npm run texts:import   # reads the Kurdish column back into src/i18n/ku/
```

Admins can also change any text inside the site (Admin, Texts and languages). Those changes win over the files.

## Speed

- Each page is its own file, loaded the first time it is opened. Leaflet and the 400 kB border file load only when a map is shown, then stay cached.
- Lists filter in one pass after typing stops (200 ms), sort once per change, and draw one page of rows at a time.
- Totals per governorate, district, sub-district and crop are computed in one pass over the farms and reused until a farm changes.
- The news bar and the intro use CSS animation. The cursor's animation loop runs only while the ring is moving.
