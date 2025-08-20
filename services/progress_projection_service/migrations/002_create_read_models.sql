-- Progress Read Model Tables

-- プロジェクション状態管理
CREATE TABLE IF NOT EXISTS projection_states (
    projection_name VARCHAR(100) PRIMARY KEY,
    last_position BIGINT NOT NULL DEFAULT 0,
    last_event_id UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ユーザー進捗
CREATE TABLE IF NOT EXISTS user_progress (
    user_id UUID PRIMARY KEY,
    total_items_learned INTEGER NOT NULL DEFAULT 0,
    total_items_mastered INTEGER NOT NULL DEFAULT 0,
    total_study_minutes INTEGER NOT NULL DEFAULT 0,
    current_streak_days INTEGER NOT NULL DEFAULT 0,
    longest_streak_days INTEGER NOT NULL DEFAULT 0,
    last_study_date TIMESTAMPTZ,
    achievements_unlocked TEXT [] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 日次進捗
CREATE TABLE IF NOT EXISTS daily_progress (
    user_id UUID NOT NULL,
    date DATE NOT NULL,
    items_learned INTEGER NOT NULL DEFAULT 0,
    items_reviewed INTEGER NOT NULL DEFAULT 0,
    items_mastered INTEGER NOT NULL DEFAULT 0,
    correct_answers INTEGER NOT NULL DEFAULT 0,
    total_answers INTEGER NOT NULL DEFAULT 0,
    study_minutes INTEGER NOT NULL DEFAULT 0,
    sessions_count INTEGER NOT NULL DEFAULT 0,
    goal_completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, date)
);

-- 週次進捗
CREATE TABLE IF NOT EXISTS weekly_progress (
    user_id UUID NOT NULL,
    week_start_date DATE NOT NULL,
    week_end_date DATE NOT NULL,
    items_learned INTEGER NOT NULL DEFAULT 0,
    items_reviewed INTEGER NOT NULL DEFAULT 0,
    items_mastered INTEGER NOT NULL DEFAULT 0,
    study_minutes INTEGER NOT NULL DEFAULT 0,
    study_days INTEGER NOT NULL DEFAULT 0,
    goals_completed INTEGER NOT NULL DEFAULT 0,
    average_accuracy REAL NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, week_start_date)
);

-- 語彙アイテム進捗
CREATE TABLE IF NOT EXISTS vocabulary_item_progress (
    user_id UUID NOT NULL,
    vocabulary_item_id UUID NOT NULL,
    attempts_count INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    last_attempt_date TIMESTAMPTZ NOT NULL,
    last_accuracy REAL NOT NULL DEFAULT 0,
    average_accuracy REAL NOT NULL DEFAULT 0,
    mastery_level INTEGER NOT NULL DEFAULT 0,
    time_spent_seconds INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, vocabulary_item_id)
);

-- アチーブメント
CREATE TABLE IF NOT EXISTS achievements (
    user_id UUID NOT NULL,
    achievement_id VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    category VARCHAR(50) NOT NULL,
    unlocked_at TIMESTAMPTZ NOT NULL,
    progress INTEGER NOT NULL DEFAULT 0,
    target INTEGER NOT NULL DEFAULT 100,
    PRIMARY KEY (user_id, achievement_id)
);

-- インデックス
CREATE INDEX idx_user_progress_updated ON user_progress (updated_at);
CREATE INDEX idx_daily_progress_user_date ON daily_progress (user_id, date DESC);
CREATE INDEX idx_weekly_progress_user_week ON weekly_progress (user_id, week_start_date DESC);
CREATE INDEX idx_vocabulary_item_progress_user ON vocabulary_item_progress (user_id);
CREATE INDEX idx_achievements_user ON achievements (user_id, unlocked_at DESC);
