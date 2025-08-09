-- Vocabulary Query Service - Read Models
-- 非正規化された読み取り専用ビュー

CREATE TABLE IF NOT EXISTS vocabulary_read_models (
    id UUID PRIMARY KEY,
    spelling VARCHAR(255) NOT NULL,
    data JSONB NOT NULL, -- 非正規化データ（全情報を含む）
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- インデックス（高速検索用）
CREATE INDEX idx_vocabulary_read_spelling ON vocabulary_read_models (spelling);
CREATE INDEX idx_vocabulary_read_status ON vocabulary_read_models (status);
CREATE INDEX idx_vocabulary_read_data ON vocabulary_read_models USING gin (data);
