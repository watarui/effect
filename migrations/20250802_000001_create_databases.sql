-- Create databases for each service
-- This migration should be run as a superuser

-- User Service Database
CREATE DATABASE user_db;

-- Vocabulary Service Database
CREATE DATABASE vocabulary_db;

-- Learning Service Database
CREATE DATABASE learning_db;

-- Algorithm Service Database
CREATE DATABASE algorithm_db;

-- AI Service Database
CREATE DATABASE ai_db;

-- Progress Service Database (Read Model)
CREATE DATABASE progress_db;

-- Event Store Database
CREATE DATABASE event_store_db;

-- Saga Orchestrator Database
CREATE DATABASE saga_db;
