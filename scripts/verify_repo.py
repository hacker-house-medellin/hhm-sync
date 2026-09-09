#!/usr/bin/env python3
import json
import re
from pathlib import Path

root = Path(__file__).resolve().parents[1]
meta = json.loads((root / "project.json").read_text())
required = ["README.md", "AGENTS.md", "project.json", "docs/architecture.md", *meta.get("required_paths", [])]
missing = [path for path in required if not (root / path).exists()]
if missing:
    raise SystemExit(f"missing required paths: {missing}")

token_pattern = re.compile(r"gh[pousr]_[A-Za-z0-9]{20,}|lin_api_[A-Za-z0-9]{20,}")
private_key_block = re.compile(
    r"-----BEGIN (?:[A-Z0-9 ]+ )?PRIVATE KEY-----\s*\r?\n(?:[A-Za-z0-9+/]{40,}={0,2}\s*)+",
    re.MULTILINE,
)

for path in root.rglob("*"):
    if not path.is_file() or ".git" in path.parts or path.stat().st_size > 1_000_000:
        continue
    try:
        text = path.read_text()
    except UnicodeDecodeError:
        continue
    if any(marker in text for marker in ("<" * 7, "=" * 7, ">" * 7)):
        raise SystemExit(f"conflict marker in {path}")
    if token_pattern.search(text) or private_key_block.search(text):
        raise SystemExit(f"credential-shaped content in {path}")

print(f"validated {meta['organization']}/{meta['repository']}")
