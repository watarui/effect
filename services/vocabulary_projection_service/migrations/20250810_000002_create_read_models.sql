-- Vocabulary Read Models
-- 読み取り専用の非正規化ビュー

-- VocabularyEntry の Read Model
CREATE TABLE IF NOT EXISTS vocabulary_entries_read (
    entry_id UUID PRIMARY KEY,
    spelling VARCHAR(255) NOT NULL,
    primary_item_id UUID,
    item_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    last_event_version BIGINT NOT NULL DEFAULT 0
);

-- VocabularyItem の Read Model
CREATE TABLE IF NOT EXISTS vocabulary_items_read (
    item_id UUID PRIMARY KEY,
    entry_id UUID NOT NULL,
    spelling VARCHAR(255) NOT NULL,
    disambiguation VARCHAR(255),
    part_of_speech VARCHAR(50),
    definition TEXT,
    ipa_pronunciation VARCHAR(255),
    cefr_level VARCHAR(10),
    frequency_rank INTEGER,
    is_published BOOLEAN NOT NULL DEFAULT FALSE,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    example_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    last_event_version BIGINT NOT NULL DEFAULT 0
);

-- 例文の Read Model
CREATE TABLE IF NOT EXISTS vocabulary_examples_read (
    example_id UUID PRIMARY KEY DEFAULT GEN_RANDOM_UUID(),
    item_id UUID NOT NULL,
    example TEXT NOT NULL,
    translation TEXT,
    added_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

-- インデックス
CREATE INDEX idx_vocabulary_entries_spelling ON vocabulary_entries_read (spelling);
CREATE INDEX idx_vocabulary_items_entry_id ON vocabulary_items_read (entry_id);
CREATE INDEX idx_vocabulary_items_spelling ON vocabulary_items_read (spelling);
CREATE INDEX idx_vocabulary_items_published ON vocabulary_items_read (is_published) WHERE NOT is_deleted;
CREATE INDEX idx_vocabulary_items_cefr ON vocabulary_items_read (cefr_level) WHERE NOT is_deleted;
CREATE INDEX idx_vocabulary_examples_item_id ON vocabulary_examples_read (item_id);

-- Note: 全文検索は Meilisearch で実装するため、PostgreSQL の tsvector は使用しない
