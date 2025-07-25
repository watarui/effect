# ドメインサービス

## 概要

ドメインサービスは、特定のエンティティやアグリゲートに属さないドメインロジックをカプセル化します。
複数のアグリゲートにまたがる操作や、外部リソースとの協調が必要な場合に使用します。

## ドメインサービスの原則

1. **ステートレス**: 状態を持たない
2. **ドメインロジック**: ビジネスルールを含む
3. **アグリゲート間の調整**: 複数のアグリゲートを操作
4. **名前は動詞**: 行動を表す名前（〜Service）

## Learning Context のドメインサービス

### WordSelectionService

**責務**: 学習セッションのための単語選択アルゴリズム

```rust
use async_trait::async_trait;

#[async_trait]
pub trait WordSelectionService: Send + Sync {
    async fn select_words_for_session(
        &self,
        user_id: UserId,
        count: u32,
        criteria: SelectionCriteria,
    ) -> Result<Vec<WordId>, DomainError>;
}

pub struct SmartWordSelectionService {
    word_repository: Arc<dyn WordRepository>,
    progress_repository: Arc<dyn UserProgressRepository>,
}

#[async_trait]
impl WordSelectionService for SmartWordSelectionService {
    async fn select_words_for_session(
        &self,
        user_id: UserId,
        count: u32,
        criteria: SelectionCriteria,
    ) -> Result<Vec<WordId>, DomainError> {
        // 1. 期限切れの復習単語を取得
        let overdue_words = self.progress_repository
            .find_overdue_words(user_id, Utc::today())
            .await?;

        // 2. 今日復習予定の単語を取得
        let due_today = self.progress_repository
            .find_due_words(user_id, Utc::today())
            .await?;

        // 3. 新規学習単語の候補を取得
        let new_words = self.word_repository
            .find_unlearned_words(user_id, criteria.clone())
            .await?;

        // 4. 優先順位に基づいて選択
        let mut selected = Vec::new();

        // 期限切れを最優先
        selected.extend(overdue_words.into_iter().take(count as usize));

        // 残りの枠で今日の復習
        let remaining = count as usize - selected.len();
        selected.extend(due_today.into_iter().take(remaining));

        // さらに残りの枠で新規単語
        let remaining = count as usize - selected.len();
        selected.extend(new_words.into_iter().take(remaining));

        // 指定数に満たない場合のエラー
        if selected.len() < count as usize * 0.5 {
            return Err(DomainError::InsufficientWords);
        }

        Ok(selected)
    }
}

#[derive(Clone, Debug)]
pub struct SelectionCriteria {
    pub categories: Vec<Category>,
    pub difficulty_range: (u8, u8),
    pub exclude_mastered: bool,
}
```

### SM2CalculationService

**責務**: SM-2 アルゴリズムの計算ロジック

```rust
pub struct SM2CalculationService;

impl SM2CalculationService {
    pub fn calculate_next_review(
        &self,
        current_params: &SM2Parameters,
        quality: QualityRating,
    ) -> (SM2Parameters, Date) {
        let mut new_params = current_params.clone();

        // Easiness Factor の更新
        new_params.easiness_factor = self.calculate_easiness_factor(
            current_params.easiness_factor,
            quality.value(),
        );

        // 復習間隔の計算
        if quality.value() < 3 {
            // 不正解の場合はリセット
            new_params.repetition_count = 0;
            new_params.interval_days = 1;
        } else {
            new_params.repetition_count += 1;
            new_params.interval_days = self.calculate_interval(
                new_params.repetition_count,
                new_params.easiness_factor,
                current_params.interval_days,
            );
        }

        let next_review_date = Utc::today() + Duration::days(new_params.interval_days as i64);

        (new_params, next_review_date.naive_utc().into())
    }

    fn calculate_easiness_factor(&self, current_ef: f32, quality: u8) -> f32 {
        let q = quality as f32;
        let new_ef = current_ef + 0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02);
        new_ef.max(1.3)  // 最小値は1.3
    }

    fn calculate_interval(
        &self,
        repetition: u32,
        easiness_factor: f32,
        previous_interval: u32,
    ) -> u32 {
        match repetition {
            1 => 1,
            2 => 6,
            _ => (previous_interval as f32 * easiness_factor).round() as u32,
        }
    }
}
```

### QuestionGenerationService

