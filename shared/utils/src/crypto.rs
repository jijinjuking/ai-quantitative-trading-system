use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Result};
use base64::prelude::*;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// 加密服务
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    /// 创建新的加密服务实例
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    /// 从密码派生密钥
    pub fn from_password(password: &str, salt: &[u8]) -> Self {
        let key = Self::derive_key(password, salt);
        Self::new(&key)
    }

    /// 加密数据
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // 将nonce和密文组合并编码为base64
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(base64::prelude::BASE64_STANDARD.encode(result))
    }

    /// 解密数据
    pub fn decrypt(&self, ciphertext: &str) -> Result<String> {
        let data = base64::prelude::BASE64_STANDARD
            .decode(ciphertext)
            .map_err(|e| anyhow!("Base64 decode failed: {}", e))?;

        if data.len() < 12 {
            return Err(anyhow!("Invalid ciphertext length"));
        }

        let (nonce_bytes, ciphertext_bytes) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext_bytes)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| anyhow!("UTF-8 decode failed: {}", e))
    }

    /// 从密码派生密钥
    fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
        use pbkdf2::pbkdf2_hmac;
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
        key
    }
}

/// 哈希服务
pub struct HashService;

impl HashService {
    /// SHA256哈希
    pub fn sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// SHA256哈希字符串
    pub fn sha256_string(data: &str) -> String {
        Self::sha256(data.as_bytes())
    }

    /// HMAC-SHA256签名
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<String> {
        let mut mac = <HmacSha256 as hmac::Mac>::new_from_slice(key)
            .map_err(|e| anyhow!("Invalid key length: {}", e))?;
        mac.update(data);
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }

    /// HMAC-SHA256签名字符串
    pub fn hmac_sha256_string(key: &str, data: &str) -> Result<String> {
        Self::hmac_sha256(key.as_bytes(), data.as_bytes())
    }

    /// 验证HMAC-SHA256签名
    pub fn verify_hmac_sha256(key: &[u8], data: &[u8], signature: &str) -> Result<bool> {
        let expected = Self::hmac_sha256(key, data)?;
        Ok(expected == signature)
    }
}

/// 随机数生成器
pub struct RandomGenerator;

impl RandomGenerator {
    /// 生成随机字节
    pub fn random_bytes(length: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut bytes = vec![0u8; length];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }

    /// 生成随机字符串
    pub fn random_string(length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// 生成随机数字字符串
    pub fn random_numeric_string(length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"0123456789";
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// 生成UUID
    pub fn generate_uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// 生成盐值
    pub fn generate_salt() -> [u8; 32] {
        let mut salt = [0u8; 32];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut salt);
        salt
    }
}

/// 数字签名服务
pub struct SignatureService {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
}

impl SignatureService {
    /// 创建新的签名服务
    pub fn new() -> Self {
        // 这里应该使用真正的密钥对生成
        // 为了简化，我们使用固定的密钥
        Self {
            private_key: vec![0u8; 32],
            public_key: vec![0u8; 32],
        }
    }

    /// 签名数据
    pub fn sign(&self, data: &[u8]) -> Result<String> {
        // 使用HMAC作为简单的签名方案
        HashService::hmac_sha256(&self.private_key, data)
    }

    /// 验证签名
    pub fn verify(&self, data: &[u8], signature: &str) -> Result<bool> {
        let expected = self.sign(data)?;
        Ok(expected == signature)
    }
}

/// Base64编码工具
pub struct Base64;

impl Base64 {
    /// 编码
    pub fn encode(data: &[u8]) -> String {
        base64::prelude::BASE64_STANDARD.encode(data)
    }

    /// 编码字符串
    pub fn encode_string(data: &str) -> String {
        Self::encode(data.as_bytes())
    }

    /// 解码
    pub fn decode(data: &str) -> Result<Vec<u8>> {
        base64::prelude::BASE64_STANDARD
            .decode(data)
            .map_err(|e| anyhow!("Base64 decode failed: {}", e))
    }

    /// 解码为字符串
    pub fn decode_string(data: &str) -> Result<String> {
        let bytes = Self::decode(data)?;
        String::from_utf8(bytes).map_err(|e| anyhow!("UTF-8 decode failed: {}", e))
    }
}

/// Hex编码工具
pub struct Hex;

impl Hex {
    /// 编码
    pub fn encode(data: &[u8]) -> String {
        hex::encode(data)
    }

    /// 编码字符串
    pub fn encode_string(data: &str) -> String {
        Self::encode(data.as_bytes())
    }

    /// 解码
    pub fn decode(data: &str) -> Result<Vec<u8>> {
        hex::decode(data).map_err(|e| anyhow!("Hex decode failed: {}", e))
    }

    /// 解码为字符串
    pub fn decode_string(data: &str) -> Result<String> {
        let bytes = Self::decode(data)?;
        String::from_utf8(bytes).map_err(|e| anyhow!("UTF-8 decode failed: {}", e))
    }
}

/// 密钥派生函数
pub struct KeyDerivation;

impl KeyDerivation {
    /// PBKDF2密钥派生
    pub fn pbkdf2(password: &str, salt: &[u8], iterations: u32, key_length: usize) -> Vec<u8> {
        use pbkdf2::pbkdf2_hmac;
        let mut key = vec![0u8; key_length];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
        key
    }

    /// 从密码生成AES密钥
    pub fn password_to_aes_key(password: &str, salt: &[u8]) -> [u8; 32] {
        let key_bytes = Self::pbkdf2(password, salt, 100_000, 32);
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_service() {
        let key = [0u8; 32];
        let service = EncryptionService::new(&key);

        let plaintext = "Hello, World!";
        let ciphertext = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_hash_service() {
        let data = "test data";
        let hash1 = HashService::sha256_string(data);
        let hash2 = HashService::sha256_string(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_hmac() {
        let key = "secret_key";
        let data = "test data";

        let signature = HashService::hmac_sha256_string(key, data).unwrap();
        let is_valid =
            HashService::verify_hmac_sha256(key.as_bytes(), data.as_bytes(), &signature).unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_random_generator() {
        let random_string = RandomGenerator::random_string(16);
        assert_eq!(random_string.len(), 16);

        let random_numeric = RandomGenerator::random_numeric_string(8);
        assert_eq!(random_numeric.len(), 8);
        assert!(random_numeric.chars().all(|c| c.is_numeric()));
    }

    #[test]
    fn test_base64() {
        let data = "Hello, World!";
        let encoded = Base64::encode_string(data);
        let decoded = Base64::decode_string(&encoded).unwrap();

        assert_eq!(data, decoded);
    }
}
