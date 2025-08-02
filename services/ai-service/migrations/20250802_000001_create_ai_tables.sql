-- Create ai_generation_requests table
CREATE TABLE IF NOT EXISTS ai_generation_requests (
    -- Primary key
    id VARCHAR(255) PRIMARY KEY,
    
    -- Request details
    request_type VARCHAR(100) NOT NULL,
    prompt TEXT NOT NULL,
    parameters JSONB NOT NULL DEFAULT '{}',
    
    -- Response
    response TEXT,
    response_metadata JSONB,
    
    -- Status tracking
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    
    -- User association
    requested_by VARCHAR(255) NOT NULL,
    
    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Create ai_templates table
CREATE TABLE IF NOT EXISTS ai_templates (
    -- Primary key
    id VARCHAR(255) PRIMARY KEY,
    
    -- Template details
    template_name VARCHAR(255) NOT NULL UNIQUE,
    template_type VARCHAR(100) NOT NULL,
    prompt_template TEXT NOT NULL,
    
    -- Configuration
    model_name VARCHAR(100) NOT NULL DEFAULT 'gemini-pro',
    temperature DECIMAL(2,1) NOT NULL DEFAULT 0.7,
    max_tokens INTEGER NOT NULL DEFAULT 1000,
    
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_ai_generation_requests_request_type ON ai_generation_requests(request_type);
CREATE INDEX idx_ai_generation_requests_status ON ai_generation_requests(status);
CREATE INDEX idx_ai_generation_requests_requested_by ON ai_generation_requests(requested_by);
CREATE INDEX idx_ai_generation_requests_created_at ON ai_generation_requests(created_at);
CREATE INDEX idx_ai_templates_template_type ON ai_templates(template_type);
CREATE INDEX idx_ai_templates_is_active ON ai_templates(is_active);

-- Add comments
COMMENT ON TABLE ai_generation_requests IS 'AI generation request history';
COMMENT ON COLUMN ai_generation_requests.id IS 'Request unique identifier';
COMMENT ON COLUMN ai_generation_requests.request_type IS 'Type of AI generation request';
COMMENT ON COLUMN ai_generation_requests.prompt IS 'Prompt sent to AI';
COMMENT ON COLUMN ai_generation_requests.parameters IS 'Additional parameters for the request';
COMMENT ON COLUMN ai_generation_requests.response IS 'AI generated response';
COMMENT ON COLUMN ai_generation_requests.response_metadata IS 'Metadata about the response';
COMMENT ON COLUMN ai_generation_requests.status IS 'Status: pending, processing, completed, failed';
COMMENT ON COLUMN ai_generation_requests.error_message IS 'Error message if failed';
COMMENT ON COLUMN ai_generation_requests.retry_count IS 'Number of retry attempts';
COMMENT ON COLUMN ai_generation_requests.requested_by IS 'User ID who made the request';
COMMENT ON COLUMN ai_generation_requests.version IS 'Version for optimistic locking';
COMMENT ON COLUMN ai_generation_requests.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN ai_generation_requests.updated_at IS 'Record last update timestamp';
COMMENT ON COLUMN ai_generation_requests.completed_at IS 'Request completion timestamp';

COMMENT ON TABLE ai_templates IS 'AI prompt templates';
COMMENT ON COLUMN ai_templates.id IS 'Template unique identifier';
COMMENT ON COLUMN ai_templates.template_name IS 'Unique name for the template';
COMMENT ON COLUMN ai_templates.template_type IS 'Type of template';
COMMENT ON COLUMN ai_templates.prompt_template IS 'Template text with placeholders';
COMMENT ON COLUMN ai_templates.model_name IS 'AI model to use';
COMMENT ON COLUMN ai_templates.temperature IS 'Temperature setting for generation';
COMMENT ON COLUMN ai_templates.max_tokens IS 'Maximum tokens for response';
COMMENT ON COLUMN ai_templates.is_active IS 'Whether this template is active';
COMMENT ON COLUMN ai_templates.version IS 'Version for optimistic locking';
COMMENT ON COLUMN ai_templates.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN ai_templates.updated_at IS 'Record last update timestamp';