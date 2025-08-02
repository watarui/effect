-- Vocabulary Service Schema
-- 英単語辞書として最適化された正規化設計

-- 1. vocabulary_entries テーブル（見出し語）
CREATE TABLE vocabulary_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spelling VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT vocabulary_entries_spelling_key UNIQUE (spelling)
);

-- 2. vocabulary_items テーブル（語彙項目 = spelling + disambiguation）
CREATE TABLE vocabulary_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_id UUID NOT NULL REFERENCES vocabulary_entries(id),
    spelling VARCHAR(255) NOT NULL,
    disambiguation VARCHAR(255) NOT NULL, -- '(fruit)', '(company)', etc
    
    -- 発音情報
    pronunciation VARCHAR(500),           -- IPA表記
    phonetic_respelling VARCHAR(500),     -- 読みやすい表記 (e.g., "i-FEM-er-ul")
    audio_url VARCHAR(1000),              -- 音声ファイルURL
    
    -- 基本的な分類
    register VARCHAR(50),                 -- formal, informal, slang, etc (NULL = neutral)
    cefr_level VARCHAR(10),               -- A1, A2, B1, B2, C1, C2
    
    -- ステータスとメタ情報
    status VARCHAR(50) NOT NULL CHECK (status IN ('draft', 'pending_ai', 'published')),
    created_by_type VARCHAR(50) NOT NULL CHECK (created_by_type IN ('user', 'system', 'import')),
    created_by_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_modified_by UUID NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    deleted_at TIMESTAMPTZ,
    
    CONSTRAINT vocabulary_items_unique_disambiguation 
        UNIQUE (spelling, disambiguation) WHERE deleted_at IS NULL
);

-- 3. vocabulary_definitions テーブル（定義）
CREATE TABLE vocabulary_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES vocabulary_items(id) ON DELETE CASCADE,
    part_of_speech VARCHAR(50) NOT NULL,  -- noun, verb, adjective, etc
    meaning TEXT NOT NULL,                 -- 英語の定義
    meaning_translation TEXT,              -- 日本語訳
    domain VARCHAR(100),                   -- medical, computing, legal, etc (NULL = general)
    register VARCHAR(50),                  -- この定義特有の使用域（itemレベルを上書き）
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. vocabulary_examples テーブル（例文）
CREATE TABLE vocabulary_examples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    definition_id UUID NOT NULL REFERENCES vocabulary_definitions(id) ON DELETE CASCADE,
    example_text TEXT NOT NULL,
    example_translation TEXT,              -- 例文の日本語訳
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 5. vocabulary_synonyms テーブル（類義語）
CREATE TABLE vocabulary_synonyms (
    definition_id UUID NOT NULL REFERENCES vocabulary_definitions(id) ON DELETE CASCADE,
    synonym VARCHAR(255) NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (definition_id, synonym)
);

-- 6. vocabulary_antonyms テーブル（対義語）
CREATE TABLE vocabulary_antonyms (
    definition_id UUID NOT NULL REFERENCES vocabulary_definitions(id) ON DELETE CASCADE,
    antonym VARCHAR(255) NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (definition_id, antonym)
);

-- 7. vocabulary_collocations テーブル（コロケーション）
CREATE TABLE vocabulary_collocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    definition_id UUID NOT NULL REFERENCES vocabulary_definitions(id) ON DELETE CASCADE,
    collocation_type VARCHAR(50) NOT NULL, -- verb_noun, adjective_noun, etc
    pattern VARCHAR(255) NOT NULL,         -- 'make a decision', 'heavy rain'
    example TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- インデックス
CREATE INDEX idx_vocabulary_items_entry_id ON vocabulary_items(entry_id);
CREATE INDEX idx_vocabulary_items_status ON vocabulary_items(status);
CREATE INDEX idx_vocabulary_items_cefr_level ON vocabulary_items(cefr_level);
CREATE INDEX idx_vocabulary_items_register ON vocabulary_items(register);
CREATE INDEX idx_vocabulary_definitions_item_id ON vocabulary_definitions(item_id);
CREATE INDEX idx_vocabulary_definitions_part_of_speech ON vocabulary_definitions(part_of_speech);
CREATE INDEX idx_vocabulary_definitions_domain ON vocabulary_definitions(domain);
CREATE INDEX idx_vocabulary_examples_definition_id ON vocabulary_examples(definition_id);
CREATE INDEX idx_vocabulary_collocations_definition_id ON vocabulary_collocations(definition_id);

-- コメント
COMMENT ON TABLE vocabulary_entries IS '語彙エントリー（見出し語）';
COMMENT ON TABLE vocabulary_items IS '語彙項目（見出し語 + 曖昧性解消）';
COMMENT ON TABLE vocabulary_definitions IS '定義（品詞別の意味）';
COMMENT ON TABLE vocabulary_examples IS '例文';
COMMENT ON TABLE vocabulary_synonyms IS '類義語';
COMMENT ON TABLE vocabulary_antonyms IS '対義語';
COMMENT ON TABLE vocabulary_collocations IS 'コロケーション（連語）';