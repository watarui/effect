-- Saga instances table
CREATE TABLE IF NOT EXISTS saga_instances (
    -- Saga identification
    saga_id VARCHAR(255) PRIMARY KEY,
    saga_type VARCHAR(255) NOT NULL,
    
    -- State management
    current_state VARCHAR(100) NOT NULL,
    saga_data JSONB NOT NULL DEFAULT '{}',
    
    -- Status tracking
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    
    -- Correlation
    correlation_id VARCHAR(255) NOT NULL,
    
    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Saga steps table
CREATE TABLE IF NOT EXISTS saga_steps (
    -- Step identification
    step_id VARCHAR(255) PRIMARY KEY,
    saga_id VARCHAR(255) NOT NULL REFERENCES saga_instances(saga_id),
    
    -- Step details
    step_name VARCHAR(255) NOT NULL,
    step_sequence INTEGER NOT NULL,
    
    -- Status and execution
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    
    -- Compensation
    compensation_required BOOLEAN NOT NULL DEFAULT false,
    compensation_status VARCHAR(50),
    compensated_at TIMESTAMPTZ,
    
    -- Command and response
    command_data JSONB NOT NULL,
    response_data JSONB,
    error_data JSONB,
    
    -- Retry information
    retry_count INTEGER NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Saga timeouts table
CREATE TABLE IF NOT EXISTS saga_timeouts (
    -- Timeout identification
    timeout_id VARCHAR(255) PRIMARY KEY,
    saga_id VARCHAR(255) NOT NULL REFERENCES saga_instances(saga_id),
    
    -- Timeout details
    timeout_type VARCHAR(100) NOT NULL,
    timeout_at TIMESTAMPTZ NOT NULL,
    
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    processed_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for saga_instances
CREATE INDEX idx_saga_instances_saga_type ON saga_instances(saga_type);
CREATE INDEX idx_saga_instances_status ON saga_instances(status);
CREATE INDEX idx_saga_instances_correlation_id ON saga_instances(correlation_id);
CREATE INDEX idx_saga_instances_created_at ON saga_instances(created_at);

-- Indexes for saga_steps
CREATE INDEX idx_saga_steps_saga_id ON saga_steps(saga_id);
CREATE INDEX idx_saga_steps_status ON saga_steps(status);
CREATE INDEX idx_saga_steps_step_sequence ON saga_steps(saga_id, step_sequence);

-- Indexes for saga_timeouts
CREATE INDEX idx_saga_timeouts_saga_id ON saga_timeouts(saga_id);
CREATE INDEX idx_saga_timeouts_timeout_at ON saga_timeouts(timeout_at) WHERE status = 'pending';
CREATE INDEX idx_saga_timeouts_status ON saga_timeouts(status);

-- Add comments for saga_instances
COMMENT ON TABLE saga_instances IS 'Saga orchestration instances';
COMMENT ON COLUMN saga_instances.saga_id IS 'Unique saga instance identifier';
COMMENT ON COLUMN saga_instances.saga_type IS 'Type of saga';
COMMENT ON COLUMN saga_instances.current_state IS 'Current state in the saga state machine';
COMMENT ON COLUMN saga_instances.saga_data IS 'Saga instance data';
COMMENT ON COLUMN saga_instances.status IS 'Status: active, completed, failed, compensating';
COMMENT ON COLUMN saga_instances.error_message IS 'Error message if failed';
COMMENT ON COLUMN saga_instances.retry_count IS 'Number of retry attempts';
COMMENT ON COLUMN saga_instances.correlation_id IS 'Correlation ID for tracing';
COMMENT ON COLUMN saga_instances.version IS 'Version for optimistic locking';
COMMENT ON COLUMN saga_instances.created_at IS 'Saga start timestamp';
COMMENT ON COLUMN saga_instances.updated_at IS 'Last update timestamp';
COMMENT ON COLUMN saga_instances.completed_at IS 'Saga completion timestamp';

-- Add comments for saga_steps
COMMENT ON TABLE saga_steps IS 'Individual steps within a saga';
COMMENT ON COLUMN saga_steps.step_id IS 'Unique step identifier';
COMMENT ON COLUMN saga_steps.saga_id IS 'Parent saga instance ID';
COMMENT ON COLUMN saga_steps.step_name IS 'Name of the step';
COMMENT ON COLUMN saga_steps.step_sequence IS 'Execution order';
COMMENT ON COLUMN saga_steps.status IS 'Status: pending, running, completed, failed';
COMMENT ON COLUMN saga_steps.started_at IS 'Step start timestamp';
COMMENT ON COLUMN saga_steps.completed_at IS 'Step completion timestamp';
COMMENT ON COLUMN saga_steps.compensation_required IS 'Whether compensation is needed';
COMMENT ON COLUMN saga_steps.compensation_status IS 'Compensation status';
COMMENT ON COLUMN saga_steps.compensated_at IS 'Compensation completion timestamp';
COMMENT ON COLUMN saga_steps.command_data IS 'Command sent to service';
COMMENT ON COLUMN saga_steps.response_data IS 'Response from service';
COMMENT ON COLUMN saga_steps.error_data IS 'Error details if failed';
COMMENT ON COLUMN saga_steps.retry_count IS 'Number of retry attempts';
COMMENT ON COLUMN saga_steps.next_retry_at IS 'Next retry timestamp';
COMMENT ON COLUMN saga_steps.created_at IS 'Step creation timestamp';
COMMENT ON COLUMN saga_steps.updated_at IS 'Last update timestamp';

-- Add comments for saga_timeouts
COMMENT ON TABLE saga_timeouts IS 'Saga timeout management';
COMMENT ON COLUMN saga_timeouts.timeout_id IS 'Unique timeout identifier';
COMMENT ON COLUMN saga_timeouts.saga_id IS 'Parent saga instance ID';
COMMENT ON COLUMN saga_timeouts.timeout_type IS 'Type of timeout';
COMMENT ON COLUMN saga_timeouts.timeout_at IS 'When the timeout should trigger';
COMMENT ON COLUMN saga_timeouts.status IS 'Status: pending, processed, cancelled';
COMMENT ON COLUMN saga_timeouts.processed_at IS 'When the timeout was processed';
COMMENT ON COLUMN saga_timeouts.created_at IS 'Timeout creation timestamp';