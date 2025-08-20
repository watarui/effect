-- Progress Command Service 用のイベントストアテーブル

-- イベントテーブル
CREATE TABLE IF NOT EXISTS events (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_data JSONB NOT NULL,
    event_version BIGINT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
    CONSTRAINT unique_stream_version UNIQUE (stream_id, event_version)
);

-- スナップショットテーブル
CREATE TABLE IF NOT EXISTS snapshots (
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,
    snapshot_data JSONB NOT NULL,
    version BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (aggregate_id, aggregate_type)
);

-- インデックス
CREATE INDEX idx_events_stream_id ON events (stream_id);
CREATE INDEX idx_events_event_version ON events (stream_id, event_version);
CREATE INDEX idx_events_occurred_at ON events (occurred_at);

CREATE INDEX idx_snapshots_aggregate_id ON snapshots (aggregate_id);
CREATE INDEX idx_snapshots_created_at ON snapshots (created_at);
