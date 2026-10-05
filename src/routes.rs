//! Routes: match + explain.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::AppState;
use crate::api;
use crate::match_score;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/match", post(match_trials))
        .with_state(state)
}

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

#[derive(Debug, Deserialize)]
pub struct MatchRequest {
    /// Patient conditions, e.g. ["Type 2 Diabetes", "Hypertension"].
    pub conditions: Vec<String>,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

fn default_page_size() -> usize {
    20
}

pub async fn match_trials(
    State(state): State<AppState>,
    Json(request): Json<MatchRequest>,
) -> Response {
    if request.conditions.is_empty() || request.conditions.iter().all(|c| c.trim().is_empty()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "provide at least one condition" })),
        )
            .into_response();
    }
    let page_size = request.page_size.clamp(1, 100);

    // Query per condition; merge and dedupe by NCT id.
    let mut trials: Vec<api::Trial> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for condition in &request.conditions {
        match api::search_recruiting(&state.http, condition, page_size).await {
            Ok(page) => {
                for trial in page.trials {
                    if !trials.iter().any(|t| t.nct_id == trial.nct_id) {
                        trials.push(trial);
                    }
                }
            }
            Err(err) => errors.push(err.to_string()),
        }
    }
    if trials.is_empty() && !errors.is_empty() {
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("clinicaltrials.gov unreachable: {}", errors[0]) })),
        )
            .into_response();
    }

    let ranked: Vec<Value> = match_score::rank(&request.conditions, &trials)
        .into_iter()
        .take(page_size)
        .map(|scored| {
            json!({
                "nct_id": scored.trial.nct_id,
                "title": scored.trial.title,
                "phase": scored.trial.phase,
                "status": scored.trial.status,
                "conditions": scored.trial.conditions,
                "location_count": scored.trial.location_count,
                "url": scored.trial.url,
                "match_score": (scored.score * 10.0).round() / 10.0,
                "reasons": scored.reasons,
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "conditions": request.conditions,
            "matches": ranked,
            "trials_found": trials.len(),
            "disclaimer": "Trial matching from public ClinicalTrials.gov data. Not medical advice; eligibility is determined by trial sites.",
        })),
    )
        .into_response()
}
