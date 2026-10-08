# Control Room demo (Ministry admin web)

A clickable demo of the web the Ministry would use to run the whole app: farmers and farms, alerts, the inbox, the Doctor, the Alwa market, rules, app settings, notifications, texts, data jobs, the database, security and the history. It is not connected to the backend. All numbers are sample data made in the page; nothing is saved or sent.

## Open it

Double-click `index.html`, or serve the `web/` folder (`python -m http.server 8080` in `web/`, then open `http://localhost:8080/admin_demo/`). It needs internet for the map tiles, the fonts and the icons. The district borders come from `../map_demo/kri_map_data.js`.

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
