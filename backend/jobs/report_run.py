#!/usr/bin/env python3
"""Runs a data job and tells the backend how the run went.

Usage:  report_run.py <job> <command> [arguments...]
Example: report_run.py dryness /usr/bin/python3 region_runner.py

<job> is one of the jobs the backend knows (GET /v1/dashboard/jobs):
dryness, fires, groundwater, dams, briefs, farm_analysis, plans. (farm_plan.py
reports its own `plans` runs and is not started through this wrapper.)

What it does:
  1. Tells the backend the run started (PUT /v1/ingest/jobs/<job>/runs/<start>).
  2. Runs the command, passing its output through unchanged.
  3. Tells the backend the run finished: ok when the command exited 0, the
     number of rows from the job's last "done: N ..." line when there is one,
     and the job's last line as the message.

A backend that cannot be reached never stops the job: the report is skipped
with a line on stderr and the job's own exit code is returned.

Environment:
  FARM_DOCTOR_API       base address, default http://127.0.0.1:3000/v1
  INGEST__SERVICE_KEY   the backend's ingest key (required)
"""
import datetime as dt
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request

API = os.environ.get("FARM_DOCTOR_API", "http://127.0.0.1:3000/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")
MESSAGE_LIMIT = 500


def now():
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def report(job, started, body):
    path = f"{API}/ingest/jobs/{job}/runs/{urllib.parse.quote(started, safe='')}"
    request = urllib.request.Request(
        path,
        data=json.dumps(body).encode(),
        method="PUT",
        headers={"Content-Type": "application/json", "X-Service-Key": KEY},
    )
    try:
        with urllib.request.urlopen(request, timeout=20):
            pass
    except (urllib.error.URLError, OSError) as error:
        print(f"report_run: could not report the {job} run: {error}", file=sys.stderr)


def rows_from(lines):
    """The count after "done:" in the job's summary line, if it wrote one."""
    for line in reversed(lines):
        found = re.search(r"\bdone: (\d+)\b", line)
        if found:
            return int(found.group(1))
    return None


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    if not KEY:
        sys.exit("INGEST__SERVICE_KEY is not set")
    job, command = sys.argv[1], sys.argv[2:]

    started = now()
    report(job, started, {"finished_at": None, "ok": None, "rows": None, "message": None})

    lines = []
    try:
        process = subprocess.Popen(
            command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
        )
        for line in process.stdout:
            sys.stdout.write(line)
            sys.stdout.flush()
            if line.strip():
                lines.append(line.strip())
                del lines[:-50]
        code = process.wait()
    except OSError as error:
        code = 127
        lines.append(f"could not start: {error}")

    message = lines[-1][:MESSAGE_LIMIT] if lines else None
    report(
        job,
        started,
        {"finished_at": now(), "ok": code == 0, "rows": rows_from(lines), "message": message},
    )
    sys.exit(code)


if __name__ == "__main__":
    main()
