use crate::service_state::ServiceState;
use crate::metrics::AggregationMetrics;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimRequest {
    chain_id: String,
    input: String,
    to: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimResponse {}

pub async fn claim_endpoint(
    State(_service): State<ServiceState>,
    Json(request): Json<ClaimRequest>,
) -> Json<ClaimResponse> {
    tracing::debug!("chain_id: {:?}", request.chain_id);
    tracing::debug!("to: {:?}", request.to);
    tracing::debug!("input: {:?}", request.input);

    // Start aggregation timer
    let metrics = AggregationMetrics::start();

    // Perform the aggregation / processing work
    run_aggregation().await;

    let elapsed = metrics.finish();
    tracing::info!("Aggregation completed in {:?}", elapsed);

    Json(ClaimResponse {})
}

/// Example aggregation entry point.
///
/// In the real service this would run the actual aggregation/pipeline logic. Here it's
/// represented as an async function that can be instrumented with `AggregationMetrics`.
async fn run_aggregation() {
    // Simulate work — replace with real aggregation logic.
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
}
