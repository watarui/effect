-- Event Store のための基本テーブル

-- イベントストリーム（集約単位）
CREATE TABLE IF NOT EXISTS event_streams (
    stream_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (aggregate_id, aggregate_type)
);

-- イベントテーブル
CREATE TABLE IF NOT EXISTS events (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id UUID NOT NULL REFERENCES event_streams (stream_id),
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_version INTEGER NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT events_stream_version_unique UNIQUE (stream_id, event_version)
);

-- スナップショットテーブル
CREATE TABLE IF NOT EXISTS snapshots (
    snapshot_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,
    aggregate_version INTEGER NOT NULL,
    aggregate_data JSONB NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT snapshots_aggregate_unique UNIQUE (aggregate_id, aggregate_type, aggregate_version)
);

-- インデックス
CREATE INDEX idx_events_aggregate_id ON events (aggregate_id);
CREATE INDEX idx_events_aggregate_type ON events (aggregate_type);
CREATE INDEX idx_events_event_type ON events (event_type);
CREATE INDEX idx_events_occurred_at ON events (occurred_at);
CREATE INDEX idx_events_stream_id_version ON events (stream_id, event_version);

CREATE INDEX idx_snapshots_aggregate_id ON snapshots (aggregate_id);
CREATE INDEX idx_snapshots_aggregate_type ON snapshots (aggregate_type);
CREATE INDEX idx_snapshots_created_at ON snapshots (created_at DESC);

-- 楽観的ロックのための関数
CREATE OR REPLACE FUNCTION get_next_event_version(p_stream_id UUID)
RETURNS INTEGER AS $$
DECLARE
    v_next_version INTEGER;
BEGIN
    SELECT COALESCE(MAX(event_version), 0) + 1 INTO v_next_version
    FROM events
    WHERE stream_id = p_stream_id;
    
    RETURN v_next_version;
END;
$$ LANGUAGE plpgsql;
