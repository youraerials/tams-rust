use crate::models::*;
use crate::error::{TamsError, TamsResult};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite, SqlitePool, Row};
use std::collections::HashMap;
use uuid::Uuid;
use serde_json;
use std::path::Path;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(database_url: &str, _max_connections: u32) -> TamsResult<Self> {
        // Extract the file path from the sqlite:// URL
        let file_path = if database_url.starts_with("sqlite:") {
            database_url.strip_prefix("sqlite:").unwrap_or(database_url)
        } else {
            database_url
        };

        let pool = SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(file_path)
                .create_if_missing(true)
        )
        .await?;

        Ok(Database { pool })
    }

    pub async fn migrate(&self) -> TamsResult<()> {
        // Read and execute the schema
        let schema = std::fs::read_to_string("create_db.sql")?;
        sqlx::raw_sql(&schema).execute(&self.pool).await?;
        Ok(())
    }

    // Source operations
    pub async fn create_source(&self, source: &Source) -> TamsResult<()> {
        let source_id = source.id.to_string();
        let format_str = serde_json::to_string(&source.format)?;
        let tags_str = serde_json::to_string(&source.tags)?;
        let created_at = source.created_at.to_rfc3339();
        let updated_at = source.updated_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO sources (id, format, label, description, tags, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            source_id,
            format_str,
            source.label,
            source.description,
            tags_str,
            created_at,
            updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_source(&self, id: &Uuid) -> TamsResult<Option<Source>> {
        let id_str = id.to_string();
        let rows = sqlx::query!(
            "SELECT id, format, label, description, tags, created_at, updated_at FROM sources WHERE id = ?1",
            id_str
        )
        .fetch_all(&self.pool)
        .await?;

        if let Some(row) = rows.first() {
            Ok(Some(Source {
                id: Uuid::parse_str(row.id.as_ref().ok_or_else(|| TamsError::InvalidInput("Missing id".to_string()))?)?,
                format: serde_json::from_str(&row.format)?,
                label: row.label.clone(),
                description: row.description.clone(),
                tags: serde_json::from_str(&row.tags)?,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.updated_at)?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_source_required(&self, id: &Uuid) -> TamsResult<Source> {
        self.get_source(id).await?.ok_or_else(|| TamsError::NotFound("Source not found".to_string()))
    }

    pub async fn list_sources(&self) -> TamsResult<Vec<Source>> {
        let rows = sqlx::query!(
            "SELECT id, format, label, description, tags, created_at, updated_at FROM sources"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut sources = Vec::new();
        for row in rows {
            sources.push(Source {
                id: Uuid::parse_str(row.id.as_ref().ok_or_else(|| TamsError::InvalidInput("Missing id".to_string()))?)?,
                format: serde_json::from_str(&row.format)?,
                label: row.label,
                description: row.description,
                tags: serde_json::from_str(&row.tags)?,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.updated_at)?.with_timezone(&Utc),
            });
        }
        Ok(sources)
    }

    pub async fn update_source(&self, source: &Source) -> TamsResult<()> {
        let source_id = source.id.to_string();
        let format_str = serde_json::to_string(&source.format)?;
        let tags_str = serde_json::to_string(&source.tags)?;
        let updated_at = source.updated_at.to_rfc3339();

        sqlx::query!(
            r#"
            UPDATE sources 
            SET format = ?2, label = ?3, description = ?4, tags = ?5, updated_at = ?6
            WHERE id = ?1
            "#,
            source_id,
            format_str,
            source.label,
            source.description,
            tags_str,
            updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_source(&self, id: &Uuid) -> TamsResult<()> {
        let id_str = id.to_string();
        sqlx::query!("DELETE FROM sources WHERE id = ?1", id_str)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Flow operations
    pub async fn create_flow(&self, flow: &Flow) -> TamsResult<()> {
        let flow_id = flow.id.to_string();
        let source_id = flow.source_id.map(|id| id.to_string());
        let format_str = serde_json::to_string(&flow.format)?;
        let tags_str = serde_json::to_string(&flow.tags)?;
        let flow_collection_str = flow.flow_collection.as_ref().map(|fc| serde_json::to_string(fc).unwrap_or_default());
        let available_timerange_str = flow.available_timerange.as_ref().map(|tr| serde_json::to_string(tr).unwrap_or_default());
        let max_bit_rate = flow.max_bit_rate.map(|v| v as i64);
        let avg_bit_rate = flow.avg_bit_rate.map(|v| v as i64);
        let frame_width = flow.frame_width.map(|v| v as i64);
        let frame_height = flow.frame_height.map(|v| v as i64);
        let sample_rate = flow.sample_rate.map(|v| v as i64);
        let channels = flow.channels.map(|v| v as i64);
        let created_at = flow.created_at.to_rfc3339();
        let updated_at = flow.updated_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO flows (
                id, source_id, format, label, description, tags, read_only,
                max_bit_rate, avg_bit_rate, container, codec, frame_width,
                frame_height, sample_rate, channels, flow_collection,
                available_timerange, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            "#,
            flow_id,
            source_id,
            format_str,
            flow.label,
            flow.description,
            tags_str,
            flow.read_only,
            max_bit_rate,
            avg_bit_rate,
            flow.container,
            flow.codec,
            frame_width,
            frame_height,
            sample_rate,
            channels,
            flow_collection_str,
            available_timerange_str,
            created_at,
            updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_flow(&self, id: &Uuid) -> TamsResult<Option<Flow>> {
        let id_str = id.to_string();
        let rows = sqlx::query!(
            "SELECT * FROM flows WHERE id = ?1",
            id_str
        )
        .fetch_all(&self.pool)
        .await?;

        if let Some(row) = rows.first() {
            let flow_collection = row.flow_collection.as_ref()
                .and_then(|fc| serde_json::from_str(fc).ok());
            let available_timerange = row.available_timerange.as_ref()
                .and_then(|tr| serde_json::from_str(tr).ok());
                
            Ok(Some(Flow {
                id: Uuid::parse_str(row.id.as_ref().ok_or_else(|| TamsError::InvalidInput("Missing id".to_string()))?)?,
                source_id: row.source_id.as_ref().map(|s| Uuid::parse_str(s)).transpose()?,
                format: serde_json::from_str(&row.format)?,
                label: row.label.clone(),
                description: row.description.clone(),
                tags: serde_json::from_str(&row.tags)?,
                read_only: row.read_only.map(|v| v != 0),
                max_bit_rate: row.max_bit_rate.map(|v| v as u64),
                avg_bit_rate: row.avg_bit_rate.map(|v| v as u64),
                container: row.container.clone(),
                codec: row.codec.clone(),
                frame_width: row.frame_width.map(|v| v as u32),
                frame_height: row.frame_height.map(|v| v as u32),
                sample_rate: row.sample_rate.map(|v| v as u32),
                channels: row.channels.map(|v| v as u32),
                flow_collection,
                available_timerange,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.updated_at)?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_flow_required(&self, id: &Uuid) -> TamsResult<Flow> {
        self.get_flow(id).await?.ok_or_else(|| TamsError::NotFound("Flow not found".to_string()))
    }

    pub async fn list_flows(&self) -> TamsResult<Vec<Flow>> {
        let rows = sqlx::query!("SELECT * FROM flows")
            .fetch_all(&self.pool)
            .await?;

        let mut flows = Vec::new();
        for row in rows {
            let flow_collection = row.flow_collection.as_ref()
                .map(|fc| serde_json::from_str(fc).unwrap_or_default());
            let available_timerange = row.available_timerange.as_ref()
                .map(|tr| serde_json::from_str(tr).unwrap_or_default());
                
            flows.push(Flow {
                id: Uuid::parse_str(row.id.as_ref().ok_or_else(|| TamsError::InvalidInput("Missing id".to_string()))?)?,
                source_id: row.source_id.as_ref().map(|s| Uuid::parse_str(s)).transpose()?,
                format: serde_json::from_str(&row.format)?,
                label: row.label,
                description: row.description,
                tags: serde_json::from_str(&row.tags)?,
                read_only: row.read_only.map(|v| v != 0),
                max_bit_rate: row.max_bit_rate.map(|v| v as u64),
                avg_bit_rate: row.avg_bit_rate.map(|v| v as u64),
                container: row.container,
                codec: row.codec,
                frame_width: row.frame_width.map(|v| v as u32),
                frame_height: row.frame_height.map(|v| v as u32),
                sample_rate: row.sample_rate.map(|v| v as u32),
                channels: row.channels.map(|v| v as u32),
                flow_collection,
                available_timerange,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.updated_at)?.with_timezone(&Utc),
            });
        }
        Ok(flows)
    }

    pub async fn update_flow(&self, flow: &Flow) -> TamsResult<()> {
        let flow_id = flow.id.to_string();
        let source_id = flow.source_id.map(|id| id.to_string());
        let format_str = serde_json::to_string(&flow.format)?;
        let tags_str = serde_json::to_string(&flow.tags)?;
        let flow_collection_str = flow.flow_collection.as_ref().map(|fc| serde_json::to_string(fc).unwrap_or_default());
        let available_timerange_str = flow.available_timerange.as_ref().map(|tr| serde_json::to_string(tr).unwrap_or_default());
        let max_bit_rate = flow.max_bit_rate.map(|v| v as i64);
        let avg_bit_rate = flow.avg_bit_rate.map(|v| v as i64);
        let frame_width = flow.frame_width.map(|v| v as i64);
        let frame_height = flow.frame_height.map(|v| v as i64);
        let sample_rate = flow.sample_rate.map(|v| v as i64);
        let channels = flow.channels.map(|v| v as i64);
        let updated_at = flow.updated_at.to_rfc3339();

        sqlx::query!(
            r#"
            UPDATE flows SET
                source_id = ?2, format = ?3, label = ?4, description = ?5,
                tags = ?6, read_only = ?7, max_bit_rate = ?8, avg_bit_rate = ?9,
                container = ?10, codec = ?11, frame_width = ?12, frame_height = ?13,
                sample_rate = ?14, channels = ?15, flow_collection = ?16,
                available_timerange = ?17, updated_at = ?18
            WHERE id = ?1
            "#,
            flow_id,
            source_id,
            format_str,
            flow.label,
            flow.description,
            tags_str,
            flow.read_only,
            max_bit_rate,
            avg_bit_rate,
            flow.container,
            flow.codec,
            frame_width,
            frame_height,
            sample_rate,
            channels,
            flow_collection_str,
            available_timerange_str,
            updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_flow_segments(&self, id: &Uuid) -> TamsResult<()> {
        let id_str = id.to_string();
        sqlx::query!("DELETE FROM flow_segments WHERE flow_id = ?1", id_str)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_flow(&self, id: &Uuid) -> TamsResult<()> {
        let id_str = id.to_string();
        sqlx::query!("DELETE FROM flows WHERE id = ?1", id_str)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Flow segment operations
    pub async fn add_flow_segment(&self, segment: &FlowSegment) -> TamsResult<()> {
        let flow_id = segment.flow_id.to_string();
        let get_urls_json = serde_json::to_string(&segment.get_urls).unwrap_or_default();
        let sample_offset = segment.sample_offset.map(|v| v as i64);
        let sample_count = segment.sample_count.map(|v| v as i64);
        let key_frame_count = segment.key_frame_count.map(|v| v as i64);
        let created_at = segment.created_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO flow_segments (
                flow_id, object_id, timerange, ts_offset, sample_offset,
                sample_count, key_frame_count, get_urls, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            flow_id,
            segment.object_id,
            segment.timerange,
            segment.ts_offset,
            sample_offset,
            sample_count,
            key_frame_count,
            get_urls_json,
            created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_flow_segments(&self, flow_id: &Uuid) -> TamsResult<Vec<FlowSegment>> {
        let flow_id_str = flow_id.to_string();
        let rows = sqlx::query!(
            "SELECT * FROM flow_segments WHERE flow_id = ?1 ORDER BY ts_offset",
            flow_id_str
        )
        .fetch_all(&self.pool)
        .await?;

        let mut segments = Vec::new();
        for row in rows {
            let default_urls = "{}".to_string();
            let get_urls_str = row.get_urls.as_ref().unwrap_or(&default_urls);
            let get_urls: HashMap<String, String> = 
                serde_json::from_str(get_urls_str).unwrap_or_default();

            segments.push(FlowSegment {
                flow_id: Uuid::parse_str(&row.flow_id)?,
                object_id: row.object_id,
                timerange: row.timerange,
                ts_offset: row.ts_offset,
                sample_offset: row.sample_offset.map(|v| v as u64),
                sample_count: row.sample_count.map(|v| v as u64),
                key_frame_count: row.key_frame_count.map(|v| v as u32),
                get_urls,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
            });
        }
        Ok(segments)
    }

    // Media object operations
    pub async fn create_media_object(&self, object: &MediaObject) -> TamsResult<()> {
        let flow_references_json = serde_json::to_string(&object.flow_references).unwrap_or_default();
        let size_bytes = object.size_bytes.map(|v| v as i64);
        let created_at = object.created_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO media_objects (object_id, size_bytes, mime_type, flow_references, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            object.object_id,
            size_bytes,
            object.mime_type,
            flow_references_json,
            created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_media_object(&self, object_id: &str) -> TamsResult<Option<MediaObject>> {
        let rows = sqlx::query!(
            "SELECT * FROM media_objects WHERE object_id = ?1",
            object_id
        )
        .fetch_all(&self.pool)
        .await?;

        if let Some(row) = rows.first() {
            let flow_references: Vec<Uuid> = serde_json::from_str(&row.flow_references).unwrap_or_default();

            Ok(Some(MediaObject {
                object_id: row.object_id.as_ref().ok_or_else(|| TamsError::InvalidInput("Missing object_id".to_string()))?.clone(),
                size_bytes: row.size_bytes.map(|v| v as u64),
                mime_type: row.mime_type.clone(),
                flow_references,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_media_object_required(&self, object_id: &str) -> TamsResult<MediaObject> {
        self.get_media_object(object_id).await?.ok_or_else(|| TamsError::NotFound("Media object not found".to_string()))
    }

    // Webhook operations
    pub async fn create_webhook(&self, webhook: &Webhook) -> TamsResult<()> {
        let events_str = webhook.events.join(",");
        
        sqlx::query!(
            r#"
            INSERT INTO webhooks (url, api_key_name, api_key_value, events)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            webhook.url,
            webhook.api_key_name,
            webhook.api_key_value,
            events_str
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_webhooks_for_event(&self, event: &str) -> TamsResult<Vec<Webhook>> {
        let event_pattern = format!("%{}%", event);
        let rows = sqlx::query!(
            "SELECT * FROM webhooks WHERE events LIKE ?1",
            event_pattern
        )
        .fetch_all(&self.pool)
        .await?;

        let mut webhooks = Vec::new();
        for row in rows {
            webhooks.push(Webhook {
                url: row.url.ok_or_else(|| TamsError::InvalidInput("Missing url".to_string()))?,
                api_key_name: row.api_key_name,
                api_key_value: row.api_key_value,
                events: row.events.split(',').map(|s| s.to_string()).collect(),
            });
        }
        Ok(webhooks)
    }

    pub async fn get_webhooks_list(&self) -> TamsResult<Vec<Webhook>> {
        let rows = sqlx::query!("SELECT * FROM webhooks")
            .fetch_all(&self.pool)
            .await?;

        let mut webhooks = Vec::new();
        for row in rows {
            webhooks.push(Webhook {
                url: row.url.ok_or_else(|| TamsError::InvalidInput("Missing url".to_string()))?,
                api_key_name: row.api_key_name,
                api_key_value: None, // Don't return the actual key value for security
                events: row.events.split(',').map(|s| s.to_string()).collect(),
            });
        }
        Ok(webhooks)
    }

    // Deletion request operations
    pub async fn create_deletion_request(&self, request: &DeletionRequest) -> TamsResult<()> {
        let flow_id_str = request.flow_id.to_string();
        let created_at = request.created_at.to_rfc3339();
        let updated_at = request.updated_at.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO deletion_requests (id, flow_id, timerange, status, progress, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            request.id,
            flow_id_str,
            request.timerange,
            request.status,
            request.progress,
            created_at,
            updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_deletion_requests(&self) -> TamsResult<Vec<DeletionRequest>> {
        let rows = sqlx::query!("SELECT * FROM deletion_requests ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut requests = Vec::new();
        for row in rows {
            let flow_id_str = row.flow_id.as_ref().ok_or_else(|| TamsError::InvalidInput("Missing flow_id".to_string()))?;
            let progress = row.progress.as_ref().and_then(|p| p.parse::<i32>().ok());
            
            requests.push(DeletionRequest {
                id: row.id.ok_or_else(|| TamsError::InvalidInput("Missing id".to_string()))?,
                flow_id: Uuid::parse_str(flow_id_str)?,
                timerange: row.timerange,
                status: row.status,
                progress,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.updated_at)?.with_timezone(&Utc),
            });
        }
        Ok(requests)
    }

    pub async fn get_deletion_request(&self, id: &str) -> TamsResult<Option<DeletionRequest>> {
        let rows = sqlx::query!(
            "SELECT * FROM deletion_requests WHERE id = ?1",
            id
        )
        .fetch_all(&self.pool)
        .await?;

        if let Some(row) = rows.first() {
            let flow_id_str = row.flow_id.as_ref().ok_or_else(|| TamsError::InvalidInput("Missing flow_id".to_string()))?;
            let progress = row.progress.as_ref().and_then(|p| p.parse::<i32>().ok());
            
            Ok(Some(DeletionRequest {
                id: row.id.as_ref().ok_or_else(|| TamsError::InvalidInput("Missing id".to_string()))?.clone(),
                flow_id: Uuid::parse_str(flow_id_str)?,
                timerange: row.timerange.clone(),
                status: row.status.clone(),
                progress,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.updated_at)?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_deletion_request_required(&self, id: &str) -> TamsResult<DeletionRequest> {
        self.get_deletion_request(id).await?.ok_or_else(|| TamsError::NotFound("Deletion request not found".to_string()))
    }

    // Helper methods for handlers
    pub async fn get_sources(&self, _limit: u32, _page: Option<&str>) -> TamsResult<Vec<Source>> {
        self.list_sources().await
    }

    pub async fn get_flows(&self, _limit: u32, _page: Option<&str>) -> TamsResult<Vec<Flow>> {
        self.list_flows().await
    }

    pub async fn delete_flow_segments_by_timerange(&self, flow_id: &Uuid, _timerange: &TimeRange) -> TamsResult<()> {
        // For now, delete all segments for the flow
        // In a real implementation, you'd filter by timerange
        self.delete_flow_segments(flow_id).await
    }

    pub async fn get_flow_segments_by_timerange(
        &self, 
        flow_id: &Uuid, 
        _timerange: Option<&TimeRange>, 
        _limit: u32
    ) -> TamsResult<Vec<FlowSegment>> {
        // For now, return all segments for the flow
        // In a real implementation, you'd filter by timerange and limit
        self.get_flow_segments(flow_id).await
    }
}

// Filter structs for queries
#[derive(Debug, Default)]
pub struct SourceFilters {
    pub format: Option<ContentFormat>,
    pub label: Option<String>,
}

#[derive(Debug, Default)]
pub struct FlowFilters {
    pub source_id: Option<Uuid>,
    pub format: Option<ContentFormat>,
    pub label: Option<String>,
    pub codec: Option<String>,
    pub frame_width: Option<u32>,
    pub frame_height: Option<u32>,
    pub timerange: Option<TimeRange>,
}

#[derive(Debug, Default)]
pub struct FlowSegmentFilters {
    pub object_id: Option<String>,
    pub timerange: Option<TimeRange>,
    pub reverse_order: Option<bool>,
} 