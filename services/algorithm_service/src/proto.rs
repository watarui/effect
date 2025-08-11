//! Proto generated code

/// Algorithm service definitions
pub mod effect {
    /// Services module
    pub mod services {
        /// Algorithm service module
        #[allow(clippy::all)]
        #[allow(warnings)]
        pub mod algorithm {
            tonic::include_proto!("effect.services.algorithm");
        }
    }

    /// Event store module
    #[allow(clippy::all)]
    #[allow(warnings)]
    pub mod event_store {
        tonic::include_proto!("effect.event_store");
    }

    /// Common types module
    pub use shared_kernel::proto::effect::common;

    /// Learning types module (CorrectnessJudgment など)
    #[allow(clippy::all)]
    #[allow(warnings)]
    pub mod learning {
        tonic::include_proto!("effect.common");

        // CorrectnessJudgment の定義
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration,
        )]
        #[repr(i32)]
        pub enum CorrectnessJudgment {
            Unspecified      = 0,
            Incorrect        = 1,
            PartiallyCorrect = 2,
            Correct          = 3,
            Perfect          = 4,
        }

        impl CorrectnessJudgment {
            /// String representation of the enum field names
            pub fn as_str_name(&self) -> &'static str {
                match self {
                    CorrectnessJudgment::Unspecified => "CORRECTNESS_JUDGMENT_UNSPECIFIED",
                    CorrectnessJudgment::Incorrect => "CORRECTNESS_JUDGMENT_INCORRECT",
                    CorrectnessJudgment::PartiallyCorrect => {
                        "CORRECTNESS_JUDGMENT_PARTIALLY_CORRECT"
                    },
                    CorrectnessJudgment::Correct => "CORRECTNESS_JUDGMENT_CORRECT",
                    CorrectnessJudgment::Perfect => "CORRECTNESS_JUDGMENT_PERFECT",
                }
            }
        }
    }

    /// `CorrectnessJudgment` を common モジュールに再エクスポート
    pub mod common_ext {
        pub use super::{common::*, learning::CorrectnessJudgment};
    }

    /// Events module
    pub mod events {
        /// Algorithm events module
        #[allow(clippy::all)]
        #[allow(warnings)]
        pub mod algorithm {
            tonic::include_proto!("effect.events.algorithm");
        }

        /// Learning events module
        #[allow(clippy::all)]
        #[allow(warnings)]
        pub mod learning {
            tonic::include_proto!("effect.events.learning");
        }
    }
}
