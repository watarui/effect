-- Create users table
CREATE TABLE IF NOT EXISTS users (
    -- Primary key
    id VARCHAR(255) PRIMARY KEY,
    
    -- User information
    email VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255),
    
    -- Profile JSON
    profile JSONB NOT NULL DEFAULT '{}',
    
    -- Status and role
    account_status VARCHAR(50) NOT NULL DEFAULT 'active',
    role VARCHAR(50) NOT NULL DEFAULT 'free_user',
    
    -- Learning goal
    learning_goal VARCHAR(50) NOT NULL DEFAULT 'general',
    
    -- Versioning for optimistic locking
    version BIGINT NOT NULL DEFAULT 1,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_account_status ON users(account_status);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NOT NULL;

-- Add comment to table
COMMENT ON TABLE users IS 'User aggregate root table';
COMMENT ON COLUMN users.id IS 'User unique identifier';
COMMENT ON COLUMN users.email IS 'User email address (unique)';
COMMENT ON COLUMN users.display_name IS 'User display name';
COMMENT ON COLUMN users.profile IS 'User profile data in JSON format';
COMMENT ON COLUMN users.account_status IS 'Account status: active, suspended, deleted';
COMMENT ON COLUMN users.role IS 'User role: free_user, premium_user, admin';
COMMENT ON COLUMN users.learning_goal IS 'User learning goal';
COMMENT ON COLUMN users.version IS 'Version for optimistic locking';
COMMENT ON COLUMN users.created_at IS 'Record creation timestamp';
COMMENT ON COLUMN users.updated_at IS 'Record last update timestamp';
COMMENT ON COLUMN users.deleted_at IS 'Soft delete timestamp';