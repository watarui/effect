-- Progress Command Service - Event Store
-- イベントソーシング用のテーブル

CREATE TABLE IF NOT EXISTS progress_events (
    id UUID PRIMARY KEY,
    stream_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- インデックス
CREATE INDEX idx_progress_events_stream_id ON progress_events (stream_id);
CREATE INDEX idx_progress_events_occurred_at ON progress_events (occurred_at);
CREATE INDEX idx_progress_events_event_type ON progress_events (event_type);
