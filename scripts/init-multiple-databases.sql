-- マイクロサービス用のデータベースを作成
-- このファイルは PostgreSQL の初期化時に自動的に実行されます

-- Command Service 用データベース
CREATE DATABASE command_db
    WITH
    OWNER = effect
    ENCODING = 'UTF8'
    LC_COLLATE = 'ja_JP.UTF-8'
    LC_CTYPE = 'ja_JP.UTF-8'
    TEMPLATE = template0;

-- Query Service 用データベース
CREATE DATABASE query_db
    WITH
    OWNER = effect
    ENCODING = 'UTF8'
    LC_COLLATE = 'ja_JP.UTF-8'
    LC_CTYPE = 'ja_JP.UTF-8'
    TEMPLATE = template0;

-- Saga Executor 用データベース
CREATE DATABASE saga_db
    WITH
    OWNER = effect
    ENCODING = 'UTF8'
    LC_COLLATE = 'ja_JP.UTF-8'
    LC_CTYPE = 'ja_JP.UTF-8'
    TEMPLATE = template0;

-- Event Store 用データベース（共通）
CREATE DATABASE event_store_db
    WITH
    OWNER = effect
    ENCODING = 'UTF8'
    LC_COLLATE = 'ja_JP.UTF-8'
    LC_CTYPE = 'ja_JP.UTF-8'
    TEMPLATE = template0;

-- 各データベースへの権限付与
GRANT ALL PRIVILEGES ON DATABASE command_db TO effect;
GRANT ALL PRIVILEGES ON DATABASE query_db TO effect;
GRANT ALL PRIVILEGES ON DATABASE saga_db TO effect;
GRANT ALL PRIVILEGES ON DATABASE event_store_db TO effect;
