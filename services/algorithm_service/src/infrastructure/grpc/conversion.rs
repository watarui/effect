//! 型変換ヘルパー関数
//!
//! gRPC proto（u32）とデータベース（i32）間の必要最小限の型変換

use tonic::Status;

/// i32 から u32 への安全な変換（proto用）
///
/// # Errors
///
/// 値が負の場合、`Status::invalid_argument` を返します
#[inline]
pub fn i32_to_u32(value: i32) -> Result<u32, Status> {
    u32::try_from(value).map_err(|_| {
        Status::invalid_argument(format!("Negative value {value} cannot be converted to u32"))
    })
}

/// u32 から i32 への安全な変換（DB用）
///
/// # Errors
///
/// 値が `i32::MAX` を超える場合、`Status::invalid_argument` を返します
#[inline]
pub fn u32_to_i32(value: u32) -> Result<i32, Status> {
    i32::try_from(value)
        .map_err(|_| Status::invalid_argument(format!("Value {value} exceeds i32::MAX")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i32_to_u32() {
        assert_eq!(i32_to_u32(100).unwrap(), 100);
        assert_eq!(i32_to_u32(0).unwrap(), 0);
        assert_eq!(i32_to_u32(i32::MAX).unwrap(), i32::MAX as u32);
        assert!(i32_to_u32(-1).is_err());
    }

    #[test]
    fn test_u32_to_i32() {
        assert_eq!(u32_to_i32(100).unwrap(), 100);
        assert_eq!(u32_to_i32(0).unwrap(), 0);
        assert_eq!(u32_to_i32(i32::MAX as u32).unwrap(), i32::MAX);
        assert!(u32_to_i32(u32::MAX).is_err());
    }
}
