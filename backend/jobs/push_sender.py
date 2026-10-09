#!/usr/bin/env python3
"""Sends red alerts to farmers' phones through Firebase Cloud Messaging (FCM).

What it does, each run:
  1. Asks the backend which alerts are waiting (GET /v1/ingest/alerts/unpushed):
     only level "alarm", at most one per farm per day, each with the phones
     (device tokens) of the farm's owner. The backend applies those rules.
  2. Sends each alert to each of those phones with FCM's HTTP v1 API.
  3. Tells the backend the alert was pushed (POST /v1/ingest/alerts/{id}/pushed)
     once at least one phone accepted it, so it is never sent twice.

What a phone receives: a notification (title and text in the phone's language)
and the data fields of BACKEND.md 2.7: farm_id, alert_id, type, day, level,
confidence, ku, en, action_ku, action_en.

Signing in to Google: FCM's old "server key" was switched off in 2024. The v1
API needs a Firebase service account: Firebase console, Project settings,
Service accounts, "Generate new private key". Put that JSON file on the server
and point FCM__SERVICE_ACCOUNT_FILE at it. The file is a secret: never commit
it. The job signs a short-lived token with the `openssl` program, so no Python
package is needed.

Without a service account file the job says so and exits 0: nothing is sent
and the alerts keep waiting.

Environment:
  FARM_DOCTOR_API            base address, default http://127.0.0.1:3000/v1
  INGEST__SERVICE_KEY        the backend's ingest key (required)
  FCM__SERVICE_ACCOUNT_FILE  path of the service account JSON
                             (default /opt/farm-doctor/fcm-service-account.json)

Run with --dry-run to print what would be sent without sending.
"""
import base64
import datetime as dt
import json
import os
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

API = os.environ.get("FARM_DOCTOR_API", "http://127.0.0.1:3000/v1").rstrip("/")
KEY = os.environ.get("INGEST__SERVICE_KEY", "")
ACCOUNT_FILE = Path(
    os.environ.get("FCM__SERVICE_ACCOUNT_FILE", "/opt/farm-doctor/fcm-service-account.json")
)
SCOPE = "https://www.googleapis.com/auth/firebase.messaging"
TITLES = {"ku": "ئاگاداری بۆ کێڵگەکەت", "en": "Alert for your farm"}
MAX_BODY = 240
# FCM's answers that mean "this phone no longer has the app": the token is dead.
DEAD_TOKEN = ("UNREGISTERED", "INVALID_ARGUMENT", "NOT_FOUND")


def log(message):
    print(f"{dt.datetime.now(dt.timezone.utc):%Y-%m-%d %H:%M:%S}Z {message}", flush=True)


def backend(method, path, body=None):
    headers = {"accept": "application/json", "x-service-key": KEY}
    data = None
    if body is not None:
        data = json.dumps(body).encode()
        headers["content-type"] = "application/json"
    request = urllib.request.Request(f"{API}{path}", data=data, method=method, headers=headers)
    with urllib.request.urlopen(request, timeout=30) as response:
        raw = response.read()
        return json.loads(raw) if raw else {}


def b64url(raw):
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode()


def signed_assertion(account, now):
    """The RS256-signed request for an access token, signed by `openssl`."""
    header = b64url(json.dumps({"alg": "RS256", "typ": "JWT"}).encode())
    claims = b64url(
        json.dumps(
            {
                "iss": account["client_email"],
                "scope": SCOPE,
                "aud": account["token_uri"],
                "iat": now,
                "exp": now + 3000,
            }
        ).encode()
    )
    signing_input = f"{header}.{claims}".encode()

    # The key goes through a file only this user can read, removed at once:
    # it must not appear in the process list or in a log.
    with tempfile.NamedTemporaryFile("w", suffix=".pem", delete=False) as key_file:
        os.chmod(key_file.name, 0o600)
        key_file.write(account["private_key"])
    try:
        signature = subprocess.run(
            ["openssl", "dgst", "-sha256", "-sign", key_file.name],
            input=signing_input,
            capture_output=True,
            check=True,
        ).stdout
    finally:
        os.unlink(key_file.name)

    return f"{header}.{claims}.{b64url(signature)}"


