#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${RED}⚠️  データベースをリセットします${NC}"
echo "================================================"
echo "すべてのデータが削除されます！"
echo ""
read -p "本当に実行しますか？ (y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
	echo -e "${YELLOW}キャンセルしました${NC}"
	exit 1
fi

echo -e "\n${YELLOW}🗑️  データベースをリセット中...${NC}"

# Event Store DB
echo -e "${YELLOW}Resetting event_store_db...${NC}"
PGPASSWORD=effect_password psql -h localhost -p 5432 -U effect -d postgres -c "DROP DATABASE IF EXISTS event_store_db;" 2>/dev/null || true
PGPASSWORD=effect_password psql -h localhost -p 5432 -U effect -d postgres -c "CREATE DATABASE event_store_db;"
echo -e "${GREEN}✓ event_store_db${NC}"

# Vocabulary Command DB
echo -e "${YELLOW}Resetting vocabulary_command_db...${NC}"
PGPASSWORD=effect_password psql -h localhost -p 5434 -U effect -d postgres -c "DROP DATABASE IF EXISTS vocabulary_command_db;" 2>/dev/null || true
PGPASSWORD=effect_password psql -h localhost -p 5434 -U effect -d postgres -c "CREATE DATABASE vocabulary_command_db;"
echo -e "${GREEN}✓ vocabulary_command_db${NC}"

# Vocabulary Query DB
echo -e "${YELLOW}Resetting vocabulary_query_db...${NC}"
PGPASSWORD=effect_password psql -h localhost -p 5440 -U effect -d postgres -c "DROP DATABASE IF EXISTS vocabulary_query_db;" 2>/dev/null || true
PGPASSWORD=effect_password psql -h localhost -p 5440 -U effect -d postgres -c "CREATE DATABASE vocabulary_query_db;"
echo -e "${GREEN}✓ vocabulary_query_db${NC}"

# Vocabulary Projection DB
echo -e "${YELLOW}Resetting vocabulary_projection_db...${NC}"
PGPASSWORD=effect_password psql -h localhost -p 5441 -U effect -d postgres -c "DROP DATABASE IF EXISTS vocabulary_projection_db;" 2>/dev/null || true
PGPASSWORD=effect_password psql -h localhost -p 5441 -U effect -d postgres -c "CREATE DATABASE vocabulary_projection_db;"
echo -e "${GREEN}✓ vocabulary_projection_db${NC}"

# Learning DB
echo -e "${YELLOW}Resetting learning_db...${NC}"
PGPASSWORD=effect_password psql -h localhost -p 5433 -U effect -d postgres -c "DROP DATABASE IF EXISTS learning_db;" 2>/dev/null || true
PGPASSWORD=effect_password psql -h localhost -p 5433 -U effect -d postgres -c "CREATE DATABASE learning_db;"
echo -e "${GREEN}✓ learning_db${NC}"

# User DB
echo -e "${YELLOW}Resetting user_db...${NC}"
PGPASSWORD=effect_password psql -h localhost -p 5435 -U effect -d postgres -c "DROP DATABASE IF EXISTS user_db;" 2>/dev/null || true
PGPASSWORD=effect_password psql -h localhost -p 5435 -U effect -d postgres -c "CREATE DATABASE user_db;"
echo -e "${GREEN}✓ user_db${NC}"

# Progress DB
echo -e "${YELLOW}Resetting progress_db...${NC}"
PGPASSWORD=effect_password psql -h localhost -p 5436 -U effect -d postgres -c "DROP DATABASE IF EXISTS progress_db;" 2>/dev/null || true
PGPASSWORD=effect_password psql -h localhost -p 5436 -U effect -d postgres -c "CREATE DATABASE progress_db;"
echo -e "${GREEN}✓ progress_db${NC}"

# Algorithm DB
echo -e "${YELLOW}Resetting algorithm_db...${NC}"
PGPASSWORD=effect_password psql -h localhost -p 5437 -U effect -d postgres -c "DROP DATABASE IF EXISTS algorithm_db;" 2>/dev/null || true
PGPASSWORD=effect_password psql -h localhost -p 5437 -U effect -d postgres -c "CREATE DATABASE algorithm_db;"
echo -e "${GREEN}✓ algorithm_db${NC}"

# AI DB
echo -e "${YELLOW}Resetting ai_db...${NC}"
PGPASSWORD=effect_password psql -h localhost -p 5438 -U effect -d postgres -c "DROP DATABASE IF EXISTS ai_db;" 2>/dev/null || true
PGPASSWORD=effect_password psql -h localhost -p 5438 -U effect -d postgres -c "CREATE DATABASE ai_db;"
echo -e "${GREEN}✓ ai_db${NC}"

# Saga DB
echo -e "${YELLOW}Resetting saga_db...${NC}"
PGPASSWORD=effect_password psql -h localhost -p 5439 -U effect -d postgres -c "DROP DATABASE IF EXISTS saga_db;" 2>/dev/null || true
PGPASSWORD=effect_password psql -h localhost -p 5439 -U effect -d postgres -c "CREATE DATABASE saga_db;"
echo -e "${GREEN}✓ saga_db${NC}"

echo -e "\n================================================"
echo -e "${GREEN}✅ データベースリセット完了${NC}"
echo -e "📝 マイグレーションを実行してください: ${YELLOW}./scripts/migrate-local.sh${NC}"
echo -e "   または: ${YELLOW}make db-migrate${NC}"
