# Jutyar Control Room demo (Ministry admin web)

A clickable demo of the web the Ministry would use to run the whole app: farmers and farms, alerts, the inbox, the Doctor, the Alwa market, rules, app settings, notifications, texts, data jobs, the database, security and the history. It is not connected to the backend. All numbers are sample data made in the page; nothing is saved or sent.

## Open it

Double-click `index.html`, or serve the `web/` folder (`python -m http.server 8080` in `web/`, then open `http://localhost:8080/admin_demo/`). It needs internet for the map tiles, the fonts and the icons. The district borders come from `../map_demo/kri_map_data.js`.

## Look and motion

Same style as the Jutyar app: Manrope and Noto Sans Arabic, the app colours, and the Grain Sun logo. Loading and interaction details (all in `motion.js` and `motion.css`, switched off for visitors who ask for reduced motion):

| Group | Details |
|---|---|
| Text | letter-by-letter greeting, page titles and numbers decrypt from symbols, eyebrow and subtitle slide up from a mask, Manrope weight grows 200 to 800, live news marquee that speeds up while loading and scrolling |
| Preloader | deep green curtain that lifts, the Grain Sun drawing itself as gold lines, farm words instead of a spinner, a 00 to 100 counter that grows near the end, bento blinds that flip open under the curtain |
| Each page | skeleton blocks shaped like the coming page with a shimmer, satellite pictures and map tiles that load blurred then sharp, cards that cascade in |
| Cursor | a gold cursor with a following ring, cards that glow under it, the satellite banner that turns sharp around it, magnetic main buttons, a green loading line and a gold scroll line at the top |

Test addresses: `?static=1` shows every page finished (no animation), `?pre=40` freezes the preloader at 40%, `?skeleton=1` freezes the skeleton screen. The Pen copy of this site is `design/web/jutyar_control_room.pen`.

## What to try

- Switch officer in the top right corner: Karwan Aziz (admin), Shilan Ahmed (district officer, Sulaymaniyah only), Nazdar Hassan (viewer). Sections a role may not open show a lock.
- Two-officer rule: a change you asked for cannot be approved by you (Approvals). Alerts, rule changes, price publishing, the water plan, account deletes and backup restores all wait for a second officer.
- Protected mode: phones show the last 4 digits, farms show at 1 km. Farms whose farmer sent a report or a Doctor case are open (try farm #1207). "Show phone" asks for a reason and writes it to the History.
- Press `/` to search farms (id or last 4 digits), officers and settings.
- Database: only SELECT questions run; a DELETE is refused.

## Sections

| Group | Sections |
|---|---|
| Home | Overview (what needs you today), Approvals |
| People | Farmers and farms, Officers and roles |
| Fields and region | Crop register, Region data (dryness, dams, fires, season outlook, water plan) |
| Act | Alerts, Inbox, The Doctor (AI), Alwa market |
| Control the app | Rules, App control (versions, feature switches, maintenance, limits, crop list, map sources), Notifications and SMS, Texts and languages |
| System | Data jobs, Database, Security and privacy, History, System settings |

What the backend needs for this is in `BACKEND.md` section 2.11. App changes needed first: test mode off, rules read from the server, the app sends its version, push registration, settings read from the server at start.
