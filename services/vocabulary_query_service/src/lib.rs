//! Vocabulary Query Service Library
//!
//! Read Model に対する基本的な読み取り操作を提供

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ports;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
