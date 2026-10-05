# AGENTS.md

## Commands

```bash
cargo run                  # :8008
cargo test -q              # scoring unit tests + contract tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cd frontend && pnpm install && pnpm dev && pnpm build
```

## Environment

`APP_PORT` (8008)

## Conventions

- Conventional commits; hygiene hook strips AI attribution
- Scoring stays pure in `src/match_score.rs` — weights documented,
  unit-tested; changing weights changes tests
- ClinicalTrials.gov etiquette: cursor pagination (absence of
  nextPageToken = end), no key, cache at the caller
- Deps ≥7 days old (BEST_PRACTICES/INDEX.md)