def access_token(account):
    body = urllib.parse.urlencode(
        {
            "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer",
            "assertion": signed_assertion(account, int(time.time())),
        }
    ).encode()
    request = urllib.request.Request(account["token_uri"], data=body, method="POST")
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.loads(response.read())["access_token"]


def message_for(alert, device):
    """The FCM message for one alert and one phone."""
    lang = "en" if device.get("lang") == "en" else "ku"
    text = alert.get(lang) or alert.get("en") or ""
    action = alert.get(f"action_{lang}") or alert.get("action_en") or ""
    body = f"{text} {action}".strip()[:MAX_BODY]
    data = {
        "farm_id": alert.get("farm_id"),
        "alert_id": alert.get("alert_id"),
        "type": alert.get("type"),
        "day": alert.get("day"),
        "level": alert.get("level"),
        "confidence": alert.get("confidence"),
        "ku": alert.get("ku"),
        "en": alert.get("en"),
        "action_ku": alert.get("action_ku"),
        "action_en": alert.get("action_en"),
    }
    return {
        "message": {
            "token": device["push_token"],
            "notification": {"title": TITLES[lang], "body": body},
            # FCM data values must all be text.
            "data": {key: str(value) for key, value in data.items() if value is not None},
            "android": {"priority": "high"},
        }
    }


def send(project_id, token, message):
    """Returns (accepted, dead_token). Never raises for an answer from FCM."""
    request = urllib.request.Request(
        f"https://fcm.googleapis.com/v1/projects/{project_id}/messages:send",
        data=json.dumps(message).encode(),
        method="POST",
        headers={"authorization": f"Bearer {token}", "content-type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30):
            return True, False
    except urllib.error.HTTPError as error:
        try:
            status = json.loads(error.read()).get("error", {}).get("status", "")
        except ValueError:
            status = ""
        log(f"FCM refused a message: {error.code} {status}")
        return False, status in DEAD_TOKEN
    except (urllib.error.URLError, OSError) as error:
        log(f"FCM could not be reached: {error}")
        return False, False


def devices_of(alert):
    """The phones that want red alerts. A phone that switched them off is skipped."""
    return [
        device
        for device in alert.get("devices", [])
        if device.get("push_token") and (device.get("notify") or {}).get("red_alerts", True)
    ]


def main():
    dry_run = "--dry-run" in sys.argv
    if not KEY:
        sys.exit("INGEST__SERVICE_KEY is not set")

    alerts = backend("GET", "/ingest/alerts/unpushed").get("alerts", [])
    if not alerts:
        log("done: 0 pushed, nothing is waiting")
        return

    if dry_run:
        for alert in alerts:
            for device in devices_of(alert):
                shown = message_for(alert, device)
                shown["message"]["token"] = "(hidden)"
                print(json.dumps(shown, ensure_ascii=False))
        log(f"done: 0 pushed, {len(alerts)} waiting (dry run)")
        return

    if not ACCOUNT_FILE.is_file():
        log(
            f"done: 0 pushed, {len(alerts)} waiting: no Firebase service account at "
            f"{ACCOUNT_FILE} (set FCM__SERVICE_ACCOUNT_FILE)"
        )
        return

    account = json.loads(ACCOUNT_FILE.read_text())
    try:
        token = access_token(account)
    except (urllib.error.URLError, OSError, subprocess.CalledProcessError, KeyError, ValueError) as error:
        log(f"could not sign in to Google with the service account: {error}")
        sys.exit(1)

    pushed, no_phone, failed = 0, 0, 0
    for alert in alerts:
        devices = devices_of(alert)
        if not devices:
            no_phone += 1
            continue

        accepted = 0
        for device in devices:
            ok, dead = send(account["project_id"], token, message_for(alert, device))
            accepted += ok
            if dead:
                try:
                    backend(
                        "DELETE",
                        "/ingest/devices/" + urllib.parse.quote(device["push_token"], safe=""),
                    )
                except (urllib.error.URLError, OSError) as error:
                    log(f"a dead phone token could not be removed: {error}")

        if accepted:
            backend("POST", f"/ingest/alerts/{alert['alert_id']}/pushed")
            pushed += 1
            log(f"alert {alert['alert_id']} ({alert.get('type')}) sent to {accepted} phone(s)")
        else:
            failed += 1

    log(f"done: {pushed} pushed, {no_phone} without a phone, {failed} failed")
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
