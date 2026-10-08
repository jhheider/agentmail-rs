#!/usr/bin/env python3
"""Compare the crate's bound AgentMail API surface against the live OpenAPI spec.

The baseline is the crate itself: every (method, path) the client modules bind
and every field the type modules model. Spec additions the crate lacks are
drift; so are bound operations the spec removed. Run weekly in CI
(.github/workflows/drift.yml); run locally with:

    python3 scripts/drift_check.py                 # fetch live spec, print report
    python3 scripts/drift_check.py --check         # exit 1 on drift
    python3 scripts/drift_check.py --spec file.json
"""

import argparse
import json
import re
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC_URL = "https://docs.agentmail.to/openapi.json"

SCOPE_PREFIX = {"org": "/v0", "inbox": "/v0/inboxes/{}", "pod": "/v0/pods/{}"}
# Capability trait -> scopes, mirroring src/client/scope.rs (and the private
# traits inside client modules).
TRAIT_SCOPES = {
    "Threads": ["org", "inbox", "pod"],
    "Drafts": ["org", "inbox", "pod"],
    "Webhooks": ["org", "inbox", "pod"],
    "Lists": ["org", "inbox", "pod"],
    "Metrics": ["org", "inbox", "pod"],
    "ApiKeys": ["org", "inbox", "pod"],
    "Domains": ["org", "pod"],
    "Inboxes": ["org", "pod"],
    "AccountScopes": ["inbox", "pod"],
}

# types file -> spec schema family prefixes. Shared-enum and pagination
# schemas that aren't struct fields are filtered below.
FAMILY_PREFIXES = {
    "inboxes.rs": ["inboxes_"],
    "threads.rs": ["threads_"],
    "messages.rs": ["messages_"],
    "attachments.rs": ["attachments_"],
    "drafts.rs": ["drafts_"],
    "webhooks.rs": ["webhooks_"],
    "domains.rs": ["domains_"],
    "pods.rs": ["pods_"],
    "lists.rs": ["lists_"],
    "metrics.rs": ["metrics_"],
    "api_keys.rs": ["apiKeys_"],
    "auth.rs": ["auth_"],
    "organizations.rs": ["organizations_"],
    "agent.rs": ["agent_"],
    "inbox_events.rs": ["inboxEvents_"],
    "accounts.rs": ["accounts_"],
    "apps.rs": ["apps_"],
    "calendar.rs": ["calendar_"],
}

# Spec schemas that are query/header parameters or wire enums, not model
# fields, plus pagination echoes that list structs intentionally omit.
IGNORE_FIELDS = {
    "limit", "page_token", "ascending", "descending", "before", "after",
    "q", "consistency", "mode", "window", "period", "start", "end",
    "include_overlapping", "If-Match", "Idempotency-Key",
    # Bodies the client sends as inline JSON or typed maps rather than
    # structs, so there are no named fields to model.
    "otp_code",
    # Members of the `{start, end?, duration?}` recurrence-date objects,
    # modeled as raw JSON inside `Recurrence::rdates`.
    "duration",
}
# Error envelopes (never modeled field-by-field).
IGNORE_SCHEMAS = re.compile(r"Error")
# ApiKey permission booleans (modeled as ApiKeyPermissions = BTreeMap) and
# raw JWK members (modeled as serde_json::Value).
MAP_MODELED = re.compile(
    r"^(calendar_event|calendar|account|api_key|app|domain|draft|inbox|"
    r"label_[a-z_]+|list_entry|message|metrics|pod|webhook|organization)_"
    r"(read|create|update|delete|send|reply|connect|share_owner)$"
    r"|^(crv|kty|x|y|jwk|fingerprint|public_key)$"
)


def norm_path(p):
    return re.sub(r"\{[a-zA-Z_0-9]+\}", "{}", p)


def crate_surface():
    """(method, normalized path) pairs bound by src/client/*.rs."""
    ops = set()
    client_dir = ROOT / "src" / "client"
    path_re = re.compile(r'"(\{}/[^"]*|/v0[^"]*)"')
    method_re = re.compile(r"Method::(\w+)")

    for file in sorted(client_dir.glob("*.rs")):
        if file.name in ("mod.rs", "scope.rs"):
            continue
        lines = file.read_text().splitlines()
        cur_method = None
        for i, line in enumerate(lines):
            m = method_re.search(line)
            if m:
                cur_method = m.group(1).upper()
                rest = line[m.end():]
            elif cur_method:
                rest = line
            else:
                continue
            hit = path_re.search(rest)
            j = i
            while not hit and cur_method and j < i + 3 and j < len(lines):
                j += 1
                hit = path_re.search(lines[j - 1])
            if not hit:
                continue
            raw = hit.group(1)
            scopes = impl_scopes(lines, i)
            for sc in scopes:
                full = SCOPE_PREFIX[sc] + raw[2:] if raw.startswith("{}/") else raw
                ops.add((cur_method, norm_path(full)))
            cur_method = None
    return ops


