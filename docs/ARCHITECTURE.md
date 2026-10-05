# Architecture

## Modules

```
src/
├── api.rs          # ClinicalTrials.gov v2 client
├── match_score.rs  # pure scorer: rank + score_trial (unit-tested)
├── routes.rs       # match pipeline (per-condition search, merge, rank)
└── state.rs        # shared HTTP client
```

## The v2 client (src/api.rs)

`search_recruiting_page` maps to the documented v2 surface:
`query.cond` for condition scoping, `filter.overallStatus=RECRUITING`,
`pageSize`, and `pageToken` cursor pagination — **absence of
`nextPageToken` is the end signal** (not an empty array), which the
client models as `TrialPage { trials, next_page_token }`. Etiquette: no
API key, cached queries at the caller, no faster than polite.

Multi-condition requests search per condition, merge, and dedupe by
NCT id before ranking.

## The scoring model (src/match_score.rs)

| Signal | Weight | Rationale |
|--------|--------|-----------|
| Condition overlap — token Jaccard between patient conditions and trial conditions | 0–60 | the patient's conditions are the whole point |
| Recruiting status | 10 | enforced by the query; kept explicit in the record |
| Phase ladder (3 → 2 → 1 → other) | 0–15 | the evidence ladder |
| Site access — √(locations), capped | 0–15 | travel is a real eligibility factor |

Scores cap at 100; ties break on NCT id (stable output).

**The tokenizer detail that earns its test**: tokens keep words >3 chars
**and bare numbers** — `"2"` vs `"1"` is exactly what distinguishes
*Type 2* from *Type 1 Diabetes*. Dropping short tokens made the two
indistinguishable; the test pins this.

Every match carries `reasons` strings ("condition match 3 term(s) (75%)",
"phase: PHASE3", "25 site(s)") — the model explains itself.

## Design decisions

| Decision | Why |
|----------|-----|
| Scoring as a pure module | Weight changes are policy — they ship with tests; Phase 1 eligibility swaps in cleanly |
| Recruiting filter server-side | Actionable results only; the UI never re-filters |
| Jaccard before embeddings | Same philosophy as the literature app's TF-IDF: zero infra, deterministic, upgradable |
