#!/usr/bin/env python3
"""Writes API.md, the route-by-route reference, from the server's own
description of itself (`/api-docs/openapi.json`). Run it after any change to
a route, against a running server:

    python3 tools/api_reference.py http://localhost:3000 > API.md

Nothing in API.md is written by hand, so it cannot drift from the code.
"""

import json
import sys
import urllib.request

GROUPS = [
    ("Sign in and profile (farmer app)", lambda p: p.startswith("/v1/auth") or p == "/v1/me"),
    ("Farms (farmer app)", lambda p: p.startswith("/v1/farms")),
    ("Alwa market: farmers and buyers", lambda p: p.startswith("/v1/alwa")),
    ("Region data, no login", lambda p: p.startswith(("/v1/region", "/v1/zones", "/v1/dams", "/v1/outlooks", "/v1/water", "/v1/fires", "/v1/briefs"))),
    ("Dashboard: sign in, staff and roles", lambda p: p.startswith(("/v1/dashboard/auth", "/v1/dashboard/me", "/v1/dashboard/permissions", "/v1/dashboard/roles", "/v1/dashboard/staff"))),
    ("Dashboard: data", lambda p: p.startswith("/v1/dashboard")),
    ("Data jobs (service key)", lambda p: p.startswith("/v1/ingest")),
    ("Server", lambda p: True),
]

# Which resource a dashboard path belongs to, longest prefix first.
RESOURCES = [
    ("/v1/dashboard/farms/{id}/insights", "insights"),
    ("/v1/dashboard/outlook-runs", "outlooks"),
    ("/v1/dashboard/outlooks", "outlooks"),
    ("/v1/dashboard/zones", "zones"),
    ("/v1/dashboard/dams", "dams"),
    ("/v1/dashboard/water", "water"),
    ("/v1/dashboard/fires", "fires"),
    ("/v1/dashboard/alwa", "alwa"),
    ("/v1/dashboard/farmers", "farmers"),
    ("/v1/dashboard/farms", "farms"),
    ("/v1/dashboard/briefs", "briefs"),
    ("/v1/dashboard/roles", "roles"),
    ("/v1/dashboard/staff", "staff"),
]
ACTIONS = {"get": "read", "post": "create", "put": "update", "delete": "delete"}


def access(path, method, operation):
    if path.startswith("/v1/ingest"):
        return "service key"
    if path.startswith("/v1/dashboard"):
        if path == "/v1/dashboard/auth/login":
            return "none"
        if path in ("/v1/dashboard/me", "/v1/dashboard/permissions"):
            return "any staff"
        for prefix, resource in RESOURCES:
            if path.startswith(prefix):
                return f"staff `{resource}:{ACTIONS[method]}`"
        return "staff"
    return "farmer token" if operation.get("security") else "none"


def type_of(schema, schemas, depth=0):
    """A short, readable type for one schema."""
    if not schema:
        return "any"
    if "$ref" in schema:
        name = schema["$ref"].rsplit("/", 1)[-1]
        target = schemas.get(name, {})
        if "enum" in target:
            return " \\| ".join(f"`{value}`" for value in target["enum"])
        if target.get("type") in ("string", "integer", "number", "boolean"):
            return type_of(target, schemas, depth)
        return name
    for key in ("oneOf", "anyOf", "allOf"):
        if key in schema:
            parts = [type_of(part, schemas, depth) for part in schema[key] if part.get("type") != "null"]
            optional = any(part.get("type") == "null" for part in schema[key])
            return " or ".join(dict.fromkeys(parts)) + (" or null" if optional else "")
    kind = schema.get("type")
    if isinstance(kind, list):
        rest = [k for k in kind if k != "null"]
        base = type_of({**schema, "type": rest[0] if rest else "any"}, schemas, depth)
        return base + (" or null" if "null" in kind else "")
    if "enum" in schema:
        return " \\| ".join(f"`{value}`" for value in schema["enum"])
    if kind == "array":
        return "list of " + type_of(schema.get("items", {}), schemas, depth)
    if kind == "string":
        return {"date-time": "timestamp", "date": "day"}.get(schema.get("format"), "text")
    if kind in ("integer", "number"):
        return "number"
    if kind == "boolean":
        return "true/false"
    if kind == "object":
        return "object"
    return "any"


