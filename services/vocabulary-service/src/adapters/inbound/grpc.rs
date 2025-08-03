//! gRPC サービスの実装

use std::sync::Arc;

use crate::{application::services::VocabularyService, ports::outbound::repository::Repository};

/// gRPC サービスの実装
#[allow(dead_code)]
pub struct VocabularyGrpcService<R> {
    service: Arc<VocabularyService<R>>,
}

impl<R> VocabularyGrpcService<R>
where
    R: Repository + Send + Sync + Clone + 'static,
{
    /// 新しい gRPC サービスを作成
    #[must_use]
    pub fn new(service: VocabularyService<R>) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

// TODO: proto ファイルから生成されたトレイトを実装
// 現在は基本的な構造のみ定義

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ports::outbound::repository::MockRepository;

    #[test]
    fn test_grpc_service_creation() {
        let mock_repo = MockRepository::new();
        let app_service = VocabularyService::new(mock_repo);
        let _grpc_service = VocabularyGrpcService::new(app_service);
    }
}
