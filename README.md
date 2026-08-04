# hhm-sync

opto-sync/syncer.rs JSON reconciliation gateway for Hacker House Medellín.

**Product:** Hacker House Medellín — Operations software for an entrepreneur coliving and coworking community.

Run rooms, desks, member stays, community events, access workflows, and day-to-day operations for a hacker house in Medellín, Colombia.

## Safety and production boundary

The bootstrap does not implement payments, identity verification, door-control hardware, or Colombian lodging compliance. Add those only after security and local regulatory review.

This repository is an executable bootstrap, not a production deployment. Before live
use, add authentication, tenant authorization, rate limits, durable migrations,
observability, backups, incident response, dependency review, and secret management.
## Reconciliation contract

`POST /api/v1/reconcile` accepts `{"base":...,"incoming":...}` and delegates to
`opto-sync/syncer.rs` at immutable commit `132a97c77867128656070be85d3046b0cc065cbf`. The default policy is
identity-keyed array merge using `id` plus last-writer-wins selectors
`updated_at,synced_at`.

The gateway is not a durable record by itself. Persist and authorize the result in
the owning API/database transaction, enforce idempotency, and retain conflict audit
metadata.
