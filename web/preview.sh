#!/bin/zsh
# Local preview: wrap index.html in the Artifact skeleton, optionally run a test script after load, screenshot with headless Chrome.
# Usage: ./preview.sh <width> <height> <out.png> <light|dark> [js-to-run-after-load]
cd "$(dirname "$0")"
{ printf '<!doctype html><html><head><meta charset=utf8><meta name=viewport content="width=device-width,initial-scale=1,viewport-fit=cover"><style>:root{color-scheme:light}body{margin:0}</style></head><body>'; cat index.html
  [[ -n "$5" ]] && printf '<script>setTimeout(()=>{%s},300)</script>' "$5"
  printf '</body></html>'; } > _preview.html
SCHEME=1; [[ "$4" == "dark" ]] && SCHEME=0
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --disable-gpu --hide-scrollbars --blink-settings=preferredColorScheme=$SCHEME ${STILL:+--force-prefers-reduced-motion} --virtual-time-budget=6000 --window-size="$1,$2" --screenshot="$3" "file://$PWD/_preview.html" 2>/dev/null
