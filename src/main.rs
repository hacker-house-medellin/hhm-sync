use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{get, post},
};
use next_loggers::{JsonObject, Logger, OpenTelemetryTransport, Options, Value as LogValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{env, net::SocketAddr, sync::Arc};
use syncer_rs::{ArrayMergeStrategy, MergeObservation, MergeOptions, VERSION, merge_json_observed};
use tower_http::trace::TraceLayer;
use tracing::info;

const MAX_RECONCILE_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone)]
struct AppState {
    options: MergeOptions,
    logger: Logger,
}

#[derive(Debug, Deserialize)]
struct ReconcileRequest {
    base: Value,
    incoming: Value,
}

#[derive(Debug, Serialize)]
struct ReconcileResponse {
    merged: Value,
    engine: &'static str,
    engine_version: &'static str,
    contract: &'static str,
}

fn merge_options() -> MergeOptions {
    MergeOptions {
        array_strategy: ArrayMergeStrategy::MergeByKey,
        resolve_by_timestamp: true,
        lww_keys: Some("updated_at,synced_at".into()),
        array_match_keys: Some("id".into()),
        ..MergeOptions::default()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const ROUTINE_ID: &str = "ores-routine-JBe7CmZ48gDE3lnwcl1Rd";
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let logger = ores_logger();
    let app = Router::new()
        .route(
            "/healthz",
            get(|| async { Json(serde_json::json!({"status":"ok","service":"hhm-sync"})) }),
        )
        .route("/readyz", get(|| async { StatusCode::NO_CONTENT }))
        .route("/v1/reconcile", post(reconcile))
        .route("/api/v1/reconcile", post(reconcile))
        .layer(DefaultBodyLimit::max(MAX_RECONCILE_BODY_BYTES))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            options: merge_options(),
            logger: logger.clone(),
        });

    let addr: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8084".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "Hacker House Medellín sync gateway listening");
    let _ = logger
        .info(vec![LogValue::String("sync gateway listening".to_owned())])
        .add_trace("ores-trace-RV1WFK-mqqmFOc8aghkHh", false)
        .add_routine_id(ROUTINE_ID)
        .send();
    if let Err(error) = axum::serve(listener, app).await {
        let _ = logger
            .error(vec![LogValue::String(
                "sync gateway server stopped with an I/O error".to_owned(),
            )])
            .add_trace("ores-trace-Tt411-LzdIu8cBbmOMofN", false)
            .add_routine_id(ROUTINE_ID)
            .send();
        return Err(error.into());
    }
    Ok(())
}

async fn reconcile(
    State(state): State<AppState>,
    Json(request): Json<ReconcileRequest>,
) -> Result<Json<ReconcileResponse>, (StatusCode, Json<Value>)> {
    const ROUTINE_ID: &str = "ores-routine-o4j7v9J_wcizTkyDNtInT";
    let merged = reconcile_values(
        &request.base,
        &request.incoming,
        &state.options,
        &state.logger,
    )
    .map_err(|code| {
        // `code` is one of a fixed set of static failure categories; request
        // payload values never reach the log record.
        let _ = state
            .logger
            .warn(vec![LogValue::String(
                "sync reconciliation rejected".to_owned(),
            )])
            .add_fields(JsonObject::from_iter([(
                "reconcile.failure_code".to_owned(),
                LogValue::String(code.to_owned()),
            )]))
            .add_trace("ores-trace-Y4HriM8GKzXC1HKvwxSX9", false)
            .add_routine_id(ROUTINE_ID)
            .send();
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({
                "error": "merge_failed",
                "code": code
            })),
        )
    })?;

    Ok(Json(ReconcileResponse {
        merged,
        engine: "opto-sync/syncer.rs",
        engine_version: VERSION,
        contract: "generic-json-compat-v1",
    }))
}

fn reconcile_values(
    base: &Value,
    incoming: &Value,
    options: &MergeOptions,
    logger: &Logger,
) -> Result<Value, &'static str> {
    let base = serde_json::to_string(base).map_err(|_| "request_encoding_failed")?;
    let incoming = serde_json::to_string(incoming).map_err(|_| "request_encoding_failed")?;
    let merged = merge_json_observed(
        &base,
        &incoming,
        options,
        &|observation: &MergeObservation| record_merge_observation(logger, observation),
    )
    .map_err(|_| "merge_rejected")?;
    serde_json::from_str(&merged).map_err(|_| "response_decoding_failed")
}

fn ores_logger() -> Logger {
    let transport = OpenTelemetryTransport::new(|record| {
        tracing::info!(
            target: "ores_otel",
            otel_body = %record.body,
            otel_severity_text = %record.severity_text,
            otel_severity_number = record.severity_number,
            otel_attributes = ?record.attributes,
            "Ores structured log"
        );
        Ok(())
    });
    Logger::new(Options {
        app_name: "hhm-sync".to_owned(),
        console: false,
        transports: vec![Arc::new(transport)],
        ..Options::default()
    })
}

fn record_merge_observation(logger: &Logger, observation: &MergeObservation) {
    const ROUTINE_ID: &str = "ores-routine-Gp1yKWX5fIEPxVlRKPzGx";
    let Ok(LogValue::Object(fields)) = serde_json::to_value(observation) else {
        let _ = logger
            .warn(vec![LogValue::String(
                "sync reconciliation observation could not be encoded".to_owned(),
            )])
            .add_trace("ores-trace-4in20RfZoc2DvJQ7oeOb4", false)
            .add_routine_id(ROUTINE_ID)
            .send();
        return;
    };
    let _ = logger
        .info(vec![LogValue::String("sync reconciliation".to_owned())])
        .add_fields(fields)
        .add_trace("ores-trace-wwgBnzKgej0Az6iVVtdez", false)
        .add_routine_id(ROUTINE_ID)
        .send();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_reservations_by_id_and_accepts_newer_state() {
        let base = serde_json::json!([
            {"id":"stay-1","updated_at":"2026-08-04T10:00:00Z","status":"pending"}
        ]);
        let incoming = serde_json::json!([
            {"id":"stay-1","updated_at":"2026-08-04T11:00:00Z","status":"confirmed"},
            {"id":"stay-2","updated_at":"2026-08-04T11:00:00Z","status":"pending"}
        ]);
        let merged = reconcile_values(&base, &incoming, &merge_options(), &ores_logger()).unwrap();
        assert_eq!(merged[0]["status"], "confirmed");
        assert_eq!(merged[1]["id"], "stay-2");
    }

    #[test]
    fn rejects_older_last_writer_state() {
        let base = serde_json::json!([
            {"id":"stay-1","updated_at":"2026-08-04T11:00:00Z","status":"confirmed"}
        ]);
        let incoming = serde_json::json!([
            {"id":"stay-1","updated_at":"2026-08-04T10:00:00Z","status":"pending"}
        ]);
        let merged = reconcile_values(&base, &incoming, &merge_options(), &ores_logger()).unwrap();
        assert_eq!(merged[0]["status"], "confirmed");
    }
}