def fields(schema, schemas):
    """`name: type` for each field of an object schema, following one $ref."""
    if not schema:
        return ""
    if "$ref" in schema:
        name = schema["$ref"].rsplit("/", 1)[-1]
        return fields(schemas.get(name, {}), schemas) or name
    if schema.get("type") == "array":
        return "list of " + (type_of(schema.get("items", {}), schemas))
    properties = schema.get("properties")
    if not properties:
        return type_of(schema, schemas)
    required = set(schema.get("required", []))
    parts = []
    for name, sub in properties.items():
        mark = "" if name in required else "?"
        parts.append(f"`{name}{mark}`: {type_of(sub, schemas)}")
    return ", ".join(parts)


def body_of(operation, schemas):
    content = operation.get("requestBody", {}).get("content", {})
    for kind, value in content.items():
        described = fields(value.get("schema", {}), schemas)
        return described if kind == "application/json" else f"({kind}) {described}"
    return ""


def answers(operation, schemas):
    out = []
    for status, answer in sorted(operation.get("responses", {}).items()):
        if not status.startswith("2"):
            continue
        content = answer.get("content", {}).get("application/json", {})
        described = fields(content.get("schema", {}), schemas)
        out.append(f"**{status}** {described}" if described else f"**{status}**")
    return "; ".join(out)


def failures(operation):
    return ", ".join(status for status in sorted(operation.get("responses", {})) if not status.startswith("2"))


def main():
    base = sys.argv[1].rstrip("/") if len(sys.argv) > 1 else "http://localhost:3000"
    with urllib.request.urlopen(f"{base}/api-docs/openapi.json", timeout=120) as response:
        document = json.load(response)
    schemas = document.get("components", {}).get("schemas", {})

    grouped = {title: [] for title, _ in GROUPS}
    for path, methods in sorted(document["paths"].items()):
        title = next(title for title, belongs in GROUPS if belongs(path))
        for method, operation in methods.items():
            if method in ACTIONS:
                grouped[title].append((path, method, operation))

    total = sum(len(rows) for rows in grouped.values())
    print("# API reference")
    print()
    print("Written by `tools/api_reference.py` from the server's own description of itself. Do not edit by hand; run the tool again after a route changes. `FRONTEND.md` at the repo root explains how to use all this; the live, clickable version is at `/api-docs` on any running server.")
    print()
    print(f"{total} operations. A `?` after a field name means it may be left out. Query parameters are listed with the route in `/api-docs`.")
    print()
    print("Access: **none** = no login; **farmer token** = `Authorization: Bearer <token>` from `POST /v1/auth/otp/verify`; **staff** = a token from `POST /v1/dashboard/auth/login` whose roles hold the named permission; **service key** = the `X-Service-Key` header, for our data jobs only.")

    used_shapes = set()
    for title, _ in GROUPS:
        rows = grouped[title]
        if not rows:
            continue
        print()
        print(f"## {title}")
        for path, method, operation in rows:
            print()
            print(f"### `{method.upper()} {path}`")
            print()
            summary = (operation.get("summary") or "").strip()
            if summary:
                print(summary + ("" if summary.endswith(".") else "."))
                print()
            print(f"- Access: {access(path, method, operation)}")
            parameters = [p for p in operation.get("parameters", []) if p.get("in") == "query"]
            if parameters:
                listed = ", ".join(
                    f"`{p['name']}{'' if p.get('required') else '?'}`" for p in parameters
                )
                print(f"- Query: {listed}")
            body = body_of(operation, schemas)
            if body:
                print(f"- Body: {body}")
            answer = answers(operation, schemas)
            if answer:
                print(f"- Answers: {answer}")
            failed = failures(operation)
            if failed:
                print(f"- Can fail with: {failed}")
            text = json.dumps(operation)
            for name in schemas:
                if f"/schemas/{name}\"" in text:
                    used_shapes.add(name)

    # Shapes referred to by name above, spelled out once.
    pending, seen = sorted(used_shapes), set()
    described = []
    while pending:
        name = pending.pop(0)
        if name in seen:
            continue
        seen.add(name)
        schema = schemas.get(name, {})
        if "enum" in schema or schema.get("type") in ("string", "integer", "number", "boolean"):
            continue
        described.append((name, fields(schema, schemas)))
        for other in schemas:
            if f"/schemas/{other}\"" in json.dumps(schema) and other not in seen:
                pending.append(other)
    print()
    print("## Shapes")
    print()
    print("Objects that the routes above refer to by name.")
    print()
    for name, described_fields in sorted(described):
        print(f"- **{name}**: {described_fields}")


if __name__ == "__main__":
    main()