**責務**: 学習問題の生成

```rust
#[async_trait]
pub trait QuestionGenerationService: Send + Sync {
    async fn generate_question(
        &self,
        word_id: WordId,
        mode: LearningMode,
        user_progress: &UserProgress,
    ) -> Result<Question, DomainError>;
}

pub struct AdaptiveQuestionGenerationService {
    word_repository: Arc<dyn WordRepository>,
}

#[async_trait]
impl QuestionGenerationService for AdaptiveQuestionGenerationService {
    async fn generate_question(
        &self,
        word_id: WordId,
        mode: LearningMode,
        user_progress: &UserProgress,
    ) -> Result<Question, DomainError> {
        let word = self.word_repository.find_by_id(word_id).await?
            .ok_or(DomainError::WordNotFound)?;

        match mode {
            LearningMode::MultipleChoice { options } => {
                self.generate_multiple_choice(word, options, user_progress).await
            }
            LearningMode::Typing => {
                self.generate_typing_question(word, user_progress).await
            }
            LearningMode::Listening => {
                self.generate_listening_question(word).await
            }
            _ => Err(DomainError::UnsupportedMode),
        }
    }

    async fn generate_multiple_choice(
        &self,
        word: Word,
        option_count: u8,
        user_progress: &UserProgress,
    ) -> Result<Question, DomainError> {
        // 難易度に応じて問題タイプを選択
        let question_type = if user_progress.mastery_level.percentage() < 30 {
            QuestionType::WordToMeaning  // 初級は単語→意味
        } else if user_progress.mastery_level.percentage() < 70 {
            QuestionType::MeaningToWord  // 中級は意味→単語
        } else {
            QuestionType::FillInTheBlank // 上級は穴埋め
        };

        // 選択肢の生成
        let distractors = self.find_distractors(&word, option_count - 1).await?;

        Ok(Question {
            id: QuestionId::new(),
            word_id: word.id,
            question_type,
            question_text: self.format_question(&word, question_type),
            options: self.create_options(word, distractors),
            correct_answer: self.determine_correct_answer(&word, question_type),
            hint: self.generate_hint(&word, user_progress),
        })
    }
}
```

## Word Management Context のドメインサービス

### WordEnrichmentService

**責務**: 外部リソースを使用した単語情報の充実化

```rust
#[async_trait]
pub trait WordEnrichmentService: Send + Sync {
    async fn enrich_word(
        &self,
        word: &mut Word,
    ) -> Result<EnrichmentResult, DomainError>;
}

pub struct AIWordEnrichmentService {
    pronunciation_service: Arc<dyn PronunciationService>,
    image_service: Arc<dyn ImageSearchService>,
    ai_client: Arc<dyn AIClient>,
}

#[async_trait]
impl WordEnrichmentService for AIWordEnrichmentService {
    async fn enrich_word(
        &self,
        word: &mut Word,
    ) -> Result<EnrichmentResult, DomainError> {
        let mut result = EnrichmentResult::default();

        // 発音記号の生成
        if word.phonetic_ipa.is_empty() {
            match self.pronunciation_service.get_ipa(&word.text).await {
                Ok(ipa) => {
                    word.phonetic_ipa = ipa;
                    result.pronunciation_added = true;
                }
                Err(_) => result.errors.push("Failed to get pronunciation".into()),
            }
        }

        // AI による例文生成
        if word.examples.is_empty() {
            match self.generate_examples(word).await {
                Ok(examples) => {
                    for example in examples {
                        word.add_example(
                            example.meaning_id,
                            example.sentence,
                            example.translation,
                            UserId::system(),
                        )?;
                    }
                    result.examples_added = true;
                }
                Err(_) => result.errors.push("Failed to generate examples".into()),
            }
        }

        // 画像の検索
        if word.image_url.is_none() {
            match self.image_service.search_image(&word.text).await {
                Ok(image_url) => {
                    word.image_url = Some(image_url);
                    result.image_added = true;
                }
                Err(_) => result.errors.push("Failed to find image".into()),
            }
        }

        Ok(result)
    }

    async fn generate_examples(&self, word: &Word) -> Result<Vec<GeneratedExample>, DomainError> {
        let prompt = format!(
            "Generate 3 example sentences for the word '{}' with Japanese translations. \
             Focus on practical, everyday usage.",
            word.text.as_str()
        );

        self.ai_client.generate_examples(prompt).await
    }
}
```

