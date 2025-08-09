-- Progress Projection Service - State Management
-- イベント処理の位置追跡

CREATE TABLE IF NOT EXISTS projection_checkpoints (
    projection_name VARCHAR(100) PRIMARY KEY,
    last_processed_event_id UUID,
    last_processed_at TIMESTAMPTZ,
    position BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 失敗したイベントの記録
CREATE TABLE IF NOT EXISTS projection_failures (
    id UUID PRIMARY KEY,
    projection_name VARCHAR(100) NOT NULL,
    event_id UUID NOT NULL,
    error_message TEXT NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    failed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- インデックス
CREATE INDEX idx_projection_failures_projection ON projection_failures (projection_name);
CREATE INDEX idx_projection_failures_retry ON projection_failures (retry_count, failed_at);
