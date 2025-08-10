-- Vocabulary Projection Service - Projection State Management
-- イベントストリームの処理状態を管理
-- 完全に独立したデータベースで動作

CREATE TABLE IF NOT EXISTS projection_state (
    projection_name VARCHAR(255) PRIMARY KEY,
    last_processed_position BIGINT NOT NULL DEFAULT 0,  -- 最後に処理したイベントのグローバルポジション
    last_processed_event_id UUID,  -- 最後に処理したイベントのID
    last_processed_at TIMESTAMPTZ,
    error_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    last_error_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -- Note: updated_at はアプリケーション側で管理（Event Sourcing パターンに従う）
);

-- チェックポイント履歴（デバッグ・監査用）
CREATE TABLE IF NOT EXISTS projection_checkpoints (
    id UUID PRIMARY KEY DEFAULT GEN_RANDOM_UUID(),
    projection_name VARCHAR(255) NOT NULL REFERENCES projection_state (projection_name),
    position BIGINT NOT NULL,
    event_id UUID,
    events_processed INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- インデックス
CREATE INDEX idx_projection_checkpoints_name ON projection_checkpoints (projection_name);
CREATE INDEX idx_projection_checkpoints_created ON projection_checkpoints (created_at);

-- Note: トリガーは使用せず、アプリケーション側でタイムスタンプを管理
