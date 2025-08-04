//! 統合テスト
//!
//! サービスの基本的な動作確認

#[cfg(test)]
mod tests {
    use vocabulary_query_service::domain::read_models::VocabularyStats;

    #[test]
    fn test_vocabulary_stats() {
        // 基本的な動作確認テスト
        let stats = VocabularyStats {
            total_entries:  10,
            total_items:    20,
            total_examples: 30,
            last_updated:   chrono::Utc::now(),
        };

        assert_eq!(stats.total_entries, 10);
        assert_eq!(stats.total_items, 20);
        assert_eq!(stats.total_examples, 30);
    }
}
