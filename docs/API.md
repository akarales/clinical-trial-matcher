# API Reference

Base URL: `http://localhost:8008`.

## Health

```bash
curl localhost:8008/health
```

```json
{ "status": "ok", "version": "0.1.0" }
```

## Match

```bash
curl -X POST localhost:8008/api/v1/match \
  -H 'content-type: application/json' \
  -d '{"conditions": ["Type 2 Diabetes"]}'
```

```json
{
  "conditions": ["Type 2 Diabetes"],
  "matches": [
    {
      "nct_id": "NCT06543210",
      "title": "SGLT2 Inhibitors in Type 2 Diabetes …",
      "phase": "PHASE3",
      "status": "RECRUITING",
      "conditions": ["Type 2 Diabetes", "Cardiovascular Risk"],
      "location_count": 42,
      "url": "https://clinicaltrials.gov/study/NCT06543210",
      "match_score": 78.5,
      "reasons": [
        "condition match 2 term(s) (60%)",
        "phase: PHASE3",
        "42 site(s)"
      ]
    }
  ],
  "trials_found": 20,
  "disclaimer": "Trial matching from public ClinicalTrials.gov data. Not medical advice; eligibility is determined by trial sites."
}
```

- Multi-condition requests merge per-condition result sets, dedupe by
  NCT id, then rank
- `page_size` (query param on the internal fetch, default 20, clamp 1–100)
  bounds each per-condition search

## Errors

| Status | Meaning |
|--------|---------|
| `400` | no conditions provided (empty or all-whitespace) |
| `502` | clinicaltrials.gov unreachable (and nothing cached) |
