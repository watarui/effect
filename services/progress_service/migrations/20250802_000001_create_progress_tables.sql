-- Create user_progress_summaries table (Read Model)
CREATE TABLE IF NOT EXISTS user_progress_summaries (
    -- Primary key
    id UUID PRIMARY KEY,

    -- User association
    user_id UUID NOT NULL UNIQUE,

    -- Overall statistics
    total_vocabularies_learned INTEGER NOT NULL DEFAULT 0,
    total_learning_time_minutes INTEGER NOT NULL DEFAULT 0,
    total_sessions_completed INTEGER NOT NULL DEFAULT 0,

    -- Current streak
    current_streak_days INTEGER NOT NULL DEFAULT 0,
    longest_streak_days INTEGER NOT NULL DEFAULT 0,
    last_activity_date DATE,

    -- Performance metrics
    average_accuracy DECIMAL(5, 2) NOT NULL DEFAULT 0.00,
    average_response_time_ms INTEGER NOT NULL DEFAULT 0,

    -- Level and achievements
    current_level INTEGER NOT NULL DEFAULT 1,
    experience_points INTEGER NOT NULL DEFAULT 0,
    achievement_count INTEGER NOT NULL DEFAULT 0,

    -- Learning goal progress
    learning_goal VARCHAR(50),
    goal_progress_percentage DECIMAL(5, 2) NOT NULL DEFAULT 0.00,

    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create daily_progress_snapshots table
CREATE TABLE IF NOT EXISTS daily_progress_snapshots (
    -- Primary key
    id UUID PRIMARY KEY,

    -- User and date
    user_id UUID NOT NULL,
    snapshot_date DATE NOT NULL,

    -- Daily metrics
    vocabularies_learned INTEGER NOT NULL DEFAULT 0,
    vocabularies_reviewed INTEGER NOT NULL DEFAULT 0,
    learning_time_minutes INTEGER NOT NULL DEFAULT 0,
    sessions_completed INTEGER NOT NULL DEFAULT 0,

    -- Performance
    accuracy DECIMAL(5, 2),
    average_response_time_ms INTEGER,

    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint for one snapshot per user per day
    CONSTRAINT unique_user_date UNIQUE (user_id, snapshot_date)
);

-- Create vocabulary_mastery_levels table
CREATE TABLE IF NOT EXISTS vocabulary_mastery_levels (
    -- Primary key
    id UUID PRIMARY KEY,

    -- Associations
    user_id UUID NOT NULL,
    vocabulary_id UUID NOT NULL,

    -- Mastery data
    mastery_level INTEGER NOT NULL DEFAULT 0 CHECK (
        mastery_level BETWEEN 0 AND 5
    ),
    total_reviews INTEGER NOT NULL DEFAULT 0,
    successful_reviews INTEGER NOT NULL DEFAULT 0,
    last_reviewed_at TIMESTAMPTZ,

    -- SM-2 current state
    current_easiness_factor DECIMAL(3, 2) NOT NULL DEFAULT 2.5,
    current_interval_days INTEGER NOT NULL DEFAULT 1,
    next_review_date DATE,

    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint
    CONSTRAINT unique_user_vocabulary UNIQUE (user_id, vocabulary_id)
);

-- Indexes for user_progress_summaries
CREATE INDEX idx_user_progress_summaries_user_id ON user_progress_summaries (
    user_id
);
CREATE INDEX idx_user_progress_summaries_current_level ON user_progress_summaries (
    current_level
);
CREATE INDEX idx_user_progress_summaries_last_activity_date ON user_progress_summaries (
    last_activity_date
);

-- Indexes for daily_progress_snapshots
CREATE INDEX idx_daily_progress_snapshots_user_id ON daily_progress_snapshots (
    user_id
);
CREATE INDEX idx_daily_progress_snapshots_snapshot_date ON daily_progress_snapshots (
    snapshot_date
);
CREATE INDEX idx_daily_progress_snapshots_user_date ON daily_progress_snapshots (
    user_id, snapshot_date
);

-- Indexes for vocabulary_mastery_levels
CREATE INDEX idx_vocabulary_mastery_levels_user_id ON vocabulary_mastery_levels (
    user_id
);
CREATE INDEX idx_vocabulary_mastery_levels_vocabulary_id ON vocabulary_mastery_levels (
    vocabulary_id
);
CREATE INDEX idx_vocabulary_mastery_levels_mastery_level ON vocabulary_mastery_levels (
    mastery_level
);
CREATE INDEX idx_vocabulary_mastery_levels_next_review_date ON vocabulary_mastery_levels (
    next_review_date
);

-- Add comments
COMMENT ON TABLE user_progress_summaries IS 'Aggregated user progress read model';
COMMENT ON COLUMN user_progress_summaries.id IS 'Summary unique identifier';
COMMENT ON COLUMN user_progress_summaries.user_id IS 'User ID (one-to-one relationship)';
COMMENT ON COLUMN user_progress_summaries.total_vocabularies_learned IS 'Total number of vocabularies learned';
COMMENT ON COLUMN user_progress_summaries.total_learning_time_minutes IS 'Total learning time in minutes';
COMMENT ON COLUMN user_progress_summaries.total_sessions_completed IS 'Total number of sessions completed';
COMMENT ON COLUMN user_progress_summaries.current_streak_days IS 'Current consecutive days of activity';
COMMENT ON COLUMN user_progress_summaries.longest_streak_days IS 'Longest streak achieved';
COMMENT ON COLUMN user_progress_summaries.last_activity_date IS 'Date of last learning activity';
COMMENT ON COLUMN user_progress_summaries.average_accuracy IS 'Overall accuracy percentage';
COMMENT ON COLUMN user_progress_summaries.average_response_time_ms IS 'Average response time in milliseconds';
COMMENT ON COLUMN user_progress_summaries.current_level IS 'Current user level';
COMMENT ON COLUMN user_progress_summaries.experience_points IS 'Total experience points earned';
COMMENT ON COLUMN user_progress_summaries.achievement_count IS 'Number of achievements unlocked';
COMMENT ON COLUMN user_progress_summaries.learning_goal IS 'Current learning goal';
COMMENT ON COLUMN user_progress_summaries.goal_progress_percentage IS 'Progress towards learning goal';
COMMENT ON COLUMN user_progress_summaries.version IS 'Version for optimistic locking';
COMMENT ON COLUMN user_progress_summaries.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN user_progress_summaries.updated_at IS 'Record last update timestamp';

COMMENT ON TABLE daily_progress_snapshots IS 'Daily progress snapshots for trend analysis';
COMMENT ON COLUMN daily_progress_snapshots.id IS 'Snapshot unique identifier';
COMMENT ON COLUMN daily_progress_snapshots.user_id IS 'User ID';
COMMENT ON COLUMN daily_progress_snapshots.snapshot_date IS 'Date of this snapshot';
COMMENT ON COLUMN daily_progress_snapshots.vocabularies_learned IS 'New vocabularies learned on this day';
COMMENT ON COLUMN daily_progress_snapshots.vocabularies_reviewed IS 'Vocabularies reviewed on this day';
COMMENT ON COLUMN daily_progress_snapshots.learning_time_minutes IS 'Time spent learning on this day';
COMMENT ON COLUMN daily_progress_snapshots.sessions_completed IS 'Sessions completed on this day';
COMMENT ON COLUMN daily_progress_snapshots.accuracy IS 'Accuracy percentage for this day';
COMMENT ON COLUMN daily_progress_snapshots.average_response_time_ms IS 'Average response time for this day';
COMMENT ON COLUMN daily_progress_snapshots.version IS 'Version for optimistic locking';
COMMENT ON COLUMN daily_progress_snapshots.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN daily_progress_snapshots.updated_at IS 'Record last update timestamp';

COMMENT ON TABLE vocabulary_mastery_levels IS 'Individual vocabulary mastery tracking';
COMMENT ON COLUMN vocabulary_mastery_levels.id IS 'Mastery record unique identifier';
COMMENT ON COLUMN vocabulary_mastery_levels.user_id IS 'User ID';
COMMENT ON COLUMN vocabulary_mastery_levels.vocabulary_id IS 'Vocabulary ID';
COMMENT ON COLUMN vocabulary_mastery_levels.mastery_level IS 'Mastery level (0-5)';
COMMENT ON COLUMN vocabulary_mastery_levels.total_reviews IS 'Total number of reviews';
COMMENT ON COLUMN vocabulary_mastery_levels.successful_reviews IS 'Number of successful reviews';
COMMENT ON COLUMN vocabulary_mastery_levels.last_reviewed_at IS 'Last review timestamp';
COMMENT ON COLUMN vocabulary_mastery_levels.current_easiness_factor IS 'Current SM-2 easiness factor';
COMMENT ON COLUMN vocabulary_mastery_levels.current_interval_days IS 'Current review interval in days';
COMMENT ON COLUMN vocabulary_mastery_levels.next_review_date IS 'Next scheduled review date';
COMMENT ON COLUMN vocabulary_mastery_levels.version IS 'Version for optimistic locking';
COMMENT ON COLUMN vocabulary_mastery_levels.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN vocabulary_mastery_levels.updated_at IS 'Record last update timestamp';
