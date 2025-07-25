# 値オブジェクトカタログ

## 概要

このドキュメントでは、Effect で使用する値オブジェクトを定義します。
値オブジェクトは不変で、等価性は属性によって判断されます。

## 共通の値オブジェクト

### ID 型

```rust
use uuid::Uuid;
use serde::{Serialize, Deserialize};

// 基底トレイト
pub trait EntityId: Clone + Debug + PartialEq + Eq + Hash + Serialize {
    fn value(&self) -> Uuid;
    fn new() -> Self;
}

// マクロで各種 ID を定義
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn value(&self) -> Uuid {
                self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// 各種 ID の定義
define_id!(UserId);
define_id!(WordId);
define_id!(SessionId);
define_id!(QuestionId);
define_id!(MeaningId);
define_id!(ExampleId);
define_id!(RelationId);
```

### 日時関連

```rust
use chrono::{DateTime, Utc, NaiveTime};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn value(&self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Time(NaiveTime);

impl Time {
    pub fn new(hour: u32, minute: u32) -> Result<Self, ValueError> {
        NaiveTime::from_hms_opt(hour, minute, 0)
            .map(Self)
            .ok_or(ValueError::InvalidTime)
    }
}
```

## Learning Context の値オブジェクト

### SessionState

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    InProgress,
    Completed,
    Abandoned,
    Paused,
}

impl SessionState {
    pub fn can_submit_answer(&self) -> bool {
        matches!(self, Self::InProgress)
    }

    pub fn is_final(&self) -> bool {
        matches!(self, Self::Completed | Self::Abandoned)
    }
}
```

### LearningMode

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LearningMode {
    MultipleChoice { options: u8 },  // 通常4択
    Typing,
    Listening,
    Speaking,
    FlashCard,
}

impl Default for LearningMode {
    fn default() -> Self {
        Self::MultipleChoice { options: 4 }
    }
}
```

### QuestionType

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum QuestionType {
    WordToMeaning,      // 単語 → 意味
    MeaningToWord,      // 意味 → 単語
    FillInTheBlank,     // 穴埋め
    Synonym,            // 同義語
    Antonym,            // 反義語
    AudioToWord,        // 音声 → 単語
}
```

### QualityRating

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QualityRating(u8);

impl QualityRating {
    pub fn new(value: u8) -> Result<Self, ValueError> {
        if value <= 5 {
            Ok(Self(value))
        } else {
            Err(ValueError::OutOfRange)
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn is_correct(&self) -> bool {
        self.0 >= 3
    }

    pub fn is_perfect(&self) -> bool {
        self.0 == 5
    }
}

// 便利なコンストラクタ
impl QualityRating {
    pub const BLACKOUT: Self = Self(0);
    pub const INCORRECT: Self = Self(2);
    pub const CORRECT_DIFFICULT: Self = Self(3);
    pub const CORRECT: Self = Self(4);
    pub const PERFECT: Self = Self(5);
}
```

### MasteryLevel

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MasteryLevel(f32);  // 0.0 - 1.0

impl MasteryLevel {
    pub fn new(value: f32) -> Result<Self, ValueError> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ValueError::OutOfRange)
        }
    }

    pub fn percentage(&self) -> u8 {
        (self.0 * 100.0) as u8
    }

    pub fn is_mastered(&self) -> bool {
        self.0 >= 0.8
    }

    pub fn level_name(&self) -> &'static str {
        match self.percentage() {
            0..=20 => "Beginner",
            21..=40 => "Elementary",
            41..=60 => "Intermediate",
            61..=80 => "Advanced",
            81..=100 => "Master",
            _ => unreachable!(),
        }
    }
}
```

## Word Management Context の値オブジェクト

### WordText

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WordText(String);

impl WordText {
    pub fn new(text: String) -> Result<Self, ValueError> {
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return Err(ValueError::Empty);
        }

        if trimmed.len() > 100 {
            return Err(ValueError::TooLong);
        }

        // 英単語の基本的な検証
        if !trimmed.chars().all(|c| c.is_alphabetic() || c == '-' || c == '\'' || c == ' ') {
            return Err(ValueError::InvalidCharacters);
        }

        Ok(Self(trimmed.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### Phonetic (IPA)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Phonetic(String);

impl Phonetic {
    pub fn new(ipa: String) -> Result<Self, ValueError> {
        // IPA 文字の基本的な検証
        if ipa.chars().any(|c| !is_valid_ipa_char(c)) {
            return Err(ValueError::InvalidIPA);
        }

        Ok(Self(ipa))
    }

    pub fn empty() -> Self {
        Self(String::new())
    }
}
```

### PartOfSpeech

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PartOfSpeech {
    Noun { countable: bool, uncountable: bool },
    Verb { transitive: bool, intransitive: bool },
    Adjective,
    Adverb,
    Preposition,
    Conjunction,
    Pronoun,
    Interjection,
    Article,
    Unknown,
}

