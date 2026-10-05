import { useCallback, useState } from 'react';

import { postMatch } from '@/api/client';
import type { MatchResult } from '@/api/types';

const SAMPLE_CONDITIONS = [
  ['Type 2 Diabetes'],
  ['Asthma'],
  ['Heart Failure', 'Chronic Kidney Disease'],
];

export default function App() {
  const [conditions, setConditions] = useState<string[]>(SAMPLE_CONDITIONS[0] ?? []);
  const [input, setInput] = useState('Type 2 Diabetes');
  const [result, setResult] = useState<MatchResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(async (list: string[]) => {
    const cleaned = list.map((c) => c.trim()).filter(Boolean);
    if (cleaned.length === 0) return;
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      setResult(await postMatch(cleaned));
      setConditions(cleaned);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }, []);

  return (
    <div className="flex h-full flex-col">
      <header className="border-b border-line bg-panel px-4 py-3">
        <h1 className="text-base font-semibold">Clinical Trial Matcher</h1>
        <p className="text-xs text-muted">
          Recruiting trials from ClinicalTrials.gov (API v2), ranked by
          condition match, phase, and site access — public data, not
          medical advice.
        </p>
      </header>

      <main className="mx-auto flex w-full max-w-4xl flex-1 flex-col gap-4 overflow-y-auto p-4">
        <section className="rounded-lg border border-line bg-panel p-4">
          <label className="mb-2 block text-sm font-semibold">
            Patient conditions (one per line)
          </label>
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            rows={2}
            className="w-full rounded border border-line p-2 text-sm"
          />
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={() => void run(input.split('\n'))}
              disabled={busy || !input.trim()}
              className="rounded bg-primary px-4 py-1.5 text-sm font-medium text-white disabled:opacity-50"
            >
              {busy ? 'Searching trials…' : 'Find matching trials'}
            </button>
            {SAMPLE_CONDITIONS.map(([sample]) => (
              <button
                key={sample}
                type="button"
                onClick={() => {
                  setInput(sample);
                  void run([sample]);
                }}
                className="rounded border border-line px-2 py-1 text-xs text-muted hover:bg-surface"
              >
                {sample}
              </button>
            ))}
          </div>
        </section>

        {error && (
          <p className="rounded border border-red-300 bg-red-50 p-3 text-sm text-red-800">
            {error}
          </p>
        )}

        {result && (
          <section className="flex flex-col gap-2">
            <h2 className="text-sm font-semibold">
              {result.matches.length} matches ({result.trials_found} trials
              found for {conditions.join(' + ')})
            </h2>
            {result.matches.map((match) => (
              <article
                key={match.nct_id}
                className="rounded-lg border border-line bg-panel p-3"
              >
                <div className="flex items-start justify-between gap-2">
                  <a
                    href={match.url}
                    target="_blank"
                    rel="noreferrer"
                    className="text-sm font-medium text-primary underline"
                  >
                    {match.title || match.nct_id}
                  </a>
                  <span className="shrink-0 rounded bg-accent/10 px-2 py-0.5 text-xs font-semibold text-accent">
                    {match.match_score}
                  </span>
                </div>
                <p className="mt-1 text-xs text-muted">
                  {match.nct_id} · {match.phase || 'N/A'} ·{' '}
                  {match.conditions.join(', ')} · {match.location_count} sites
                </p>
                <p className="mt-1 text-xs">{match.reasons.join(' · ')}</p>
              </article>
            ))}
            <p className="text-[10px] italic text-muted">{result.disclaimer}</p>
          </section>
        )}
      </main>
    </div>
  );
}
