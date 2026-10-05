//! Scoring: rank trials against a patient's conditions — pure functions,
//! unit-tested. Signals: condition overlap (primary), phase clarity,
//! location availability (access), recency of title terms.

use crate::api::Trial;

#[derive(Debug, Clone, PartialEq)]
pub struct ScoredTrial {
    pub trial: Trial,
    pub score: f64,
    pub reasons: Vec<String>,
}

fn normalized_tokens(text: &str) -> std::collections::HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        // Keep words and bare numbers: "2" vs "1" distinguishes
        // Type 2 from Type 1 Diabetes.
        .filter(|t| t.len() > 3 || t.chars().all(|c| c.is_ascii_digit()) && !t.is_empty())
        .map(String::from)
        .collect()
}

/// Score one trial for a patient's conditions (0..100).
pub fn score_trial(patient_conditions: &[String], trial: &Trial) -> ScoredTrial {
    let mut score = 0.0;
    let mut reasons = Vec::new();

    // Condition overlap: token Jaccard between patient conditions and the
    // trial's condition list (0-60 points).
    let patient_tokens: std::collections::HashSet<String> = patient_conditions
        .iter()
        .flat_map(|c| normalized_tokens(c))
        .collect();
    let trial_tokens: std::collections::HashSet<String> = trial
        .conditions
        .iter()
        .flat_map(|c| normalized_tokens(c))
        .collect();
    if !patient_tokens.is_empty() && !trial_tokens.is_empty() {
        let intersection = patient_tokens.intersection(&trial_tokens).count();
        let union = patient_tokens.union(&trial_tokens).count();
        let overlap = intersection as f64 / union.max(1) as f64;
        let points = overlap * 60.0;
        if points > 0.0 {
            reasons.push(format!(
                "condition match {intersection} term(s) ({:.0}%)",
                overlap * 100.0
            ));
        }
        score += points;
    }

    // Recruiting status is enforced by the query, but keep it explicit
    // in the record (10 points; belt-and-braces for other statuses).
    if trial.status.eq_ignore_ascii_case("RECRUITING") {
        score += 10.0;
    }

    // Phase clarity: specific phases (2/3) score higher — the classic
    // evidence ladder (0-15).
    let phase = trial.phase.to_uppercase();
    let phase_points = if phase.contains("PHASE3") {
        15.0
    } else if phase.contains("PHASE2") {
        12.0
    } else if phase.contains("PHASE1") {
        8.0
    } else {
        4.0
    };
    score += phase_points;
    reasons.push(format!("phase: {}", trial.phase));

    // Access: more locations → better odds of a nearby site (0-15).
    let access_points = (trial.location_count as f64).sqrt().min(15.0);
    if access_points > 0.0 {
        reasons.push(format!("{} site(s)", trial.location_count));
    }
    score += access_points;

    ScoredTrial {
        trial: trial.clone(),
        score: score.min(100.0),
        reasons,
    }
}

/// Rank trials best-first.
pub fn rank(patient_conditions: &[String], trials: &[Trial]) -> Vec<ScoredTrial> {
    let mut scored: Vec<ScoredTrial> = trials
        .iter()
        .map(|trial| score_trial(patient_conditions, trial))
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.trial.nct_id.cmp(&b.trial.nct_id))
    });
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trial(nct: &str, conditions: &[&str], phase: &str, sites: usize) -> Trial {
        Trial {
            nct_id: nct.into(),
            title: format!("Study {nct}"),
            phase: phase.into(),
            status: "RECRUITING".into(),
            conditions: conditions.iter().map(|c| c.to_string()).collect(),
            location_count: sites,
            url: format!("https://clinicaltrials.gov/study/{nct}"),
        }
    }

    #[test]
    fn exact_condition_match_outranks_partial() {
        let patient = vec!["Type 2 Diabetes".to_string()];
        let trials = vec![
            trial("NCT1", &["Type 2 Diabetes"], "PHASE3", 25),
            trial("NCT2", &["Type 1 Diabetes"], "PHASE3", 25),
            trial("NCT3", &["Asthma"], "PHASE2", 25),
        ];
        let ranked = rank(&patient, &trials);
        assert_eq!(ranked[0].trial.nct_id, "NCT1");
        assert!(ranked[0].score > ranked[1].score, "type 2 beats type 1");
        assert!(ranked[1].score > ranked[2].score, "diabetes beats asthma");
    }

    #[test]
    fn phase_and_sites_contribute() {
        let patient = vec!["Asthma".to_string()];
        let ranked = rank(
            &patient,
            &[
                trial("NCT1", &["Asthma"], "PHASE3", 100),
                trial("NCT2", &["Asthma"], "PHASE1", 0),
            ],
        );
        assert!(ranked[0].score > ranked[1].score);
        assert!(ranked[0].reasons.iter().any(|r| r.contains("phase")));
    }

    #[test]
    fn scores_capped_at_100() {
        let patient = vec!["Asthma".to_string()];
        let scored = score_trial(&patient, &trial("NCT1", &["Asthma"], "PHASE3", 900));
        assert!(scored.score <= 100.0);
    }
}
