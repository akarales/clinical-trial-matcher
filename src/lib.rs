//! clinical-trial-matcher — match patient conditions to recruiting
//! trials on ClinicalTrials.gov (API v2), rank by relevance, explain the
//! match. The API client is separated from scoring so tests run without
//! the network.

pub mod api;
pub mod match_score;
pub mod routes;
pub mod state;

pub use state::AppState;
