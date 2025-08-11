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

    /// Common types module
    #[allow(clippy::all)]
    #[allow(warnings)]
    pub mod common {
        tonic::include_proto!("effect.common");
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
