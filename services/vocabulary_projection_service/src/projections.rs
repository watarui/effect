//! プロジェクション（Read Model）定義

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 語彙項目のプロジェクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyItemProjection {
    /// 項目ID
    pub id:                    Uuid,
    /// 単語
    pub word:                  String,
    /// 品詞
    pub part_of_speech:        String,
    /// 定義
    pub definitions:           Vec<String>,
    /// CEFR レベル
    pub cefr_level:            Option<String>,
    /// レジスター
    pub register:              String,
    /// ドメイン
    pub domain:                String,
    /// 難易度推定値
    pub difficulty_estimate:   Option<f32>,
    /// コンテンツ品質スコア
    pub content_quality_score: Option<f32>,
    /// 公開状態
    pub is_published:          bool,
    /// 作成日時
    pub created_at:            DateTime<Utc>,
    /// 更新日時
    pub updated_at:            DateTime<Utc>,
    /// 公開日時
    pub published_at:          Option<DateTime<Utc>>,
}

/// 語彙エントリのプロジェクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyEntryProjection {
    /// エントリID
    pub id:         Uuid,
    /// 単語
    pub word:       String,
    /// 関連する項目のID一覧
    pub item_ids:   Vec<Uuid>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 更新日時
    pub updated_at: DateTime<Utc>,
}
