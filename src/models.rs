use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

// Core TAMS data types

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentFormat {
    #[serde(rename = "urn:x-nmos:format:video")]
    Video,
    #[serde(rename = "urn:x-tam:format:image")]
    Image,
    #[serde(rename = "urn:x-nmos:format:audio")]
    Audio,
    #[serde(rename = "urn:x-nmos:format:data")]
    Data,
    #[serde(rename = "urn:x-nmos:format:multi")]
    Multi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: String,  // Timestamp format: "seconds:nanoseconds"
    pub end: String, // Changed from Option<String> to String to match handlers
}

impl Default for TimeRange {
    fn default() -> Self {
        TimeRange {
            start: "0:0".to_string(),
            end: "0:0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: Uuid,
    pub format: ContentFormat,
    pub label: Option<String>,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Flow {
    pub id: Uuid,
    pub source_id: Option<Uuid>,
    pub format: ContentFormat,
    pub label: Option<String>,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
    pub read_only: Option<bool>,
    pub max_bit_rate: Option<u64>,
    pub avg_bit_rate: Option<u64>,
    pub container: Option<String>,
    pub codec: Option<String>,
    pub frame_width: Option<u32>,
    pub frame_height: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub flow_collection: Option<FlowCollection>,
    pub available_timerange: Option<TimeRange>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowCollection {
    pub flows: Vec<FlowCollectionItem>,
}

impl Default for FlowCollection {
    fn default() -> Self {
        FlowCollection {
            flows: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowCollectionItem {
    pub flow_id: Uuid,
    pub role: Option<String>,
    pub container_map: Option<ContainerMap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMap {
    pub track_id: Option<String>,
    pub program_id: Option<String>,
    pub stream_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct FlowSegment {
    pub flow_id: Uuid,
    pub object_id: String,
    pub timerange: String, // Changed from TimeRange to String to match database storage
    pub ts_offset: Option<String>,
    pub sample_offset: Option<u64>,
    pub sample_count: Option<u64>,
    pub key_frame_count: Option<u32>, // Changed from u64 to u32 to match database usage
    pub get_urls: HashMap<String, String>, // Changed from Option<Vec<GetUrl>> to HashMap to match database usage
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUrl {
    pub url: String,
    pub label: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStorage {
    pub objects: Vec<StorageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageObject {
    pub object_id: String,
    pub put_url: String,
    pub put_headers: Option<HashMap<String, String>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStorageRequest {
    pub limit: Option<u32>,
    pub object_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaObject {
    pub object_id: String,
    pub size_bytes: Option<u64>,
    pub mime_type: Option<String>,
    pub flow_references: Vec<Uuid>, // Changed from Vec<FlowReference> to Vec<Uuid> to match database usage
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowReference {
    pub flow_id: Uuid,
    pub timerange: TimeRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionRequest {
    pub id: String,
    pub flow_id: Uuid, // Changed from Option<Uuid> to Uuid to match database usage
    pub timerange: Option<String>, // Changed to Option<String> to match database usage
    pub status: String, // Changed from DeletionStatus to String to match database usage
    pub progress: Option<i32>, // Changed to Option<i32> to match database usage
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub media_store_type: String,
    pub event_stream_mechanisms: Vec<String>,
    pub capabilities: ServiceCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    pub supports_webhooks: bool,
    pub supports_flow_deletion: bool,
    pub supports_segment_deletion: bool,
    pub supports_read_only_flows: bool,
    pub max_file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub url: String,
    pub api_key_name: Option<String>,
    pub api_key_value: Option<String>, // Only for requests, omitted in responses
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRequest {
    pub url: String,
    pub api_key_name: Option<String>,
    pub api_key_value: String,
    pub events: Vec<String>,
}

// Request DTOs (Data Transfer Objects) for API endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSourceRequest {
    pub id: Uuid,
    pub format: ContentFormat,
    pub label: Option<String>,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
}

impl CreateSourceRequest {
    pub fn into_source(self) -> Source {
        let now = Utc::now();
        Source {
            id: self.id,
            format: self.format,
            label: self.label,
            description: self.description,
            tags: self.tags,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFlowRequest {
    pub id: Option<Uuid>,
    pub source_id: Option<Uuid>,
    pub format: Option<ContentFormat>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
    pub read_only: Option<bool>,
    pub max_bit_rate: Option<u64>,
    pub avg_bit_rate: Option<u64>,
    pub container: Option<String>,
    pub codec: Option<String>,
    pub frame_width: Option<u32>,
    pub frame_height: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub flow_collection: Option<FlowCollection>,
    pub available_timerange: Option<TimeRange>,
}

impl CreateFlowRequest {
    pub fn into_flow(self) -> Flow {
        let now = Utc::now();
        Flow {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            source_id: self.source_id,
            format: self.format.unwrap_or(ContentFormat::Data),
            label: self.label,
            description: self.description,
            tags: self.tags,
            read_only: self.read_only,
            max_bit_rate: self.max_bit_rate,
            avg_bit_rate: self.avg_bit_rate,
            container: self.container,
            codec: self.codec,
            frame_width: self.frame_width,
            frame_height: self.frame_height,
            sample_rate: self.sample_rate,
            channels: self.channels,
            flow_collection: self.flow_collection,
            available_timerange: self.available_timerange,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSourceRequest {
    pub format: Option<ContentFormat>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub tags: Option<HashMap<String, String>>,
}

impl UpdateSourceRequest {
    pub fn apply_to_source(self, mut source: Source) -> Source {
        if let Some(format) = self.format {
            source.format = format;
        }
        if let Some(label) = self.label {
            source.label = Some(label);
        }
        if let Some(description) = self.description {
            source.description = Some(description);
        }
        if let Some(tags) = self.tags {
            source.tags = tags;
        }
        source.updated_at = Utc::now();
        source
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFlowRequest {
    pub source_id: Option<Uuid>,
    pub format: Option<ContentFormat>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub tags: Option<HashMap<String, String>>,
    pub read_only: Option<bool>,
    pub max_bit_rate: Option<u64>,
    pub avg_bit_rate: Option<u64>,
    pub container: Option<String>,
    pub codec: Option<String>,
    pub frame_width: Option<u32>,
    pub frame_height: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub flow_collection: Option<FlowCollection>,
    pub available_timerange: Option<TimeRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSegmentRequest {
    pub object_id: String,
    pub timerange: TimeRange,
    pub ts_offset: Option<String>,
    pub sample_offset: Option<u64>,
    pub sample_count: Option<u64>,
    pub key_frame_count: Option<u32>,
}

impl CreateSegmentRequest {
    pub fn into_segment(self, flow_id: Uuid) -> FlowSegment {
        let now = Utc::now();
        let timerange_str = format!("{}:{}", self.timerange.start, self.timerange.end);
        
        FlowSegment {
            flow_id,
            object_id: self.object_id,
            timerange: timerange_str,
            ts_offset: self.ts_offset,
            sample_offset: self.sample_offset,
            sample_count: self.sample_count,
            key_frame_count: self.key_frame_count,
            get_urls: HashMap::new(),
            created_at: now,
        }
    }
}

impl UpdateFlowRequest {
    pub fn apply_to_flow(self, mut flow: Flow) -> Flow {
        if let Some(source_id) = self.source_id {
            flow.source_id = Some(source_id);
        }
        if let Some(format) = self.format {
            flow.format = format;
        }
        if let Some(label) = self.label {
            flow.label = Some(label);
        }
        if let Some(description) = self.description {
            flow.description = Some(description);
        }
        if let Some(tags) = self.tags {
            flow.tags = tags;
        }
        if let Some(read_only) = self.read_only {
            flow.read_only = Some(read_only);
        }
        if let Some(max_bit_rate) = self.max_bit_rate {
            flow.max_bit_rate = Some(max_bit_rate);
        }
        if let Some(avg_bit_rate) = self.avg_bit_rate {
            flow.avg_bit_rate = Some(avg_bit_rate);
        }
        if let Some(container) = self.container {
            flow.container = Some(container);
        }
        if let Some(codec) = self.codec {
            flow.codec = Some(codec);
        }
        if let Some(frame_width) = self.frame_width {
            flow.frame_width = Some(frame_width);
        }
        if let Some(frame_height) = self.frame_height {
            flow.frame_height = Some(frame_height);
        }
        if let Some(sample_rate) = self.sample_rate {
            flow.sample_rate = Some(sample_rate);
        }
        if let Some(channels) = self.channels {
            flow.channels = Some(channels);
        }
        if let Some(flow_collection) = self.flow_collection {
            flow.flow_collection = Some(flow_collection);
        }
        if let Some(available_timerange) = self.available_timerange {
            flow.available_timerange = Some(available_timerange);
        }
        flow.updated_at = Utc::now();
        flow
    }
}

// Pagination support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u32>,
    pub page: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub limit: u32,
    pub next_key: Option<String>,
    pub count: Option<u64>,
    pub timerange: Option<TimeRange>,
    pub reverse_order: Option<bool>,
}

// Event notifications for webhooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventNotification<T> {
    pub event_timestamp: DateTime<Utc>,
    pub event_type: String,
    pub event: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowCreatedEvent {
    pub flow: Flow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowUpdatedEvent {
    pub flow: Flow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowDeletedEvent {
    pub flow_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentsAddedEvent {
    pub flow_id: Uuid,
    pub segments: Vec<FlowSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentsDeletedEvent {
    pub flow_id: Uuid,
    pub timerange: TimeRange,
}

// Bulk operations support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSegmentBulkFailure {
    pub failed_segments: Vec<FlowSegmentFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSegmentFailure {
    pub segment: FlowSegment,
    pub error: String,
}

// Helper implementations
impl TimeRange {
    pub fn new(start: &str, end: Option<&str>) -> Self {
        Self {
            start: start.to_string(),
            end: end.map(|s| s.to_string()).unwrap_or_default(),
        }
    }

    pub fn is_valid(&self) -> bool {
        // Basic validation - should be extended with proper timestamp parsing
        !self.start.is_empty() && !self.end.is_empty()
    }

    pub fn overlaps(&self, _other: &TimeRange) -> bool {
        // TODO: Implement actual overlap detection logic
        // For now, return false as a placeholder
        false
    }
}

impl Flow {
    pub fn new(id: Uuid, format: ContentFormat) -> Self {
        let now = Utc::now();
        Self {
            id,
            source_id: None,
            format,
            label: None,
            description: None,
            tags: HashMap::new(),
            read_only: None,
            max_bit_rate: None,
            avg_bit_rate: None,
            container: None,
            codec: None,
            frame_width: None,
            frame_height: None,
            sample_rate: None,
            channels: None,
            flow_collection: None,
            available_timerange: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only.unwrap_or(false)
    }
}

impl Source {
    pub fn new(id: Uuid, format: ContentFormat) -> Self {
        let now = Utc::now();
        Self {
            id,
            format,
            label: None,
            description: None,
            tags: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
} 