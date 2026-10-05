export interface TrialMatch {
  nct_id: string;
  title: string;
  phase: string;
  status: string;
  conditions: string[];
  location_count: number;
  url: string;
  match_score: number;
  reasons: string[];
}

export interface MatchResult {
  conditions: string[];
  matches: TrialMatch[];
  trials_found: number;
  disclaimer: string;
}
