# Development Guide

Machine-facing commands live in [AGENTS.md](../AGENTS.md).

## Prerequisites

Rust 1.96, cargo · pnpm 11 / Node 24 · network access to
clinicaltrials.gov when actually matching (tests don't need it).

## Daily loop

```bash
cargo run                  # :8008
cd frontend && pnpm dev     # :5173 → /api proxied
cargo test -q              # 6 tests — scoring + contracts
cargo clippy --all-targets -- -D warnings
```

## Testing notes

- Scorer tests pin the policy: exact condition match outranks partial
  (Type 2 > Type 1 Diabetes > asthma), phase and sites contribute, and
  scores cap at 100
- Contract tests cover validation (empty/whitespace conditions → 400)
  without network

## Changing the weights

Scoring is policy: update `score_trial` in `src/match_score.rs`, adjust
the table in [ARCHITECTURE.md](ARCHITECTURE.md), and update the unit
tests in the same commit — never separately.

## Gotchas learned here

- **Tokenizers and numbers**: keep bare digits or *Type 2* becomes
  indistinguishable from *Type 1* Diabetes — the test exists because
  this bit us
- **v2 pagination**: absence of `nextPageToken` (not an empty array) is
  the end signal
- Port 8008

## Conventions

Conventional commits; hygiene hook strips AI attribution. The
ClinicalTrials.gov client stays in `src/api.rs` with cursor-pagination
fields and caller-side caching; the scorer stays pure.
