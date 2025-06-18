-- TAMS Database Schema
-- Time-addressable Media Store (TAMS) API v6.0
-- SQLite Database Creation Script

-- Sources table
-- Stores media sources with format information and metadata
CREATE TABLE IF NOT EXISTS sources (
    id TEXT PRIMARY KEY,
    format TEXT NOT NULL,
    label TEXT,
    description TEXT,
    tags TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Flows table  
-- Stores media flows with encoding parameters and timerange information
CREATE TABLE IF NOT EXISTS flows (
    id TEXT PRIMARY KEY,
    source_id TEXT,
    format TEXT NOT NULL,
    label TEXT,
    description TEXT,
    tags TEXT NOT NULL,
    read_only INTEGER,
    max_bit_rate INTEGER,
    avg_bit_rate INTEGER,
    container TEXT,
    codec TEXT,
    frame_width INTEGER,
    frame_height INTEGER,
    sample_rate INTEGER,
    channels INTEGER,
    flow_collection TEXT,
    available_timerange TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (source_id) REFERENCES sources (id) ON DELETE SET NULL
);

-- Flow segments table
-- Stores time-bounded segments within flows
CREATE TABLE IF NOT EXISTS flow_segments (
    flow_id TEXT NOT NULL,
    object_id TEXT NOT NULL,
    timerange TEXT NOT NULL,
    ts_offset TEXT,
    sample_offset INTEGER,
    sample_count INTEGER,
    key_frame_count INTEGER,
    get_urls TEXT,
    created_at TEXT NOT NULL,
    PRIMARY KEY (flow_id, object_id, timerange),
    FOREIGN KEY (flow_id) REFERENCES flows (id) ON DELETE CASCADE
);

-- Media objects table
-- Stores actual media file metadata and flow references
CREATE TABLE IF NOT EXISTS media_objects (
    object_id TEXT PRIMARY KEY,
    size_bytes INTEGER,
    mime_type TEXT,
    flow_references TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Webhooks table
-- Stores registered webhook endpoints for event notifications
CREATE TABLE IF NOT EXISTS webhooks (
    url TEXT PRIMARY KEY,
    api_key_name TEXT,
    api_key_value TEXT,
    events TEXT NOT NULL
);

-- Deletion requests table
-- Stores flow deletion requests and their processing status
CREATE TABLE IF NOT EXISTS deletion_requests (
    id TEXT PRIMARY KEY,
    flow_id TEXT,
    timerange TEXT,
    status TEXT NOT NULL,
    progress TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Create indexes for better query performance

-- Sources indexes
CREATE INDEX IF NOT EXISTS idx_sources_format ON sources(format);
CREATE INDEX IF NOT EXISTS idx_sources_created_at ON sources(created_at);

-- Flows indexes  
CREATE INDEX IF NOT EXISTS idx_flows_source_id ON flows(source_id);
CREATE INDEX IF NOT EXISTS idx_flows_format ON flows(format);
CREATE INDEX IF NOT EXISTS idx_flows_created_at ON flows(created_at);
CREATE INDEX IF NOT EXISTS idx_flows_codec ON flows(codec);

-- Flow segments indexes
CREATE INDEX IF NOT EXISTS idx_flow_segments_flow_id ON flow_segments(flow_id);
CREATE INDEX IF NOT EXISTS idx_flow_segments_object_id ON flow_segments(object_id);
CREATE INDEX IF NOT EXISTS idx_flow_segments_created_at ON flow_segments(created_at);

-- Media objects indexes
CREATE INDEX IF NOT EXISTS idx_media_objects_created_at ON media_objects(created_at);
CREATE INDEX IF NOT EXISTS idx_media_objects_size ON media_objects(size_bytes);

-- Deletion requests indexes
CREATE INDEX IF NOT EXISTS idx_deletion_requests_status ON deletion_requests(status);
CREATE INDEX IF NOT EXISTS idx_deletion_requests_flow_id ON deletion_requests(flow_id);
CREATE INDEX IF NOT EXISTS idx_deletion_requests_created_at ON deletion_requests(created_at);

-- Insert default service information (optional)
-- You can uncomment and modify these if you want to pre-populate data

-- INSERT OR IGNORE INTO sources (id, format, label, description, tags, created_at, updated_at) 
-- VALUES (
--     'default-source-id',
--     '"urn:x-nmos:format:video"',
--     'Default Video Source',
--     'Default video source for testing',
--     '{}',
--     datetime('now'),
--     datetime('now')
-- );

-- Comments about the schema:
-- 
-- 1. All timestamps are stored as ISO 8601 strings for SQLite compatibility
-- 2. JSON fields (tags, flow_collection, etc.) are stored as TEXT and parsed by the application
-- 3. UUIDs are stored as TEXT strings
-- 4. Foreign key constraints maintain referential integrity
-- 5. Indexes are created for commonly queried fields to improve performance
-- 6. The schema supports all TAMS API v6.0 features including:
--    - Sources with format and metadata
--    - Flows with encoding parameters  
--    - Time-bounded flow segments
--    - Media object storage references
--    - Webhook event notifications
--    - Asynchronous flow deletion requests 