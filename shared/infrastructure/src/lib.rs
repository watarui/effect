//! Shared infrastructure components for the Effect application.
//!
//! This crate contains database connections, message bus, and other
//! infrastructure.

/// Adds two unsigned 64-bit integers.
///
/// # Examples
///
/// ```
/// use infrastructure::add;
/// assert_eq!(add(2, 2), 4);
/// ```
#[must_use]
pub const fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
