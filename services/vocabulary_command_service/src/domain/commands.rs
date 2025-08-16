use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// VocabularyEntry を作成するコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVocabularyEntry {
    pub spelling: String,
}

/// VocabularyItem を作成するコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVocabularyItem {
    pub entry_id:       Uuid,
    pub spelling:       String,
    pub disambiguation: Option<String>,
}

/// VocabularyItem を更新するコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVocabularyItem {
    pub item_id:        Uuid,
    pub disambiguation: Option<String>,
    pub version:        i64,
}

/// VocabularyItem を公開するコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishVocabularyItem {
    pub item_id: Uuid,
    pub version: i64,
}

/// AI エンリッチメントをリクエストするコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestAIEnrichment {
    pub item_id: Uuid,
    pub version: i64,
}

/// AI エンリッチメントを完了するコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteAIEnrichment {
    pub item_id:       Uuid,
    pub enriched_data: EnrichedData,
    pub version:       i64,
}

/// AI エンリッチメントのデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedData {
    pub definitions:   Vec<Definition>,
    pub examples:      Vec<Example>,
    pub pronunciation: Option<String>,
    pub etymology:     Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub text:           String,
    pub part_of_speech: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub text:        String,
    pub translation: Option<String>,
}

/// 主要項目として設定するコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAsPrimaryItem {
    pub entry_id: Uuid,
    pub item_id:  Uuid,
    pub version:  i64,
}

/// VocabularyItem を削除するコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteVocabularyItem {
    pub item_id:    Uuid,
    pub deleted_by: Uuid,
}

/// VocabularyItem に例文を追加するコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddExample {
    pub item_id:     Uuid,
    pub example:     String,
    pub translation: Option<String>,
    pub added_by:    Uuid,
    pub version:     i64,
}
