-- Event Store table for Event Sourcing
CREATE TABLE IF NOT EXISTS events (
    -- Event identification
    event_id VARCHAR(255) PRIMARY KEY,
    event_type VARCHAR(255) NOT NULL,
    
    -- Aggregate information
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    
    -- Event data
    event_data JSONB NOT NULL,
    event_metadata JSONB NOT NULL DEFAULT '{}',
    
    -- User tracking
    user_id VARCHAR(255),
    
    -- Timestamp
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure events are immutable
    CONSTRAINT unique_aggregate_version UNIQUE (aggregate_id, aggregate_version)
);

-- Indexes for efficient querying
CREATE INDEX idx_events_aggregate_id ON events(aggregate_id);
CREATE INDEX idx_events_aggregate_type ON events(aggregate_type);
CREATE INDEX idx_events_event_type ON events(event_type);
CREATE INDEX idx_events_occurred_at ON events(occurred_at);
CREATE INDEX idx_events_user_id ON events(user_id) WHERE user_id IS NOT NULL;

-- Composite index for aggregate event stream queries
CREATE INDEX idx_events_aggregate_stream ON events(aggregate_id, aggregate_version);

-- Add comments
COMMENT ON TABLE events IS 'Event store for Event Sourcing pattern';
COMMENT ON COLUMN events.event_id IS 'Unique event identifier';
COMMENT ON COLUMN events.event_type IS 'Type of domain event';
COMMENT ON COLUMN events.aggregate_id IS 'ID of the aggregate this event belongs to';
COMMENT ON COLUMN events.aggregate_type IS 'Type of aggregate';
COMMENT ON COLUMN events.aggregate_version IS 'Version of aggregate after this event';
COMMENT ON COLUMN events.event_data IS 'Event payload in JSON format';
COMMENT ON COLUMN events.event_metadata IS 'Event metadata (correlation ID, causation ID, etc.)';
COMMENT ON COLUMN events.user_id IS 'User who triggered this event';
COMMENT ON COLUMN events.occurred_at IS 'When the event occurred';

-- Event snapshots table for performance optimization
CREATE TABLE IF NOT EXISTS event_snapshots (
    -- Snapshot identification
    snapshot_id VARCHAR(255) PRIMARY KEY,
    
    -- Aggregate information
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    
    -- Snapshot data
    snapshot_data JSONB NOT NULL,
    
    -- Timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure one snapshot per aggregate version
    CONSTRAINT unique_aggregate_snapshot UNIQUE (aggregate_id, aggregate_version)
);

-- Indexes for snapshots
CREATE INDEX idx_event_snapshots_aggregate_id ON event_snapshots(aggregate_id);
CREATE INDEX idx_event_snapshots_aggregate_version ON event_snapshots(aggregate_id, aggregate_version DESC);

-- Add comments for snapshots
COMMENT ON TABLE event_snapshots IS 'Aggregate snapshots for performance optimization';
COMMENT ON COLUMN event_snapshots.snapshot_id IS 'Unique snapshot identifier';
COMMENT ON COLUMN event_snapshots.aggregate_id IS 'ID of the aggregate';
COMMENT ON COLUMN event_snapshots.aggregate_type IS 'Type of aggregate';
COMMENT ON COLUMN event_snapshots.aggregate_version IS 'Version of aggregate at snapshot time';
COMMENT ON COLUMN event_snapshots.snapshot_data IS 'Serialized aggregate state';
COMMENT ON COLUMN event_snapshots.created_at IS 'When the snapshot was created';