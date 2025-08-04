-- vocabulary_projection_service の Read Model テーブル

-- 語彙項目の Read Model（非正規化されたビュー）
CREATE TABLE IF NOT EXISTS vocabulary_items_view (
    -- 基本情報
    item_id UUID PRIMARY KEY,
    entry_id UUID NOT NULL,
    spelling VARCHAR(255) NOT NULL,
    disambiguation VARCHAR(255) NOT NULL,

    -- 発音情報（非正規化）
    pronunciation VARCHAR(500),
    phonetic_respelling VARCHAR(500),
    audio_url VARCHAR(1000),

    -- 分類情報
    register VARCHAR(50),
    cefr_level VARCHAR(10),

    -- 集約データ（JSON形式で柔軟に）
    definitions JSONB NOT NULL, -- [{id, part_of_speech, meaning, meaning_translation, domain, register, examples: [...]}]
    synonyms JSONB,             -- {definition_id: [synonyms...]}
    antonyms JSONB,             -- {definition_id: [antonyms...]}
    collocations JSONB,         -- [{definition_id, type, pattern, example}]

    -- 統計情報
    definition_count INTEGER NOT NULL DEFAULT 0,
    example_count INTEGER NOT NULL DEFAULT 0,
    quality_score REAL,

    -- メタデータ
    status VARCHAR(50) NOT NULL,
    created_by_type VARCHAR(50) NOT NULL,
    created_by_id UUID,
    created_at TIMESTAMPTZ NOT NULL,
    last_modified_at TIMESTAMPTZ NOT NULL,
    last_modified_by UUID NOT NULL,
    version INTEGER NOT NULL DEFAULT 1
);

-- インデックス
CREATE INDEX idx_vocabulary_items_view_spelling ON vocabulary_items_view (spelling);
CREATE INDEX idx_vocabulary_items_view_entry_id ON vocabulary_items_view (entry_id);
CREATE INDEX idx_vocabulary_items_view_status ON vocabulary_items_view (status);
CREATE INDEX idx_vocabulary_items_view_cefr_level ON vocabulary_items_view (cefr_level);

-- プロジェクション状態管理テーブル
CREATE TABLE IF NOT EXISTS projection_state (
    projection_name VARCHAR(255) PRIMARY KEY,
    last_processed_event_id UUID,
    last_processed_timestamp TIMESTAMPTZ,
    event_store_position BIGINT,
    status VARCHAR(50) NOT NULL DEFAULT 'running',
    error_count INTEGER DEFAULT 0,
    last_error TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- projection_state の初期データ
INSERT INTO projection_state (projection_name, status, updated_at)
VALUES ('vocabulary_items_view', 'initialized', NOW())
ON CONFLICT (projection_name) DO NOTHING;
