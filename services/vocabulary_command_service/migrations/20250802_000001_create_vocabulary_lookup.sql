-- vocabulary_command_service 用の読み取り補助テーブル
-- Event Store からの集約検索を効率化するため

CREATE TABLE IF NOT EXISTS vocabulary_entries (
    entry_id VARCHAR(36) PRIMARY KEY,
    word VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT vocabulary_entries_word_unique UNIQUE (word)
);

CREATE INDEX idx_vocabulary_entries_word ON vocabulary_entries (word);

COMMENT ON TABLE vocabulary_entries IS 'Event Store の集約を効率的に検索するための補助テーブル';
COMMENT ON COLUMN vocabulary_entries.entry_id IS '集約ID（UUID）';
COMMENT ON COLUMN vocabulary_entries.word IS '単語（検索用）';
