-- Vocabulary Query Service - Read Models
-- 非正規化された読み取り専用ビュー
-- 完全に独立したデータベースで動作

CREATE TABLE IF NOT EXISTS vocabulary_read_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_id UUID NOT NULL,
    item_id UUID NOT NULL,
    spelling VARCHAR(255) NOT NULL,
    disambiguation VARCHAR(255),

    -- 詳細データ（JSONB）
    -- 含まれる内容:
    -- - pronunciation: 発音記号
    -- - phonetic_respelling: 音声表記
    -- - definitions: 定義の配列
    -- - synonyms: 類義語
    -- - antonyms: 対義語
    -- - collocations: コロケーション
    -- - register: レジスター（formal, informal など）
    -- - cefr_level: CEFR レベル
    -- - examples: 例文の配列
    data JSONB NOT NULL,

    status VARCHAR(50) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived', 'deleted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- 一意制約
    CONSTRAINT unique_entry_item UNIQUE (entry_id, item_id)
);

-- インデックス（シンプルなフィルタリング用のみ）
CREATE INDEX idx_vocabulary_read_spelling ON vocabulary_read_models (spelling);
CREATE INDEX idx_vocabulary_read_status ON vocabulary_read_models (status);
CREATE INDEX idx_vocabulary_read_entry_item ON vocabulary_read_models (entry_id, item_id);
CREATE INDEX idx_vocabulary_read_disambiguation ON vocabulary_read_models (
    disambiguation
) WHERE disambiguation IS NOT NULL;

-- JSONB インデックス（特定フィールドの検索用）
CREATE INDEX idx_vocabulary_read_cefr ON vocabulary_read_models ((data ->> 'cefr_level'));
CREATE INDEX idx_vocabulary_read_register ON vocabulary_read_models ((data ->> 'register'));

-- Note: 全文検索は Meilisearch で行うため、tsvector は使用しない
-- Note: updated_at はアプリケーション側で管理（トリガー不使用）
