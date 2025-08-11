-- Create learning_item_states table
-- This table stores the learning state for each user-item combination
CREATE TABLE IF NOT EXISTS learning_item_states (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- User and item association
    user_id UUID NOT NULL,
    item_id UUID NOT NULL,

    -- SM-2 Algorithm parameters
    easiness_factor REAL NOT NULL DEFAULT 2.5,
    repetition_number INTEGER NOT NULL DEFAULT 0,
    interval_days INTEGER NOT NULL DEFAULT 1,

    -- Mastery information
    mastery_level INTEGER NOT NULL DEFAULT 1, -- 1=Beginner, 2=Learning, 3=Familiar, 4=Proficient, 5=Mastered
    retention_rate REAL NOT NULL DEFAULT 0.0,

    -- Scheduling information
    next_review_date TIMESTAMPTZ,
    last_reviewed_at TIMESTAMPTZ,

    -- Statistics
    total_reviews INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    incorrect_count INTEGER NOT NULL DEFAULT 0,
    average_response_time_ms REAL NOT NULL DEFAULT 0.0,

    -- Item metadata (cached for performance)
    difficulty_level INTEGER NOT NULL DEFAULT 1, -- CEFR level
    is_problematic BOOLEAN NOT NULL DEFAULT FALSE,

    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Unique constraint
    UNIQUE (user_id, item_id)
);

-- Create review_histories table
-- This table stores the history of all reviews
CREATE TABLE IF NOT EXISTS review_histories (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- User and item association
    user_id UUID NOT NULL,
    item_id UUID NOT NULL,

    -- Review information
    reviewed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    judgment INTEGER NOT NULL, -- 0=Unspecified, 1=Incorrect, 2=PartiallyCorrect, 3=Correct, 4=Perfect
    response_time_ms INTEGER NOT NULL,
    interval_days INTEGER NOT NULL,
    easiness_factor REAL NOT NULL,

    -- Session information
    session_id UUID,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Create learning_strategies table
-- This table stores user-specific learning strategies
CREATE TABLE IF NOT EXISTS learning_strategies (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- User association
    user_id UUID NOT NULL UNIQUE,

    -- Strategy settings
    strategy_type INTEGER NOT NULL DEFAULT 2, -- 1=Aggressive, 2=Balanced, 3=Conservative, 4=Custom
    daily_target_items INTEGER NOT NULL DEFAULT 20,
    new_items_per_day INTEGER NOT NULL DEFAULT 5,
    difficulty_threshold REAL NOT NULL DEFAULT 0.7,

    -- Adaptive parameters
    learning_speed_factor REAL NOT NULL DEFAULT 1.0,
    retention_priority REAL NOT NULL DEFAULT 0.8,
    adaptive_scheduling BOOLEAN NOT NULL DEFAULT TRUE,

    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,

    -- Timestamps
    last_adjusted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Create user_learning_statistics table (materialized view)
-- This table stores aggregated statistics for performance
CREATE TABLE IF NOT EXISTS user_learning_statistics (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- User association
    user_id UUID NOT NULL UNIQUE,

    -- Item counts
    total_items INTEGER NOT NULL DEFAULT 0,
    mastered_items INTEGER NOT NULL DEFAULT 0,
    learning_items INTEGER NOT NULL DEFAULT 0,
    new_items INTEGER NOT NULL DEFAULT 0,

    -- Session statistics
    total_sessions INTEGER NOT NULL DEFAULT 0,
    total_reviews INTEGER NOT NULL DEFAULT 0,
    overall_accuracy REAL NOT NULL DEFAULT 0.0,
    average_session_duration_seconds INTEGER NOT NULL DEFAULT 0,

    -- Progress statistics
    daily_review_average REAL NOT NULL DEFAULT 0.0,
    current_streak_days INTEGER NOT NULL DEFAULT 0,
    longest_streak_days INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    last_calculated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Create performance_analyses table
-- This table stores performance analysis results
CREATE TABLE IF NOT EXISTS performance_analyses (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- User association
    user_id UUID NOT NULL,

    -- Analysis period
    analyzed_from TIMESTAMPTZ NOT NULL,
    analyzed_to TIMESTAMPTZ NOT NULL,

    -- Trend analysis
    accuracy_trend REAL NOT NULL DEFAULT 0.0,
    speed_trend REAL NOT NULL DEFAULT 0.0,
    retention_trend REAL NOT NULL DEFAULT 0.0,

    -- Problem areas (stored as JSON array)
    problematic_categories JSONB NOT NULL DEFAULT '[]',
    strong_categories JSONB NOT NULL DEFAULT '[]',

    -- Learning patterns
    active_hours JSONB NOT NULL DEFAULT '[]', -- Array of hours (0-23)
    consistency_score REAL NOT NULL DEFAULT 0.0,

    -- Predictions
    predicted_mastery_days INTEGER,
    burnout_risk REAL NOT NULL DEFAULT 0.0,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for learning_item_states
CREATE INDEX idx_learning_item_states_user_id ON learning_item_states (user_id);
CREATE INDEX idx_learning_item_states_item_id ON learning_item_states (item_id);
CREATE INDEX idx_learning_item_states_next_review ON learning_item_states (user_id, next_review_date);
CREATE INDEX idx_learning_item_states_mastery ON learning_item_states (user_id, mastery_level);

-- Indexes for review_histories
CREATE INDEX idx_review_histories_user_item ON review_histories (user_id, item_id);
CREATE INDEX idx_review_histories_reviewed_at ON review_histories (reviewed_at);
CREATE INDEX idx_review_histories_session ON review_histories (session_id);

-- Indexes for learning_strategies
CREATE INDEX idx_learning_strategies_user_id ON learning_strategies (user_id);

-- Indexes for user_learning_statistics
CREATE INDEX idx_user_learning_statistics_user_id ON user_learning_statistics (user_id);

-- Indexes for performance_analyses
CREATE INDEX idx_performance_analyses_user_id ON performance_analyses (user_id);
CREATE INDEX idx_performance_analyses_period ON performance_analyses (user_id, analyzed_from, analyzed_to);

-- Add comments
COMMENT ON TABLE learning_item_states IS 'Learning state for each user-item combination using SM-2 algorithm';
COMMENT ON TABLE review_histories IS 'Historical record of all review sessions';
COMMENT ON TABLE learning_strategies IS 'User-specific learning strategy configurations';
COMMENT ON TABLE user_learning_statistics IS 'Aggregated learning statistics per user';
COMMENT ON TABLE performance_analyses IS 'Performance analysis results and predictions';
