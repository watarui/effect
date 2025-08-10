-- スキーマレジストリテーブル
CREATE TABLE IF NOT EXISTS event_schemas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(255) NOT NULL,
    version INT NOT NULL,
    definition TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- 同じイベントタイプの同じバージョンは一意
    UNIQUE (event_type, version)
);

-- インデックス
CREATE INDEX idx_event_schemas_event_type ON event_schemas (event_type);
CREATE INDEX idx_event_schemas_version ON event_schemas (event_type, version DESC);

-- スキーマメタデータテーブル（オプション）
CREATE TABLE IF NOT EXISTS schema_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schema_id UUID NOT NULL REFERENCES event_schemas (id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (schema_id, key)
);

-- スキーマ互換性テーブル（将来の拡張用）
CREATE TABLE IF NOT EXISTS schema_compatibility (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_schema_id UUID NOT NULL REFERENCES event_schemas (id),
    to_schema_id UUID NOT NULL REFERENCES event_schemas (id),
    compatibility_type VARCHAR(50) NOT NULL, -- 'backward', 'forward', 'full'
    migration_script TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (from_schema_id, to_schema_id)
);

-- 初期スキーマデータの挿入
INSERT INTO event_schemas (event_type, version, definition, description) VALUES
('vocabulary.EntryCreated', 1, '{"type":"object"}', 'Vocabulary entry created event'),
('vocabulary.ItemCreated', 1, '{"type":"object"}', 'Vocabulary item created event'),
('vocabulary.FieldUpdated', 1, '{"type":"object"}', 'Vocabulary field updated event'),
('learning.SessionStarted', 1, '{"type":"object"}', 'Learning session started event'),
('learning.SessionCompleted', 1, '{"type":"object"}', 'Learning session completed event'),
('user.UserSignedUp', 1, '{"type":"object"}', 'User signed up event'),
('user.ProfileUpdated', 1, '{"type":"object"}', 'User profile updated event')
ON CONFLICT (event_type, version) DO NOTHING;
