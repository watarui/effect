# コア学習機能仕様

## 概要

Effect のコア学習機能は、科学的根拠に基づいた効率的な英単語学習を実現します。
SM-2 アルゴリズムを採用し、個人の学習進度に最適化された復習スケジュールを提供します。

## 機能一覧

### 1. 学習セッション

#### セッション開始

```rust
pub struct StartSessionInput {
    pub user_id: Uuid,
    pub mode: LearningMode,
    pub word_count: u32,
    pub categories: Option<Vec<TestCategory>>,
    pub difficulty_range: Option<(u8, u8)>,
}

pub enum LearningMode {
    MultipleChoice,    // 4択問題
    Typing,           // タイピング
    Listening,        // リスニング
    Speaking,         // スピーキング
}
```

#### 出題アルゴリズム

1. **新規単語**: 未学習の単語から優先的に出題
2. **復習単語**: SM-2 アルゴリズムに基づく復習期限の単語
3. **苦手単語**: 正答率の低い単語を重点的に

#### 問題形式

##### 4択問題

- 正解の単語 + ランダムな3つの選択肢
- 選択肢は同じカテゴリ・難易度から選定

##### タイピング問題

- 日本語の意味を表示し、英単語を入力
- スペルチェック機能

##### リスニング問題

- 音声再生（Google Text-to-Speech）
- 聞き取った単語を選択/入力

### 2. SM-2 アルゴリズム

#### 基本パラメータ

```rust
pub struct SM2Parameters {
    pub repetition_count: u32,    // 復習回数
    pub easiness_factor: f32,     // 難易度係数（初期値: 2.5）
    pub interval_days: u32,       // 次回復習までの日数
}
```

#### 計算ロジック

```rust
pub fn calculate_next_interval(
    quality: u8,  // 回答の質（0-5）
    current: &SM2Parameters,
) -> SM2Parameters {
    let mut params = current.clone();

    // 難易度係数の更新
    params.easiness_factor =
        (params.easiness_factor + 0.1 - (5.0 - quality as f32) * (0.08 + (5.0 - quality as f32) * 0.02))
        .max(1.3);

    // 復習間隔の計算
    if quality < 3 {
        params.repetition_count = 0;
        params.interval_days = 1;
    } else {
        params.repetition_count += 1;
        params.interval_days = match params.repetition_count {
            1 => 1,
            2 => 6,
            _ => (params.interval_days as f32 * params.easiness_factor) as u32,
        };
    }

    params
}
```

### 3. 進捗管理

#### 学習統計

- 日別学習単語数
- 正答率の推移
- カテゴリ別習熟度
- 学習時間統計

#### ストリーク機能

- 連続学習日数のカウント
- ストリーク維持の通知
- マイルストーン報酬

## データモデル

### LearningSession エンティティ

```rust
pub struct LearningSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub mode: LearningMode,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub word_ids: Vec<Uuid>,
    pub results: Vec<QuestionResult>,
}

pub struct QuestionResult {
    pub word_id: Uuid,
    pub is_correct: bool,
    pub response_time_ms: u32,
    pub attempt_number: u8,
}
```

### UserWordProgress エンティティ

```rust
pub struct UserWordProgress {
    pub user_id: Uuid,
    pub word_id: Uuid,
    pub sm2_params: SM2Parameters,
    pub mastery_level: f32,  // 0.0-1.0
    pub total_reviews: u32,
    pub correct_count: u32,
    pub last_reviewed_at: DateTime<Utc>,
    pub next_review_at: DateTime<Utc>,
}
```

## ユーザーインターフェース要件

### 学習画面

- 進捗バー表示
- 残り問題数表示
- スキップ機能
- ヒント表示（例文）

### 統計画面

- グラフによる可視化
- エクスポート機能（CSV）
- 目標設定と達成率