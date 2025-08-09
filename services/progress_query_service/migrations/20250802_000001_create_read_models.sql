-- Progress Query Service - Read Models
-- 読み取り専用のビューモデル

-- ユーザー進捗ビュー
CREATE TABLE IF NOT EXISTS user_progress_view (
    user_id UUID PRIMARY KEY,
    total_words_learned INTEGER NOT NULL DEFAULT 0,
    total_reviews_completed INTEGER NOT NULL DEFAULT 0,
    current_streak_days INTEGER NOT NULL DEFAULT 0,
    longest_streak_days INTEGER NOT NULL DEFAULT 0,
    last_activity_at TIMESTAMPTZ,
    level INTEGER NOT NULL DEFAULT 1,
    experience_points INTEGER NOT NULL DEFAULT 0,
    achievements JSONB DEFAULT '[]'::JSONB,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 単語別進捗ビュー
CREATE TABLE IF NOT EXISTS word_progress_view (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    word_id UUID NOT NULL,
    learning_status VARCHAR(50) NOT NULL, -- new, learning, review, mastered
    repetition_count INTEGER NOT NULL DEFAULT 0,
    success_rate DECIMAL(5, 2),
    last_reviewed_at TIMESTAMPTZ,
    next_review_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, word_id)
);

-- デイリー進捗ビュー
CREATE TABLE IF NOT EXISTS daily_progress_view (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    date DATE NOT NULL,
    words_learned INTEGER NOT NULL DEFAULT 0,
    reviews_completed INTEGER NOT NULL DEFAULT 0,
    minutes_studied INTEGER NOT NULL DEFAULT 0,
    accuracy_rate DECIMAL(5, 2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, date)
);

-- インデックス
CREATE INDEX idx_word_progress_user_id ON word_progress_view (user_id);
CREATE INDEX idx_word_progress_status ON word_progress_view (learning_status);
CREATE INDEX idx_word_progress_next_review ON word_progress_view (next_review_at);
CREATE INDEX idx_daily_progress_user_date ON daily_progress_view (user_id, date DESC);
