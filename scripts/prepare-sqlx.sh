#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔧 SQLX オフラインモード準備開始${NC}"
echo "================================================"
echo -e "${YELLOW}注意: 各サービスのsqlx-data.jsonファイルが生成されます${NC}"
echo ""

# 現在のディレクトリを保存
ORIGINAL_DIR=$(pwd)

# Event Store Service
echo -e "${YELLOW}📦 Event Store Service...${NC}"
cd services/event_store_service
DATABASE_URL="postgres://effect:effect_password@localhost:5432/event_store_db" \
	cargo sqlx prepare || echo -e "${RED}✗ Event Store Service 失敗${NC}"
cd "$ORIGINAL_DIR"
echo -e "${GREEN}✓ Event Store Service 完了${NC}"

# Vocabulary Command Service (port 5434)
echo -e "\n${YELLOW}📦 Vocabulary Command Service...${NC}"
cd services/vocabulary_command_service
DATABASE_URL="postgres://effect:effect_password@localhost:5434/vocabulary_command_db" \
	cargo sqlx prepare || echo -e "${RED}✗ Vocabulary Command Service 失敗${NC}"
cd "$ORIGINAL_DIR"
echo -e "${GREEN}✓ Vocabulary Command Service 完了${NC}"

# Vocabulary Query Service (port 5440)
echo -e "\n${YELLOW}📦 Vocabulary Query Service...${NC}"
cd services/vocabulary_query_service
DATABASE_URL="postgres://effect:effect_password@localhost:5440/vocabulary_query_db" \
	cargo sqlx prepare || echo -e "${RED}✗ Vocabulary Query Service 失敗${NC}"
cd "$ORIGINAL_DIR"
echo -e "${GREEN}✓ Vocabulary Query Service 完了${NC}"

# Vocabulary Projection Service (port 5441)
echo -e "\n${YELLOW}📦 Vocabulary Projection Service...${NC}"
cd services/vocabulary_projection_service
DATABASE_URL="postgres://effect:effect_password@localhost:5441/vocabulary_projection_db" \
	cargo sqlx prepare || echo -e "${RED}✗ Vocabulary Projection Service 失敗${NC}"
cd "$ORIGINAL_DIR"
echo -e "${GREEN}✓ Vocabulary Projection Service 完了${NC}"

# Vocabulary Search Service (Query DB を使用 - port 5440)
echo -e "\n${YELLOW}📦 Vocabulary Search Service...${NC}"
cd services/vocabulary_search_service
DATABASE_URL="postgres://effect:effect_password@localhost:5440/vocabulary_query_db" \
	cargo sqlx prepare || echo -e "${RED}✗ Vocabulary Search Service 失敗${NC}"
cd "$ORIGINAL_DIR"
echo -e "${GREEN}✓ Vocabulary Search Service 完了${NC}"

# Learning Service
echo -e "\n${YELLOW}📦 Learning Service...${NC}"
cd services/learning_service
DATABASE_URL="postgres://effect:effect_password@localhost:5433/learning_db" \
	cargo sqlx prepare || echo -e "${YELLOW}⚠ Learning Service: SQLXクエリがない、またはビルドエラー${NC}"
cd "$ORIGINAL_DIR"

# User Service
echo -e "\n${YELLOW}📦 User Service...${NC}"
cd services/user_service
DATABASE_URL="postgres://effect:effect_password@localhost:5435/user_db" \
	cargo sqlx prepare || echo -e "${YELLOW}⚠ User Service: SQLXクエリがない、またはビルドエラー${NC}"
cd "$ORIGINAL_DIR"

# Progress Command Service
echo -e "\n${YELLOW}📦 Progress Command Service...${NC}"
cd services/progress_command_service
DATABASE_URL="postgres://effect:effect_password@localhost:5436/progress_db" \
	cargo sqlx prepare || echo -e "${YELLOW}⚠ Progress Command Service: SQLXクエリがない、またはビルドエラー${NC}"
cd "$ORIGINAL_DIR"

# Progress Query Service
echo -e "\n${YELLOW}📦 Progress Query Service...${NC}"
cd services/progress_query_service
DATABASE_URL="postgres://effect:effect_password@localhost:5436/progress_db" \
	cargo sqlx prepare || echo -e "${YELLOW}⚠ Progress Query Service: SQLXクエリがない、またはビルドエラー${NC}"
cd "$ORIGINAL_DIR"

# Progress Projection Service
echo -e "\n${YELLOW}📦 Progress Projection Service...${NC}"
cd services/progress_projection_service
DATABASE_URL="postgres://effect:effect_password@localhost:5436/progress_db" \
	cargo sqlx prepare || echo -e "${YELLOW}⚠ Progress Projection Service: SQLXクエリがない、またはビルドエラー${NC}"
cd "$ORIGINAL_DIR"

# Algorithm Service
echo -e "\n${YELLOW}📦 Algorithm Service...${NC}"
cd services/algorithm_service
DATABASE_URL="postgres://effect:effect_password@localhost:5437/algorithm_db" \
	cargo sqlx prepare || echo -e "${YELLOW}⚠ Algorithm Service: SQLXクエリがない、またはビルドエラー${NC}"
cd "$ORIGINAL_DIR"

# AI Service
echo -e "\n${YELLOW}📦 AI Service...${NC}"
cd services/ai_service
DATABASE_URL="postgres://effect:effect_password@localhost:5438/ai_db" \
	cargo sqlx prepare || echo -e "${YELLOW}⚠ AI Service: SQLXクエリがない、またはビルドエラー${NC}"
cd "$ORIGINAL_DIR"

# Domain Events Service
echo -e "\n${YELLOW}📦 Domain Events Service...${NC}"
cd services/domain_events_service
DATABASE_URL="postgres://effect:effect_password@localhost:5432/event_store_db" \
	cargo sqlx prepare || echo -e "${YELLOW}⚠ Domain Events Service: SQLXクエリがない、またはビルドエラー${NC}"
cd "$ORIGINAL_DIR"

echo -e "\n================================================"
echo -e "${GREEN}✅ SQLX オフラインモード準備完了${NC}"
echo -e "📝 sqlx-data.json ファイルが各サービスに生成されました"
echo -e "これにより、${YELLOW}SQLX_OFFLINE=true${NC} でビルドできるようになります"
