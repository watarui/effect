-- Create learning_sessions table
CREATE TABLE IF NOT EXISTS learning_sessions (
    -- Primary key
    id UUID PRIMARY KEY,

    -- User association
    user_id UUID NOT NULL,

    -- Session information
    session_type VARCHAR(50) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,

    -- Progress tracking
    total_items INTEGER NOT NULL DEFAULT 0,
    completed_items INTEGER NOT NULL DEFAULT 0,
    correct_items INTEGER NOT NULL DEFAULT 0,

    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create learning_records table
CREATE TABLE IF NOT EXISTS learning_records (
    -- Primary key
    id UUID PRIMARY KEY,

    -- Associations
    user_id UUID NOT NULL,
    vocabulary_id UUID NOT NULL,
    session_id UUID,

    -- Learning data
    response_quality INTEGER CHECK (response_quality BETWEEN 0 AND 5),
    response_time_ms INTEGER,
    is_correct BOOLEAN,

    -- SM-2 algorithm data
    repetition_count INTEGER NOT NULL DEFAULT 0,
    easiness_factor DECIMAL(3, 2) NOT NULL DEFAULT 2.5,
    interval_days INTEGER NOT NULL DEFAULT 1,
    next_review_date DATE NOT NULL DEFAULT CURRENT_DATE + INTERVAL '1 day',

    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for learning_sessions
CREATE INDEX idx_learning_sessions_user_id ON learning_sessions (user_id);
CREATE INDEX idx_learning_sessions_started_at ON learning_sessions (started_at);
CREATE INDEX idx_learning_sessions_session_type ON learning_sessions (
    session_type
);

-- Indexes for learning_records
CREATE INDEX idx_learning_records_user_id ON learning_records (user_id);
CREATE INDEX idx_learning_records_vocabulary_id ON learning_records (
    vocabulary_id
);
CREATE INDEX idx_learning_records_session_id ON learning_records (session_id);
CREATE INDEX idx_learning_records_next_review_date ON learning_records (
    next_review_date
);
CREATE UNIQUE INDEX idx_learning_records_user_vocabulary ON learning_records (
    user_id, vocabulary_id
);

-- Add comments for learning_sessions
COMMENT ON TABLE learning_sessions IS 'Learning session aggregate root table';
COMMENT ON COLUMN learning_sessions.id IS 'Session unique identifier';
COMMENT ON COLUMN learning_sessions.user_id IS 'User who owns this session';
COMMENT ON COLUMN learning_sessions.session_type IS 'Type of session: study, review, test';
COMMENT ON COLUMN learning_sessions.started_at IS 'Session start timestamp';
COMMENT ON COLUMN learning_sessions.ended_at IS 'Session end timestamp';
COMMENT ON COLUMN learning_sessions.total_items IS 'Total number of items in session';
COMMENT ON COLUMN learning_sessions.completed_items IS 'Number of completed items';
COMMENT ON COLUMN learning_sessions.correct_items IS 'Number of correct answers';
COMMENT ON COLUMN learning_sessions.version IS 'Version for optimistic locking';
COMMENT ON COLUMN learning_sessions.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN learning_sessions.updated_at IS 'Record last update timestamp';

-- Add comments for learning_records
COMMENT ON TABLE learning_records IS 'Individual learning record for each vocabulary item';
COMMENT ON COLUMN learning_records.id IS 'Record unique identifier';
COMMENT ON COLUMN learning_records.user_id IS 'User who is learning';
COMMENT ON COLUMN learning_records.vocabulary_id IS 'Vocabulary being learned';
COMMENT ON COLUMN learning_records.session_id IS 'Optional session this record belongs to';
COMMENT ON COLUMN learning_records.response_quality IS 'Quality of response (0-5 for SM-2)';
COMMENT ON COLUMN learning_records.response_time_ms IS 'Time taken to respond in milliseconds';
COMMENT ON COLUMN learning_records.is_correct IS 'Whether the response was correct';
COMMENT ON COLUMN learning_records.repetition_count IS 'Number of successful repetitions';
COMMENT ON COLUMN learning_records.easiness_factor IS 'SM-2 easiness factor';
COMMENT ON COLUMN learning_records.interval_days IS 'Days until next review';
COMMENT ON COLUMN learning_records.next_review_date IS 'Date of next review';
COMMENT ON COLUMN learning_records.version IS 'Version for optimistic locking';
COMMENT ON COLUMN learning_records.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN learning_records.updated_at IS 'Record last update timestamp';
