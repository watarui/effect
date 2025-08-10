-- Event Store Service のテーブル

-- イベントストリームテーブル
CREATE TABLE IF NOT EXISTS event_streams (
    stream_id UUID NOT NULL,
    stream_type VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,  -- 集約タイプ（VocabularyEntry, VocabularyItem など）
    version BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (stream_id, stream_type)
);

-- イベントテーブル
CREATE TABLE IF NOT EXISTS events (
    event_id UUID PRIMARY KEY DEFAULT GEN_RANDOM_UUID(),
    stream_id UUID NOT NULL,
    stream_type VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,  -- 集約タイプ
    version BIGINT NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    metadata JSONB,
    correlation_id UUID,  -- Saga パターン用の相関ID
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    position BIGSERIAL,  -- グローバル順序保証
    CONSTRAINT unique_stream_version UNIQUE (stream_id, stream_type, version),
    FOREIGN KEY (stream_id, stream_type) REFERENCES event_streams (stream_id, stream_type)
);

-- スナップショットテーブル
CREATE TABLE IF NOT EXISTS snapshots (
    snapshot_id UUID PRIMARY KEY DEFAULT GEN_RANDOM_UUID(),
    stream_id UUID NOT NULL,
    stream_type VARCHAR(255) NOT NULL,
    version BIGINT NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_stream_snapshot UNIQUE (stream_id, stream_type, version),
    FOREIGN KEY (stream_id, stream_type) REFERENCES event_streams (stream_id, stream_type)
);

-- インデックス
CREATE INDEX idx_events_stream_id_version ON events (stream_id, stream_type, version);
CREATE INDEX idx_events_position ON events (position);
CREATE INDEX idx_events_event_type ON events (event_type);
CREATE INDEX idx_events_aggregate_type ON events (aggregate_type);
CREATE INDEX idx_events_correlation_id ON events (correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_events_created_at ON events (created_at);

CREATE INDEX idx_snapshots_stream_id_version ON snapshots (stream_id, stream_type, version DESC);
CREATE INDEX idx_snapshots_created_at ON snapshots (created_at);

-- サブスクリプション状態テーブル（将来の拡張用）
CREATE TABLE IF NOT EXISTS subscriptions (
    subscription_id VARCHAR(255) PRIMARY KEY,
    stream_id UUID,
    stream_type VARCHAR(255),
    last_position BIGINT NOT NULL DEFAULT 0,
    last_version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_subscriptions_stream ON subscriptions (stream_id, stream_type) WHERE stream_id IS NOT NULL;
