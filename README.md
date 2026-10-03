# hhm-sync

Offline-first member, application, stay, room, event, and project synchronization built with opto-sync contracts.

Initialized through `DEN-1950` as a testable `sync` foundation. Product behavior continues through focused pull requests.

```bash
python3 scripts/verify_repo.py
```

## Reconciliation boundary

- The merge engine is `opto-sync/syncer.rs` 0.3.1, pinned to immutable commit
  `946e23b0729c92ab3b4cadec8d4fbd540b663dd8`.
- Requests are capped at 64 KiB before JSON extraction.
- Responses identify the exact engine version and the compatibility contract.
- Merge failures return bounded error codes rather than internal parser details.
- Payload-free merge observations are emitted through the canonical
  `ores-otel/ores.otel.log` Rust adapter pinned to immutable commit
  `ca176fb6768a9750d262a536952268625ffd3a8a`.

The compatibility endpoint still accepts generic JSON and is not an authority
for identities, sessions, roles, access policy, or audit state. Do not expose it
to untrusted networks until the API gateway enforces Shared Auth identity and
HHM-owned document authorization. New offline mutation flows should use Opto
Sync causal envelopes, idempotency, checkpoints, and tombstones rather than
adding more timestamp-only merge behavior here.

## Environment secrets

Secrets live in this repo **encrypted** with [sops](https://github.com/getsops/sops) + [age](https://github.com/FiloSottile/age):
`env/enc/<dev|prod>.env.enc` is committed; `just env-use <name>` decrypts it to
`env/dec/<name>.env` (gitignored, mode 0600) and symlinks `./.env` to it. The
Nix dev shell provides the tooling, `just env-audit` runs keyless in CI, and
containers decrypt at `docker run` — never at build. See [`env/README.md`](env/README.md).
