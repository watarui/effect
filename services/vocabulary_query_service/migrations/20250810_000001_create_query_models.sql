-- Vocabulary Query Service - Read Models
-- 非正規化された読み取り専用ビュー
-- 完全に独立したデータベースで動作

-- エントリー（見出し語）テーブル
CREATE TABLE IF NOT EXISTS vocabulary_entries_read (
    entry_id UUID PRIMARY KEY,
    spelling VARCHAR(255) NOT NULL,
    primary_item_id UUID,
    item_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 語彙項目テーブル
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 例文テーブル
CREATE TABLE IF NOT EXISTS vocabulary_examples_read (
    example_id UUID PRIMARY KEY,
    item_id UUID NOT NULL,
    example TEXT NOT NULL,
    translation TEXT,
    added_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- インデックス
-- エントリーテーブル
CREATE INDEX idx_entries_spelling ON vocabulary_entries_read (spelling);
CREATE INDEX idx_entries_created_at ON vocabulary_entries_read (created_at DESC);

-- 語彙項目テーブル
CREATE INDEX idx_items_entry_id ON vocabulary_items_read (entry_id);
CREATE INDEX idx_items_spelling ON vocabulary_items_read (spelling);
CREATE INDEX idx_items_part_of_speech ON vocabulary_items_read (part_of_speech);
CREATE INDEX idx_items_cefr_level ON vocabulary_items_read (cefr_level);
CREATE INDEX idx_items_frequency_rank ON vocabulary_items_read (frequency_rank DESC NULLS LAST);
CREATE INDEX idx_items_is_published ON vocabulary_items_read (is_published);
CREATE INDEX idx_items_is_deleted ON vocabulary_items_read (is_deleted);
CREATE INDEX idx_items_created_at ON vocabulary_items_read (created_at DESC);
CREATE INDEX idx_items_disambiguation ON vocabulary_items_read (disambiguation) WHERE disambiguation IS NOT NULL;

-- 例文テーブル
CREATE INDEX idx_examples_item_id ON vocabulary_examples_read (item_id);
CREATE INDEX idx_examples_created_at ON vocabulary_examples_read (created_at DESC);

-- 複合インデックス（検索用）
CREATE INDEX idx_items_spelling_definition ON vocabulary_items_read (spelling, definition);
CREATE INDEX idx_items_published_not_deleted ON vocabulary_items_read (is_published, is_deleted) WHERE is_published
= TRUE
AND is_deleted = FALSE;

-- Note: 全文検索は Meilisearch で行うため、tsvector は使用しない
-- Note: updated_at はアプリケーション側で管理（トリガー不使用）
