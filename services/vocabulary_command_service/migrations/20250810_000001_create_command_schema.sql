-- Vocabulary Command Service - Domain Schema
-- Event Store への書き込み前の集約管理用
-- 完全に独立したデータベースで動作

-- VocabularyEntry 集約（見出し語）
CREATE TABLE IF NOT EXISTS vocabulary_entries (
    entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spelling VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version BIGINT NOT NULL DEFAULT 1
);

-- VocabularyItem 集約（語彙項目）
CREATE TABLE IF NOT EXISTS vocabulary_items (
    item_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_id UUID NOT NULL REFERENCES vocabulary_entries (entry_id) ON DELETE CASCADE,
    spelling VARCHAR(255) NOT NULL,
    disambiguation VARCHAR(255),  -- 意味の区別（例: "(fruit)", "(company)"）
    is_primary BOOLEAN DEFAULT FALSE,  -- 最も一般的な意味かどうか
    status VARCHAR(50) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'pending_ai', 'published')),
    is_deleted BOOLEAN DEFAULT FALSE,  -- ソフトデリート用
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version BIGINT NOT NULL DEFAULT 1,
    CONSTRAINT unique_entry_disambiguation UNIQUE (entry_id, disambiguation)
);

-- インデックス
CREATE INDEX idx_vocabulary_entries_spelling ON vocabulary_entries (spelling);
CREATE INDEX idx_vocabulary_entries_created_at ON vocabulary_entries (created_at);

CREATE INDEX idx_vocabulary_items_entry ON vocabulary_items (entry_id);
CREATE INDEX idx_vocabulary_items_status ON vocabulary_items (status);
CREATE INDEX idx_vocabulary_items_spelling ON vocabulary_items (spelling);
CREATE INDEX idx_vocabulary_items_created_at ON vocabulary_items (created_at);

-- 主要項目を1つだけに制約
CREATE UNIQUE INDEX idx_vocabulary_items_primary ON vocabulary_items (entry_id) WHERE is_primary = TRUE;
