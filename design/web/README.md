# Jutyar Control Room in Pen

`jutyar_control_room.pen` is the Control Room website (`web/admin_demo`) rebuilt as editable Pen layers, measured from the live page. Open it in Pen; the pictures are in `assets/` next to it.

## What is in it

| Frames | What |
|---|---|
| A0 | Motion spec: the 16 load and interaction details, where each lives in Jutyar, how it moves, and which frames show it |
| A1 to A6 | The opening sequence frozen at key moments: logo drawing, words and counter, 100, curtain lift, bento blinds, cascade with typing and decrypt. Each has a caption |
| B01 to B19 | Every section of the Control Room, full page |
| C1 to C6 | Skeleton screen, title decrypt, blur-up (low and sharp), cursor glow with a magnetic button, scroll progress with the live marquee |
| C7 to C10 | Farm drawer, show a phone with a reason, a section locked for the role, search |

Every box, text, icon and input sits at the position, size, colour, font, corner radius, border and shadow the browser measured. Maps, satellite pictures and the Grain Sun logo are images.

## Rebuild after the website changes

```bash
pip install websocket-client          # once
python build_web_pen.py               # all frames, about 2 minutes, needs Microsoft Edge
python build_web_pen.py B03 C7        # only some frames (the others keep their old pictures)
```

`build_web_pen.py` opens the page in headless Edge, waits for each state (for animations it pauses them at the chosen moment), runs `dom2pen.js` in the page, saves the pictures and writes the `.pen` file. Edit the page in `web/admin_demo`, then rebuild; edits made by hand in Pen are overwritten by a rebuild.

Style: the Jutyar app tokens (`app/lib/theme.dart`, `design/jutyar_app.pen`): Manrope and Noto Sans Arabic, background #F2F4EF, accent #1E7A5A, gold #B98A2B, the Grain Sun logo.
