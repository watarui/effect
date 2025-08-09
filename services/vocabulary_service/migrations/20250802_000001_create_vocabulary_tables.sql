-- Vocabulary Service Tables (Placeholder)
-- 実装時に適切なテーブル構造を定義予定

CREATE TABLE IF NOT EXISTS vocabulary_items (
    id UUID PRIMARY KEY,
    spelling VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- インデックス
CREATE INDEX idx_vocabulary_items_spelling ON vocabulary_items (spelling);
CREATE INDEX idx_vocabulary_items_created_at ON vocabulary_items (created_at);