impl PartOfSpeech {
    pub fn abbreviation(&self) -> &'static str {
        match self {
            Self::Noun { .. } => "n.",
            Self::Verb { .. } => "v.",
            Self::Adjective => "adj.",
            Self::Adverb => "adv.",
            Self::Preposition => "prep.",
            Self::Conjunction => "conj.",
            Self::Pronoun => "pron.",
            Self::Interjection => "interj.",
            Self::Article => "art.",
            Self::Unknown => "?",
        }
    }
}
```

### Difficulty

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Difficulty(u8);  // 1-10

impl Difficulty {
    pub fn new(value: u8) -> Result<Self, ValueError> {
        if (1..=10).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ValueError::OutOfRange)
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn to_cefr(&self) -> CefrLevel {
        match self.0 {
            1..=2 => CefrLevel::A1,
            3..=4 => CefrLevel::A2,
            5..=6 => CefrLevel::B1,
            7..=8 => CefrLevel::B2,
            9 => CefrLevel::C1,
            10 => CefrLevel::C2,
            _ => unreachable!(),
        }
    }
}
```

### CefrLevel

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CefrLevel {
    A1,  // Beginner
    A2,  // Elementary
    B1,  // Intermediate
    B2,  // Upper Intermediate
    C1,  // Advanced
    C2,  // Proficient
}

impl CefrLevel {
    pub fn description(&self) -> &'static str {
        match self {
            Self::A1 => "Beginner",
            Self::A2 => "Elementary",
            Self::B1 => "Intermediate",
            Self::B2 => "Upper Intermediate",
            Self::C1 => "Advanced",
            Self::C2 => "Proficient",
        }
    }
}
```

### Category

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Category {
    IELTS,
    TOEIC,
    TOEFL,
    GRE,
    Business,
    Academic,
    General,
    Technical,
}

impl Category {
    pub fn all() -> Vec<Self> {
        vec![
            Self::IELTS,
            Self::TOEIC,
            Self::TOEFL,
            Self::GRE,
            Self::Business,
            Self::Academic,
            Self::General,
            Self::Technical,
        ]
    }
}
```

### Tag

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag(String);

impl Tag {
    pub fn new(tag: String) -> Result<Self, ValueError> {
        let normalized = tag.trim().to_lowercase();

        if normalized.is_empty() {
            return Err(ValueError::Empty);
        }

        if normalized.len() > 50 {
            return Err(ValueError::TooLong);
        }

        // アルファベット、数字、ハイフンのみ許可
        if !normalized.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ValueError::InvalidCharacters);
        }

        Ok(Self(normalized))
    }
}
```

### RelationType

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RelationType {
    Synonym,
    Antonym,
    Collocation,
    SpellingVariant,    // color/colour
    RegionalVariant,    // elevator/lift
    Etymology,          // 語源
    Derivative,         // act -> action
}
```

## User Context の値オブジェクト

### Email

```rust
use regex::Regex;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Self, ValueError> {
        lazy_static! {
            static ref EMAIL_REGEX: Regex = Regex::new(
                r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
            ).unwrap();
        }

        let normalized = email.trim().to_lowercase();

        if EMAIL_REGEX.is_match(&normalized) {
            Ok(Self(normalized))
        } else {
            Err(ValueError::InvalidEmail)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### DisplayName

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DisplayName(String);

impl DisplayName {
    pub fn new(name: String) -> Result<Self, ValueError> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(ValueError::Empty);
        }

        if trimmed.len() > 50 {
            return Err(ValueError::TooLong);
        }

        Ok(Self(trimmed.to_string()))
    }
}
```

### DailyGoal

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DailyGoal(u32);

impl DailyGoal {
    pub fn new(words: u32) -> Result<Self, ValueError> {
        if words == 0 {
            return Err(ValueError::Zero);
        }

        if words > 200 {
            return Err(ValueError::TooLarge);
        }

        Ok(Self(words))
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for DailyGoal {
    fn default() -> Self {
        Self(10)
    }
}
```

### LearningGoal

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LearningGoal {
    test_type: TestType,
    target_score: Option<Score>,
    target_date: Option<Date>,
    description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TestType {
    IELTS,
    TOEIC,
    TOEFL,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Score(f32);

impl Score {
    pub fn new(value: f32) -> Result<Self, ValueError> {
        if value < 0.0 {
            return Err(ValueError::Negative);
        }

        Ok(Self(value))
    }
}
```

## エラー型

```rust
#[derive(Debug, thiserror::Error)]
pub enum ValueError {
    #[error("Value cannot be empty")]
    Empty,

    #[error("Value is too long")]
    TooLong,

    #[error("Value is out of range")]
    OutOfRange,

    #[error("Invalid characters")]
    InvalidCharacters,

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Invalid time format")]
    InvalidTime,

    #[error("Invalid IPA characters")]
    InvalidIPA,

    #[error("Value cannot be zero")]
    Zero,

    #[error("Value is too large")]
    TooLarge,

    #[error("Value cannot be negative")]
    Negative,
}
```

## 値オブジェクトの利点

1. **型安全性**: 文字列や数値の誤用を防ぐ
2. **検証の集約**: 作成時に必ず検証される
3. **表現力**: コードの意図が明確
4. **不変性**: 予期しない変更を防ぐ

## 使用例

```rust
// 正しい使用例
let email = Email::parse("user@example.com".to_string())?;
let difficulty = Difficulty::new(5)?;
let daily_goal = DailyGoal::new(20)?;

// コンパイルエラーになる例
// let email = Email("invalid-email");  // プライベートコンストラクタ
// let difficulty = Difficulty(11);     // 検証を通らない

// 値の比較
let rating1 = QualityRating::new(4)?;
let rating2 = QualityRating::CORRECT;
assert_eq!(rating1, rating2);
```

## 更新履歴

- 2025-07-25: 初版作成
