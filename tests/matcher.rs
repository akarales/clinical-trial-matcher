//! Contract tests (scoring lives in `match_score` unit tests; the
//! ClinicalTrials.gov client is network I/O and stays behind the API).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;

use clinical_trial_matcher::AppState;
use clinical_trial_matcher::routes;

async fn post(path: &str, body: &str) -> (StatusCode, Value) {
    let app = routes::router(AppState::default());
    let response = app
        .oneshot(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("builds"),
        )
        .await
        .expect("request");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    (status, serde_json::from_slice(&bytes).expect("json"))
}

#[tokio::test]
async fn health_ok() {
    let app = routes::router(AppState::default());
    let response = app
        .oneshot(Request::get("/health").body(Body::empty()).expect("builds"))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn empty_conditions_rejected() {
    let (status, body) = post("/api/v1/match", r#"{"conditions": []}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("condition"));
}

#[tokio::test]
async fn whitespace_conditions_rejected() {
    let (status, _) = post("/api/v1/match", r#"{"conditions": ["  "]}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
