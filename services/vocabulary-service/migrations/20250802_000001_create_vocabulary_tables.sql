-- Create vocabularies table
CREATE TABLE IF NOT EXISTS vocabularies (
    -- Primary key
    id VARCHAR(255) PRIMARY KEY,
    
    -- Word information
    word VARCHAR(255) NOT NULL,
    definition TEXT NOT NULL,
    example_sentence TEXT,
    
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    
    -- Metadata
    tags TEXT[] DEFAULT '{}',
    difficulty_level INTEGER CHECK (difficulty_level BETWEEN 1 AND 5),
    
    -- User association
    created_by VARCHAR(255) NOT NULL,
    
    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_vocabularies_word ON vocabularies(word);
CREATE INDEX idx_vocabularies_created_by ON vocabularies(created_by);
CREATE INDEX idx_vocabularies_status ON vocabularies(status);
CREATE INDEX idx_vocabularies_difficulty_level ON vocabularies(difficulty_level);
CREATE INDEX idx_vocabularies_created_at ON vocabularies(created_at);
CREATE INDEX idx_vocabularies_deleted_at ON vocabularies(deleted_at) WHERE deleted_at IS NOT NULL;

-- Add comments
COMMENT ON TABLE vocabularies IS 'Vocabulary aggregate root table';
COMMENT ON COLUMN vocabularies.id IS 'Vocabulary unique identifier';
COMMENT ON COLUMN vocabularies.word IS 'The word or phrase';
COMMENT ON COLUMN vocabularies.definition IS 'Definition of the word';
COMMENT ON COLUMN vocabularies.example_sentence IS 'Example sentence using the word';
COMMENT ON COLUMN vocabularies.status IS 'Vocabulary status: active, archived';
COMMENT ON COLUMN vocabularies.tags IS 'Array of tags for categorization';
COMMENT ON COLUMN vocabularies.difficulty_level IS 'Difficulty level from 1 (easiest) to 5 (hardest)';
COMMENT ON COLUMN vocabularies.created_by IS 'User ID who created this vocabulary';
COMMENT ON COLUMN vocabularies.version IS 'Version for optimistic locking';
COMMENT ON COLUMN vocabularies.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN vocabularies.updated_at IS 'Record last update timestamp';
COMMENT ON COLUMN vocabularies.deleted_at IS 'Soft delete timestamp';