def impl_scopes(lines, upto):
    """Scopes the enclosing impl block binds, from its receiver."""
    for line in reversed(lines[:upto]):
        if not line.startswith("impl"):
            continue
        if "Client" in line and "Scoped" not in line:
            return ["org"]
        m = re.search(r"Scoped<'_,\s*([\w:]+)", line)
        if m:
            name = m.group(1).split("::")[-1]
            if name == "InboxScope":
                return ["inbox"]
            if name == "PodScope":
                return ["pod"]
        t = re.search(r"<S:\s*([\w:]+)>", line)
        if t and t.group(1).split("::")[-1] in TRAIT_SCOPES:
            return TRAIT_SCOPES[t.group(1).split("::")[-1]]
    raise RuntimeError("no enclosing impl found")


def spec_ops(spec):
    return {
        (m.upper(), norm_path(p))
        for p, item in spec["paths"].items()
        for m in item
        if m in ("get", "post", "put", "patch", "delete")
    }


def crate_fields():
    """types file -> set of struct field names, including serde rename targets."""
    fields = {}
    for file in sorted((ROOT / "src" / "types").glob("*.rs")):
        if file.name == "mod.rs":
            continue
        names = set()
        src = file.read_text()
        for m in re.finditer(r"pub struct \w+\s*\{(.*?)\n\}", src, re.S):
            body = m.group(1)
            names.update(re.findall(r"pub (\w+):", body))
            # `#[serde(rename = "wire_name")] pub rust_name:` models both.
            for wire, rust in re.findall(
                r'#\[serde\([^)]*rename\s*=\s*"([^"]+)"[^)]*\)\]\s*(?:/\*\*[^*]*\*/\s*)?pub (\w+):',
                body,
            ):
                names.add(wire)
        fields[file.name] = names
    return fields


def schema_props(comps, name, depth=0):
    """Property names of a schema, following one $ref hop."""
    s = comps.get(name, {})
    while "$ref" in s and depth < 4:
        s = comps.get(s["$ref"].split("/")[-1], {})
        depth += 1
    return set(s.get("properties", {}))


def spec_family_fields(comps, prefixes):
    out = set()
    for name in comps:
        # Error envelopes are never modeled field-by-field; search-highlight
        # fragments are modeled as serde_json::Value.
        if IGNORE_SCHEMAS.search(name) or name.endswith("Highlights"):
            continue
        if any(name.startswith(p) for p in prefixes):
            out |= schema_props(comps, name)
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--spec", default=None, help="path to openapi.json (default: fetch live)")
    ap.add_argument("--check", action="store_true", help="exit 1 when drift is found")
    args = ap.parse_args()

    if args.spec:
        spec = json.loads(Path(args.spec).read_text())
    else:
        with urllib.request.urlopen(SPEC_URL, timeout=60) as resp:
            spec = json.loads(resp.read())

    bound = crate_surface()
    upstream = spec_ops(spec)
    missing = sorted(upstream - bound)
    stale = sorted(bound - upstream)

    comps = spec.get("components", {}).get("schemas", {})
    cfields = crate_fields()
    shape = []
    for file, prefixes in sorted(FAMILY_PREFIXES.items()):
        ours = cfields.get(file, set())
        theirs = {
            f
            for f in spec_family_fields(comps, prefixes)
            if f not in IGNORE_FIELDS and not MAP_MODELED.match(f)
        }
        for new in sorted(theirs - ours):
            shape.append(f"  - `{file}`: spec field `{new}` not modeled")

    print(f"# AgentMail drift report\n")
    print(f"Spec: {args.spec or SPEC_URL} (info.version {spec['info'].get('version', '?')})")
    print(f"Bound operations: {len(bound)}   spec operations: {len(upstream)}\n")

    drift = False
    if missing:
        drift = True
        print(f"## In the spec, not bound ({len(missing)})\n")
        for m, p in missing:
            print(f"- `{m} {p}`")
        print()
    if stale:
        drift = True
        print(f"## Bound, but gone from the spec ({len(stale)})\n")
        for m, p in stale:
            print(f"- `{m} {p}`")
        print()
    if shape:
        drift = True
        print(f"## Spec fields not modeled ({len(shape)})\n")
        print("\n".join(shape))
        print()
    if not drift:
        print("No drift: every spec operation is bound, every bound operation exists, all model fields covered.")
    return 1 if (args.check and drift) else 0


if __name__ == "__main__":
    sys.exit(main())
