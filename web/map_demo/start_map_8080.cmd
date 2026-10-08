@echo off
rem Serves the map demo at http://localhost:8080 (the address Pen's built-in browser offers).
rem Keep this window open while you use the map. Close it to stop.
echo Map demo: http://localhost:8080
python -m http.server 8080 --bind 127.0.0.1 --directory "%~dp0"
