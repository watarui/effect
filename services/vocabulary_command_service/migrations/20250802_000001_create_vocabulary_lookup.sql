-- Vocabulary Command Service - Lookup Table
-- Event Store への書き込み前の重複チェック用

CREATE TABLE IF NOT EXISTS vocabulary_lookup (
    id UUID PRIMARY KEY,
    spelling VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- インデックス
CREATE INDEX idx_vocabulary_lookup_spelling ON vocabulary_lookup (spelling);
CREATE INDEX idx_vocabulary_lookup_status ON vocabulary_lookup (status);
