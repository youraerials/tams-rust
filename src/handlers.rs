use crate::{
    config::AppConfig,
    database::Database,
    error::{TamsError, TamsResult},
    models::*,
    storage::MediaStorage,
    webhooks::WebhookManager,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    Extension,
};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub config: AppConfig,
    pub database: Database,
    pub storage: Arc<MediaStorage>,
    pub webhook_manager: Arc<WebhookManager>,
}

// Root endpoint
pub async fn get_root() -> Result<Json<Value>, TamsError> {
    Ok(Json(json!({
        "name": "TAMS API Server",
        "description": "Time-addressable Media Store API v6.0",
        "version": "0.1.0"
    })))
}

// Service info endpoint
pub async fn get_service_info(State(state): State<AppState>) -> Result<Json<ServiceInfo>, TamsError> {
    let info = ServiceInfo {
        name: "TAMS Rust Server".to_string(),
        description: "Time-addressable Media Store implementation in Rust".to_string(),
        version: "0.1.0".to_string(),
        media_store_type: "file".to_string(),
        event_stream_mechanisms: vec!["webhooks".to_string()],
        capabilities: ServiceCapabilities {
            supports_webhooks: true,
            supports_flow_deletion: true,
            supports_segment_deletion: true,
            supports_read_only_flows: true,
            max_file_size: state.config.media_storage.max_file_size,
        },
    };

    Ok(Json(info))
}

// Sources endpoints
pub async fn list_sources(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Value>, TamsError> {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100);
    let page = params.get("page");
    
    let sources = state.database.get_sources(limit, page.map(|s| s.as_str())).await?;
    
    Ok(Json(json!({
        "sources": sources,
        "pagination": {
            "limit": limit,
            "count": sources.len()
        }
    })))
}