### DuplicateCheckService

**責務**: 単語の重複チェックと関連付け

```rust
pub struct DuplicateCheckService {
    word_repository: Arc<dyn WordRepository>,
}

impl DuplicateCheckService {
    pub async fn check_and_link_duplicates(
        &self,
        new_word: &Word,
    ) -> Result<DuplicateCheckResult, DomainError> {
        let mut result = DuplicateCheckResult::default();

        // 完全一致のチェック
        if let Some(existing) = self.word_repository
            .find_by_text(new_word.text.as_str())
            .await?
        {
            result.exact_match = Some(existing.id);
            return Ok(result);
        }

        // スペルバリエーションのチェック
        let variants = self.find_spelling_variants(&new_word.text);
        for variant in variants {
            if let Some(existing) = self.word_repository
                .find_by_text(&variant)
                .await?
            {
                result.spelling_variants.push(existing.id);
            }
        }

        // 類似単語のチェック（編集距離）
        let similar = self.word_repository
            .find_similar_words(new_word.text.as_str(), 2)  // 編集距離2以内
            .await?;

        result.similar_words = similar.into_iter()
            .map(|w| (w.id, self.calculate_similarity(&new_word.text, &w.text)))
            .collect();

        Ok(result)
    }

    fn find_spelling_variants(&self, word: &WordText) -> Vec<String> {
        let text = word.as_str();
        let mut variants = Vec::new();

        // アメリカ英語 ↔ イギリス英語
        variants.extend(self.apply_spelling_rules(text));

        variants
    }

    fn apply_spelling_rules(&self, text: &str) -> Vec<String> {
        vec![
            text.replace("our", "or"),      // colour -> color
            text.replace("or", "our"),       // color -> colour
            text.replace("ise", "ize"),      // organise -> organize
            text.replace("ize", "ise"),      // organize -> organise
            text.replace("re", "er"),        // centre -> center
            text.replace("er", "re"),        // center -> centre
        ].into_iter()
        .filter(|variant| variant != text)
        .collect()
    }
}
```

## User Context のドメインサービス

### AuthenticationService

**責務**: ユーザー認証の処理

```rust
#[async_trait]
pub trait AuthenticationService: Send + Sync {
    async fn authenticate(
        &self,
        credentials: Credentials,
    ) -> Result<AuthResult, DomainError>;

    async fn validate_token(
        &self,
        token: &str,
    ) -> Result<TokenValidation, DomainError>;
}

pub struct JwtAuthenticationService {
    user_repository: Arc<dyn UserRepository>,
    password_hasher: Arc<dyn PasswordHasher>,
    jwt_service: Arc<dyn JwtService>,
}

#[async_trait]
impl AuthenticationService for JwtAuthenticationService {
    async fn authenticate(
        &self,
        credentials: Credentials,
    ) -> Result<AuthResult, DomainError> {
        match credentials {
            Credentials::EmailPassword { email, password } => {
                // ユーザーの取得
                let user = self.user_repository
                    .find_by_email(&email)
                    .await?
                    .ok_or(DomainError::InvalidCredentials)?;

                // パスワードの検証
                if !self.password_hasher.verify(&password, &user.auth_info.password_hash)? {
                    return Err(DomainError::InvalidCredentials);
                }

                // アカウント状態の確認
                if !user.account_status.can_login() {
                    return Err(DomainError::AccountLocked);
                }

                // トークンの生成
                let token_pair = self.jwt_service.generate_token_pair(&user)?;

                Ok(AuthResult {
                    user_id: user.id,
                    tokens: token_pair,
                })
            }
            Credentials::OAuth { provider, token } => {
                // OAuth プロバイダーとの連携
                self.handle_oauth_login(provider, token).await
            }
        }
    }
}
```

## Progress Context のドメインサービス

### StreakCalculationService

**責務**: 学習ストリークの計算

