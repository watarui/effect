-- Create algorithm_configurations table
CREATE TABLE IF NOT EXISTS algorithm_configurations (
    -- Primary key
    id UUID PRIMARY KEY,

    -- Configuration details
    algorithm_name VARCHAR(100) NOT NULL,
    configuration JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create learning_parameters table
CREATE TABLE IF NOT EXISTS learning_parameters (
    -- Primary key
    id UUID PRIMARY KEY,

    -- User association
    user_id UUID NOT NULL UNIQUE,

    -- SM-2 parameters
    initial_interval INTEGER NOT NULL DEFAULT 1,
    initial_easiness DECIMAL(3, 2) NOT NULL DEFAULT 2.5,
    min_easiness DECIMAL(3, 2) NOT NULL DEFAULT 1.3,

    -- Additional parameters
    review_threshold DECIMAL(3, 2) NOT NULL DEFAULT 0.8,
    difficulty_adjustment_factor DECIMAL(3, 2) NOT NULL DEFAULT 1.0,

    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_algorithm_configurations_algorithm_name ON algorithm_configurations (
    algorithm_name
);
CREATE INDEX idx_algorithm_configurations_is_active ON algorithm_configurations (
    is_active
);
CREATE INDEX idx_learning_parameters_user_id ON learning_parameters (user_id);

-- Add comments
COMMENT ON TABLE algorithm_configurations IS 'Algorithm configuration settings';
COMMENT ON COLUMN algorithm_configurations.id IS 'Configuration unique identifier';
COMMENT ON COLUMN algorithm_configurations.algorithm_name IS 'Name of the algorithm';
COMMENT ON COLUMN algorithm_configurations.configuration IS 'JSON configuration data';
COMMENT ON COLUMN algorithm_configurations.is_active IS 'Whether this configuration is active';
COMMENT ON COLUMN algorithm_configurations.version IS 'Version for optimistic locking';
COMMENT ON COLUMN algorithm_configurations.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN algorithm_configurations.updated_at IS 'Record last update timestamp';

COMMENT ON TABLE learning_parameters IS 'User-specific learning parameters';
COMMENT ON COLUMN learning_parameters.id IS 'Parameter set unique identifier';
COMMENT ON COLUMN learning_parameters.user_id IS 'User ID (one-to-one relationship)';
COMMENT ON COLUMN learning_parameters.initial_interval IS 'Initial interval for new items';
COMMENT ON COLUMN learning_parameters.initial_easiness IS 'Initial easiness factor';
COMMENT ON COLUMN learning_parameters.min_easiness IS 'Minimum easiness factor';
COMMENT ON COLUMN learning_parameters.review_threshold IS 'Threshold for triggering review';
COMMENT ON COLUMN learning_parameters.difficulty_adjustment_factor IS 'Factor for adjusting difficulty';
COMMENT ON COLUMN learning_parameters.version IS 'Version for optimistic locking';
COMMENT ON COLUMN learning_parameters.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN learning_parameters.updated_at IS 'Record last update timestamp';
