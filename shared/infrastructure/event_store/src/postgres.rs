// PostgreSQL Event Store 実装
// TODO: 実装予定 - CQRS/Event Sourcing パターンのための永続化層

/// PostgreSQL ベースの Event Store 実装
pub struct PostgresEventStore;

impl PostgresEventStore {
    pub fn new(_pool: sqlx::PgPool) -> Self {
        Self
    }
}

// TODO: EventStore trait の実装
