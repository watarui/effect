//! ID 型のユーティリティ
//!
//! データベース操作で使用される ID 型のラッパーを提供

use std::fmt;

/// バイト配列 ID のラッパー
///
/// `Vec<u8>` を Display トレイトを実装したラッパー型で包む
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    /// 新しい `Bytes` を作成
    #[must_use]
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// 内部のバイト配列への参照を取得
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// 内部のバイト配列を取得
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // UUID として解釈できる場合は UUID 形式で表示
        if self.0.len() == 16
            && let Ok(uuid) = uuid::Uuid::from_slice(&self.0)
        {
            return write!(f, "{uuid}");
        }

        // それ以外は16進数表記
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for Bytes {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_display() {
        // UUID の場合
        let uuid = uuid::Uuid::new_v4();
        let bytes_id = Bytes::new(uuid.as_bytes().to_vec());
        assert_eq!(bytes_id.to_string(), uuid.to_string());

        // 通常のバイト配列の場合
        let bytes_id = Bytes::new(vec![1, 2, 3, 4]);
        assert_eq!(bytes_id.to_string(), "0x01020304");
    }

    #[test]
    fn test_bytes_conversions() {
        let bytes = vec![1, 2, 3, 4];
        let bytes_id = Bytes::from(bytes.clone());

        assert_eq!(bytes_id.as_bytes(), &bytes);
        assert_eq!(bytes_id.into_bytes(), bytes);
    }
}
