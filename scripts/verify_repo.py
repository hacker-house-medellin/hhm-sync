#!/usr/bin/env python3
import json
import re
from pathlib import Path

TOKEN_PATTERN = re.compile(r"gh[pousr]_[A-Za-z0-9]{20,}|lin_api_[A-Za-z0-9]{20,}")
PRIVATE_KEY_BLOCK = re.compile(
    r"-----BEGIN (?:[A-Z0-9 ]+ )?PRIVATE KEY-----\s*\r?\n(?:[A-Za-z0-9+/]{40,}={0,2}\s*)+",
    re.MULTILINE,
)


def credential_shaped(text: str) -> bool:
    return TOKEN_PATTERN.search(text) is not None or PRIVATE_KEY_BLOCK.search(text) is not None


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    meta = json.loads((root / "project.json").read_text())
    required = [
        "README.md",
        "AGENTS.md",
        "project.json",
        "docs/architecture.md",
        *meta.get("required_paths", []),
    ]
    missing = [path for path in required if not (root / path).exists()]
    if missing:
        raise SystemExit(f"missing required paths: {missing}")

    for path in root.rglob("*"):
        if not path.is_file() or ".git" in path.parts or path.stat().st_size > 1_000_000:
            continue
        try:
            text = path.read_text()
        except UnicodeDecodeError:
            continue
        if any(marker in text for marker in ("<" * 7, "=" * 7, ">" * 7)):
            raise SystemExit(f"conflict marker in {path}")
        if credential_shaped(text):
            raise SystemExit(f"credential-shaped content in {path}")

    print(f"validated {meta['organization']}/{meta['repository']}")


if __name__ == "__main__":
    main()
