//! ClinicalTrials.gov API v2 client.
//!
//! Etiquette: cursor pagination via nextPageToken (absence = end), cached
//! queries, no API key. `query.cond` scopes to conditions; recruiting
//! filter keeps results actionable.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trial {
    pub nct_id: String,
    pub title: String,
    pub phase: String,
    pub status: String,
    pub conditions: Vec<String>,
    pub location_count: usize,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrialPage {
    pub trials: Vec<Trial>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TrialsError {
    #[error("network error: {0}")]
    Network(String),
    #[error("bad response: {0}")]
    BadResponse(String),
}

const BASE: &str = "https://clinicaltrials.gov/api/v2/studies";

/// One page of recruiting trials for a condition.
pub async fn search_recruiting(
    http: &reqwest::Client,
    condition: &str,
    page_size: usize,
) -> Result<TrialPage, TrialsError> {
    search_recruiting_page(http, condition, page_size, None).await
}

pub async fn search_recruiting_page(
    http: &reqwest::Client,
    condition: &str,
    page_size: usize,
    page_token: Option<&str>,
) -> Result<TrialPage, TrialsError> {
    let mut params: Vec<(&str, String)> = vec![
        ("query.cond", condition.to_string()),
        ("filter.overallStatus", "RECRUITING".to_string()),
        ("pageSize", page_size.to_string()),
    ];
    if let Some(token) = page_token {
        params.push(("pageToken", token.to_string()));
    }

    let response: serde_json::Value = http
        .get(BASE)
        .query(&params)
        .send()
        .await
        .map_err(|e| TrialsError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| TrialsError::Network(e.to_string()))?
        .json()
        .await
        .map_err(|e| TrialsError::BadResponse(e.to_string()))?;

    let trials: Vec<Trial> = response["studies"]
        .as_array()
        .map(|studies| {
            studies
                .iter()
                .filter_map(|study| {
                    let protocol = &study["protocolSection"];
                    let identification = &protocol["identificationModule"];
                    let status = &protocol["statusModule"];
                    let conditions = &protocol["conditionsModule"];
                    let design = &protocol["designModule"];
                    let contacts = &protocol["contactsLocationsModule"];
                    Some(Trial {
                        nct_id: identification["nctId"].as_str()?.to_string(),
                        title: identification["briefSummary"]
                            .as_str()
                            .or_else(|| identification["officialTitle"].as_str())
                            .unwrap_or_default()
                            .to_string(),
                        phase: design["phases"]
                            .as_array()
                            .map(|phases| {
                                phases
                                    .iter()
                                    .filter_map(|p| p.as_str())
                                    .collect::<Vec<_>>()
                                    .join("/")
                            })
                            .unwrap_or_default(),
                        status: status["overallStatus"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        conditions: conditions["conditions"]
                            .as_array()
                            .map(|list| {
                                list.iter()
                                    .filter_map(|c| c.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        location_count: contacts["locations"].as_array().map(Vec::len).unwrap_or(0),
                        url: format!(
                            "https://clinicaltrials.gov/study/{}",
                            identification["nctId"].as_str()?
                        ),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(TrialPage {
        trials,
        next_page_token: response["nextPageToken"].as_str().map(String::from),
    })
}
