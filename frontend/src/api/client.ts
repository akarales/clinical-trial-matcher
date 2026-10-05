import type { MatchResult } from './types';

export async function postMatch(conditions: string[]): Promise<MatchResult> {
  const res = await fetch('/api/v1/match', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ conditions }),
  });
  if (!res.ok) {
    const body = await res.text();
    throw new Error(body || res.statusText);
  }
  return (await res.json()) as MatchResult;
}
