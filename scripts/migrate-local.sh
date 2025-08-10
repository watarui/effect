#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 ローカル開発環境のマイグレーション開始${NC}"
echo "================================================"

# Event Store Service
echo -e "\n${YELLOW}📦 Event Store Service マイグレーション...${NC}"
DATABASE_URL="postgres://effect:effect_password@localhost:5432/event_store_db" \
	sqlx migrate run --source services/event_store_service/migrations
echo -e "${GREEN}✓ Event Store Service 完了${NC}"

# Vocabulary Services (完全に独立したDB)
echo -e "\n${YELLOW}📦 Vocabulary Services マイグレーション...${NC}"

# Command Service (port 5434)
echo -e "${YELLOW}  - Command Service...${NC}"
DATABASE_URL="postgres://effect:effect_password@localhost:5434/vocabulary_command_db" \
	sqlx migrate run --source services/vocabulary_command_service/migrations
echo -e "${GREEN}  ✓ Command Service 完了${NC}"

# Query Service (port 5440)
echo -e "${YELLOW}  - Query Service...${NC}"
DATABASE_URL="postgres://effect:effect_password@localhost:5440/vocabulary_query_db" \
	sqlx migrate run --source services/vocabulary_query_service/migrations
echo -e "${GREEN}  ✓ Query Service 完了${NC}"

# Projection Service (port 5441)
echo -e "${YELLOW}  - Projection Service...${NC}"
DATABASE_URL="postgres://effect:effect_password@localhost:5441/vocabulary_projection_db" \
	sqlx migrate run --source services/vocabulary_projection_service/migrations
echo -e "${GREEN}  ✓ Projection Service 完了${NC}"

# Learning Service
echo -e "\n${YELLOW}📦 Learning Service マイグレーション...${NC}"
DATABASE_URL="postgres://effect:effect_password@localhost:5433/learning_db" \
	sqlx migrate run --source services/learning_service/migrations || echo -e "${YELLOW}⚠ Learning Service: マイグレーションファイルがありません${NC}"

# User Service
echo -e "\n${YELLOW}📦 User Service マイグレーション...${NC}"
DATABASE_URL="postgres://effect:effect_password@localhost:5435/user_db" \
	sqlx migrate run --source services/user_service/migrations || echo -e "${YELLOW}⚠ User Service: マイグレーションファイルがありません${NC}"

# Progress Services
echo -e "\n${YELLOW}📦 Progress Services マイグレーション...${NC}"
DATABASE_URL="postgres://effect:effect_password@localhost:5436/progress_db" \
	sqlx migrate run --source services/progress_command_service/migrations || echo -e "${YELLOW}⚠ Progress Command Service: マイグレーションファイルがありません${NC}"

DATABASE_URL="postgres://effect:effect_password@localhost:5436/progress_db" \
	sqlx migrate run --source services/progress_query_service/migrations || echo -e "${YELLOW}⚠ Progress Query Service: マイグレーションファイルがありません${NC}"

DATABASE_URL="postgres://effect:effect_password@localhost:5436/progress_db" \
	sqlx migrate run --source services/progress_projection_service/migrations || echo -e "${YELLOW}⚠ Progress Projection Service: マイグレーションファイルがありません${NC}"

# Algorithm Service
echo -e "\n${YELLOW}📦 Algorithm Service マイグレーション...${NC}"
DATABASE_URL="postgres://effect:effect_password@localhost:5437/algorithm_db" \
	sqlx migrate run --source services/algorithm_service/migrations || echo -e "${YELLOW}⚠ Algorithm Service: マイグレーションファイルがありません${NC}"

# AI Service
echo -e "\n${YELLOW}📦 AI Service マイグレーション...${NC}"
DATABASE_URL="postgres://effect:effect_password@localhost:5438/ai_db" \
	sqlx migrate run --source services/ai_service/migrations || echo -e "${YELLOW}⚠ AI Service: マイグレーションファイルがありません${NC}"

# Domain Events Service (Event Store DBを使用)
echo -e "\n${YELLOW}📦 Domain Events Service マイグレーション...${NC}"
DATABASE_URL="postgres://effect:effect_password@localhost:5432/event_store_db" \
	sqlx migrate run --source services/domain_events_service/migrations || echo -e "${YELLOW}⚠ Domain Events Service: マイグレーションファイルがありません${NC}"

echo -e "\n================================================"
echo -e "${GREEN}✅ すべてのマイグレーションが完了しました${NC}"
echo -e "データベースの状態を確認するには: ${YELLOW}make db-shell SERVICE=<service-name>${NC}"
