#!/bin/bash
set -euo pipefail

# 色付き出力用の変数
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ログ関数
log_info() {
	echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
	echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
	echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
	echo -e "${RED}[ERROR]${NC} $1"
}

# .env ファイルを読み込む
if [ -f .env ]; then
	export "$(grep -v '^#' .env | xargs)"
else
	log_error ".env file not found!"
	exit 1
fi

# PostgreSQL の接続を待つ
wait_for_postgres() {
	local host=$1
	local port=$2
	local max_attempts=30
	local attempt=0

	log_info "Waiting for PostgreSQL on ${host}:${port}..."

	while ! nc -z "$host" "$port" >/dev/null 2>&1; do
		attempt=$((attempt + 1))
		if [ $attempt -eq $max_attempts ]; then
			log_error "PostgreSQL is not available after ${max_attempts} attempts"
			return 1
		fi
		sleep 1
	done

	log_success "PostgreSQL is available on ${host}:${port}"
	return 0
}

# データベースの存在を確認
check_database_exists() {
	local db_name=$1
	local port=$2

	PGPASSWORD=$POSTGRES_PASSWORD psql -h localhost -p "$port" -U "$POSTGRES_USER" -lqt | cut -d \| -f 1 | grep -qw "$db_name"
}

# ルートディレクトリでのマイグレーション実行
run_root_migrations() {
	log_info "Running root migrations..."

	# Event Store DB に接続してデータベース作成
	if wait_for_postgres "localhost" "$POSTGRES_EVENT_STORE_PORT"; then
		# postgres データベースに接続して他のデータベースを作成
		PGPASSWORD=$POSTGRES_PASSWORD psql -h localhost -p "$POSTGRES_EVENT_STORE_PORT" -U "$POSTGRES_USER" -d postgres <migrations/20240802_000001_create_databases.sql
		log_success "Databases created successfully"

		# Event Store テーブルを作成
		if check_database_exists "event_store_db" "$POSTGRES_EVENT_STORE_PORT"; then
			PGPASSWORD=$POSTGRES_PASSWORD psql -h localhost -p "$POSTGRES_EVENT_STORE_PORT" -U "$POSTGRES_USER" -d event_store_db <migrations/20240802_000002_create_event_store.sql
			log_success "Event store tables created"
		fi

		# Saga テーブルを作成
		if wait_for_postgres "localhost" "$POSTGRES_SAGA_PORT" && check_database_exists "saga_db" "$POSTGRES_SAGA_PORT"; then
			PGPASSWORD=$POSTGRES_PASSWORD psql -h localhost -p "$POSTGRES_SAGA_PORT" -U "$POSTGRES_USER" -d saga_db <migrations/20240802_000003_create_saga_tables.sql
			log_success "Saga tables created"
		fi
	else
		log_error "Failed to connect to PostgreSQL"
		exit 1
	fi
}

# サービスごとのマイグレーション実行
run_service_migrations() {
	local service=$1
	local db_name=$2
	local port=$3

	log_info "Running migrations for ${service}..."

	if wait_for_postgres "localhost" "$port"; then
		if check_database_exists "$db_name" "$port"; then
			# サービスディレクトリのマイグレーションを実行
			if [ -d "services/${service}/migrations" ]; then
				for migration in services/"${service}"/migrations/*.sql; do
					if [ -f "$migration" ]; then
						log_info "Applying migration: $(basename "$migration")"
						PGPASSWORD=$POSTGRES_PASSWORD psql -h localhost -p "$port" -U "$POSTGRES_USER" -d "$db_name" <"$migration"
					fi
				done
				log_success "Migrations completed for ${service}"
			else
				log_warning "No migrations found for ${service}"
			fi
		else
			log_error "Database ${db_name} does not exist"
			return 1
		fi
	else
		log_error "Failed to connect to PostgreSQL on port ${port}"
		return 1
	fi
}

# メイン処理
main() {
	log_info "Starting migration process..."

	# ルートマイグレーション実行
	run_root_migrations

	# 各サービスのマイグレーション実行
	run_service_migrations "user-service" "user_db" "$POSTGRES_USER_PORT"
	run_service_migrations "vocabulary-service" "vocabulary_db" "$POSTGRES_VOCABULARY_PORT"
	run_service_migrations "learning-service" "learning_db" "$POSTGRES_LEARNING_PORT"
	run_service_migrations "algorithm-service" "algorithm_db" "$POSTGRES_ALGORITHM_PORT"
	run_service_migrations "ai-service" "ai_db" "$POSTGRES_AI_PORT"
	run_service_migrations "progress-service" "progress_db" "$POSTGRES_PROGRESS_PORT"

	log_success "All migrations completed successfully!"
}

# スクリプト実行
main "$@"