pub async fn get_source(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Source>, TamsError> {
    let source = state.database.get_source_required(&id).await?;
    Ok(Json(source))
}

pub async fn create_source(
    State(state): State<AppState>,
    Json(payload): Json<CreateSourceRequest>,
) -> Result<Json<Source>, TamsError> {
    let source = payload.into_source();
    state.database.create_source(&source).await?;
    Ok(Json(source))
}

pub async fn update_source(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateSourceRequest>,
) -> Result<Json<Source>, TamsError> {
    let existing_source = state.database.get_source_required(&id).await?;
    let updated_source = payload.apply_to_source(existing_source);
    state.database.update_source(&updated_source).await?;
    Ok(Json(updated_source))
}

pub async fn delete_source(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, TamsError> {
    state.database.delete_source(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Flows endpoints
pub async fn list_flows(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Value>, TamsError> {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100);
    let page = params.get("page");
    
    let flows = state.database.get_flows(limit, page.map(|s| s.as_str())).await?;
    
    Ok(Json(json!({
        "flows": flows,
        "pagination": {
            "limit": limit,
            "count": flows.len()
        }
    })))
}

pub async fn get_flow(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Flow>, TamsError> {
    let flow = state.database.get_flow_required(&id).await?;
    Ok(Json(flow))
}

pub async fn create_flow(
    State(state): State<AppState>,
    Json(payload): Json<CreateFlowRequest>,
) -> Result<Json<Flow>, TamsError> {
    let flow = payload.into_flow();
    state.database.create_flow(&flow).await?;
    Ok(Json(flow))
}

pub async fn update_flow(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateFlowRequest>,
) -> Result<Json<Flow>, TamsError> {
    let existing_flow = state.database.get_flow_required(&id).await?;
    let updated_flow = payload.apply_to_flow(existing_flow);
    state.database.update_flow(&updated_flow).await?;
    Ok(Json(updated_flow))
}

pub async fn delete_flow(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, TamsError> {
    state.database.delete_flow(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Flow segments endpoints
pub async fn list_flow_segments(
    Path(flow_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Value>, TamsError> {
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100);

    let timerange = if let (Some(start), Some(end)) = (params.get("start"), params.get("end")) {
        Some(TimeRange {
            start: start.clone(),
            end: end.clone(),
        })
    } else {
        None
    };

    let segments = state.database.get_flow_segments_by_timerange(&flow_id, timerange.as_ref(), limit).await?;
    
    Ok(Json(json!({
        "segments": segments,
        "pagination": {
            "limit": limit,
            "count": segments.len()
        }
    })))
}

pub async fn add_flow_segment(
    Path(flow_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateSegmentRequest>,
) -> Result<Json<FlowSegment>, TamsError> {
    let segment = payload.into_segment(flow_id);
    state.database.add_flow_segment(&segment).await?;
    Ok(Json(segment))
}

pub async fn delete_flow_segments(
    Path(flow_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Value>, TamsError> {
    let timerange = if let (Some(start), Some(end)) = (params.get("start"), params.get("end")) {
        Some(TimeRange {
            start: start.clone(),
            end: end.clone(),
        })
    } else {
        None
    };

    // Delete segments based on timerange
    if let Some(ref tr) = timerange {
        state.database.delete_flow_segments_by_timerange(&flow_id, tr).await?;
    }

    Ok(Json(json!({ "deleted": true })))
}

// Storage endpoints
pub async fn allocate_storage(
    Path(_flow_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<FlowStorage>, TamsError> {
    // Parse limit from query parameters, default to 1
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(1);
    
    // Parse object_ids from query parameters if provided
    let object_ids = if let Some(object_ids_str) = params.get("object_ids") {
        Some(object_ids_str.split(',').map(|s| s.to_string()).collect())
    } else {
        None
    };
    
    // Use the storage allocate_storage method which creates proper StorageObjects
    let objects = state.storage.allocate_storage(limit, object_ids).await?;
    
    Ok(Json(FlowStorage { objects }))
}

// Media object endpoints
pub async fn get_media_object(
    Path(object_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<MediaObject>, TamsError> {
    let media_object = state.database.get_media_object_required(&object_id).await?;
    Ok(Json(media_object))
}

pub async fn put_media_object(
    Path(object_id): Path<String>,
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<StatusCode, TamsError> {
    // Store the uploaded data
    state.storage.store_object(&object_id, body.to_vec()).await?;
    
    // Create or update media object record in database
    let media_object = MediaObject {
        object_id: object_id.clone(),
        size_bytes: Some(body.len() as u64),
        mime_type: None, // Could be inferred from content-type header
        flow_references: Vec::new(),
        created_at: chrono::Utc::now(),
    };
    
    // Try to create the media object, ignore if it already exists
    let _ = state.database.create_media_object(&media_object).await;
    
    Ok(StatusCode::CREATED)
}

pub async fn head_media_object(
    State(state): State<AppState>,
    Path(object_id): Path<String>,
) -> TamsResult<StatusCode> {
    let _media_object = state.database.get_media_object_required(&object_id).await?;
    Ok(StatusCode::OK)
}

// Webhook endpoints
pub async fn list_webhooks(
    State(state): State<AppState>,
) -> Result<Json<Value>, TamsError> {
    let webhooks = state.database.get_webhooks_list().await?;
    
    Ok(Json(json!({
        "webhooks": webhooks
    })))
}

pub async fn create_webhook(
    State(state): State<AppState>,
    Json(payload): Json<WebhookRequest>,
) -> Result<Json<Webhook>, TamsError> {
    let webhook = Webhook {
        url: payload.url,
        api_key_name: payload.api_key_name,
        api_key_value: Some(payload.api_key_value),
        events: payload.events,
    };
    
    state.database.create_webhook(&webhook).await?;
    
    // Return webhook without the API key value for security
    let response_webhook = Webhook {
        url: webhook.url,
        api_key_name: webhook.api_key_name,
        api_key_value: None,
        events: webhook.events,
    };
    
    Ok(Json(response_webhook))
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    Path(webhook_url): Path<String>,
) -> TamsResult<StatusCode> {
    // TODO: Implement delete_webhook in database
    // state.database.delete_webhook(&webhook_url).await?;
    // state.webhook_manager.remove_webhook(&webhook_url).await;
    Ok(StatusCode::NO_CONTENT)
}

// Flow delete request endpoints
pub async fn request_flow_deletion(
    Path(flow_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<HashMap<String, Value>>,
) -> Result<Json<DeletionRequest>, TamsError> {
    let request_id = Uuid::new_v4().to_string();
    let timerange = payload.get("timerange")
        .and_then(|tr| serde_json::to_string(tr).ok());

    let request = DeletionRequest {
        id: request_id,
        flow_id,
        timerange,
        status: "pending".to_string(),
        progress: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state.database.create_deletion_request(&request).await?;
    
    Ok(Json(request))
}

pub async fn list_deletion_requests(
    State(state): State<AppState>,
) -> Result<Json<Value>, TamsError> {
    let requests = state.database.get_deletion_requests().await?;
    
    Ok(Json(json!({
        "deletion_requests": requests
    })))
}

pub async fn get_deletion_request(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DeletionRequest>, TamsError> {
    let request = state.database.get_deletion_request_required(&id).await?;
    Ok(Json(request))
}

// Test page endpoint
pub async fn get_test_page() -> Result<Html<String>, TamsError> {
    let html = include_str!("../test.html");
    Ok(Html(html.to_string()))
} 