```rust
pub struct StreakCalculationService {
    timezone_service: Arc<dyn TimezoneService>,
}

impl StreakCalculationService {
    pub async fn calculate_streak(
        &self,
        user_id: UserId,
        sessions: &[SessionSummary],
        user_timezone: &Timezone,
    ) -> StreakInfo {
        // ユーザーのタイムゾーンで日付を計算
        let today = self.timezone_service.today_in_timezone(user_timezone);
        let mut current_streak = 0;
        let mut longest_streak = 0;
        let mut temp_streak = 0;
        let mut last_date: Option<Date> = None;

        // セッションを日付順にソート
        let mut sorted_sessions = sessions.to_vec();
        sorted_sessions.sort_by_key(|s| s.completed_at);

        for session in sorted_sessions {
            let session_date = self.timezone_service
                .to_user_date(session.completed_at, user_timezone);

            match last_date {
                None => {
                    temp_streak = 1;
                }
                Some(last) => {
                    let days_diff = (session_date - last).num_days();

                    if days_diff == 1 {
                        // 連続している
                        temp_streak += 1;
                    } else if days_diff > 1 {
                        // 途切れた
                        longest_streak = longest_streak.max(temp_streak);
                        temp_streak = 1;
                    }
                    // days_diff == 0 の場合は同じ日なのでカウントしない
                }
            }

            last_date = Some(session_date);
        }

        // 最後のストリークの確認
        longest_streak = longest_streak.max(temp_streak);

        // 現在のストリークの計算
        if let Some(last) = last_date {
            let days_since_last = (today - last).num_days();
            current_streak = if days_since_last <= 1 {
                temp_streak
            } else {
                0
            };
        }

        StreakInfo {
            current_streak,
            longest_streak,
            last_study_date: last_date,
            at_risk: current_streak > 0 && last_date == Some(today.pred()),
        }
    }
}
```

### AchievementService

**責務**: 達成判定とバッジ付与

```rust
pub struct AchievementService {
    achievement_repository: Arc<dyn AchievementRepository>,
}

impl AchievementService {
    pub async fn check_achievements(
        &self,
        user_id: UserId,
        trigger: AchievementTrigger,
    ) -> Result<Vec<Achievement>, DomainError> {
        let mut new_achievements = Vec::new();

        // ユーザーの既存の達成を取得
        let existing = self.achievement_repository
            .find_by_user(user_id)
            .await?;

        // トリガーに応じた達成チェック
        match trigger {
            AchievementTrigger::SessionCompleted { stats } => {
                // 初回セッション
                if stats.total_sessions == 1 && !self.has_achievement(&existing, "first_session") {
                    new_achievements.push(Achievement::FirstSession);
                }

                // 100セッション達成
                if stats.total_sessions == 100 && !self.has_achievement(&existing, "century") {
                    new_achievements.push(Achievement::Century);
                }
            }

            AchievementTrigger::WordMastered { total } => {
                // 単語マスター系の達成
                let milestones = vec![(10, "novice"), (50, "learner"), (100, "scholar")];

                for (count, name) in milestones {
                    if total >= count && !self.has_achievement(&existing, name) {
                        new_achievements.push(Achievement::WordMilestone(count));
                    }
                }
            }

            AchievementTrigger::StreakReached { days } => {
                // ストリーク系の達成
                let milestones = vec![(7, "week"), (30, "month"), (100, "dedication")];

                for (count, name) in milestones {
                    if days >= count && !self.has_achievement(&existing, name) {
                        new_achievements.push(Achievement::StreakMilestone(count));
                    }
                }
            }
        }

        // 新しい達成を保存
        for achievement in &new_achievements {
            self.achievement_repository
                .grant_achievement(user_id, achievement.clone())
                .await?;
        }

        Ok(new_achievements)
    }
}
```

## ドメインサービスの利用例

```rust
// アプリケーションサービスでの利用
pub struct StartLearningSessionUseCase {
    word_selection_service: Arc<dyn WordSelectionService>,
    session_repository: Arc<dyn SessionRepository>,
}

impl StartLearningSessionUseCase {
    pub async fn execute(
        &self,
        user_id: UserId,
        mode: LearningMode,
    ) -> Result<SessionId, ApplicationError> {
        // ドメインサービスで単語を選択
        let words = self.word_selection_service
            .select_words_for_session(
                user_id,
                20,  // デフォルト20単語
                SelectionCriteria::default(),
            )
            .await?;

        // セッションの作成
        let session = LearningSession::start(
            user_id,
            words,
            SessionConfig { mode, word_count: 20, time_limit: None },
        )?;

        // 保存
        self.session_repository.save(session).await?;

        Ok(session.id)
    }
}
```

## 更新履歴

- 2025-07-25: 初版作成
