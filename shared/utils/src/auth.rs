use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// JWT Claims结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // 用户ID
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub exp: usize,  // 过期时间
    pub iat: usize,  // 签发时间
    pub nbf: usize,  // 生效时间
    pub iss: String, // 签发者
    pub aud: String, // 受众
}

/// JWT服务
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    audience: String,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
}

impl JwtService {
    pub fn new(
        secret: &str,
        issuer: String,
        audience: String,
        access_token_expiry_hours: i64,
        refresh_token_expiry_days: i64,
    ) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            issuer,
            audience,
            access_token_expiry: Duration::hours(access_token_expiry_hours),
            refresh_token_expiry: Duration::days(refresh_token_expiry_days),
        }
    }

    /// 生成访问令牌
    pub fn generate_access_token(
        &self,
        user_id: &str,
        username: &str,
        email: &str,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = (now + self.access_token_expiry).timestamp() as usize;
        let iat = now.timestamp() as usize;
        let nbf = now.timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            email: email.to_string(),
            roles,
            permissions,
            exp,
            iat,
            nbf,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };

        let header = Header::new(Algorithm::HS256);
        encode(&header, &claims, &self.encoding_key).map_err(Into::into)
    }

    /// 生成刷新令牌
    pub fn generate_refresh_token(&self, user_id: &str) -> Result<String> {
        let now = Utc::now();
        let exp = (now + self.refresh_token_expiry).timestamp() as usize;
        let iat = now.timestamp() as usize;
        let nbf = now.timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            username: String::new(),
            email: String::new(),
            roles: vec![],
            permissions: vec![],
            exp,
            iat,
            nbf,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };

        let header = Header::new(Algorithm::HS256);
        encode(&header, &claims, &self.encoding_key).map_err(Into::into)
    }

    /// 验证令牌
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }

    /// 检查权限
    pub fn check_permission(&self, claims: &Claims, required_permission: &str) -> bool {
        claims
            .permissions
            .contains(&required_permission.to_string())
    }

    /// 检查角色
    pub fn check_role(&self, claims: &Claims, required_role: &str) -> bool {
        claims.roles.contains(&required_role.to_string())
    }
}

/// 密码哈希服务
pub struct PasswordService;

impl PasswordService {
    /// 哈希密码
    pub fn hash_password(password: &str) -> Result<String> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(Into::into)
    }

    /// 验证密码
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        bcrypt::verify(password, hash).map_err(Into::into)
    }

    /// 生成随机密码
    pub fn generate_random_password(length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789\
                                !@#$%^&*";
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// 检查密码强度
    pub fn check_password_strength(password: &str) -> PasswordStrength {
        let mut score = 0;
        let mut feedback = Vec::new();

        // 长度检查
        if password.len() >= 8 {
            score += 1;
        } else {
            feedback.push("密码长度至少8位".to_string());
        }

        // 包含小写字母
        if password.chars().any(|c| c.is_lowercase()) {
            score += 1;
        } else {
            feedback.push("密码应包含小写字母".to_string());
        }

        // 包含大写字母
        if password.chars().any(|c| c.is_uppercase()) {
            score += 1;
        } else {
            feedback.push("密码应包含大写字母".to_string());
        }

        // 包含数字
        if password.chars().any(|c| c.is_numeric()) {
            score += 1;
        } else {
            feedback.push("密码应包含数字".to_string());
        }

        // 包含特殊字符
        if password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
        {
            score += 1;
        } else {
            feedback.push("密码应包含特殊字符".to_string());
        }

        let strength = match score {
            0..=2 => PasswordStrengthLevel::Weak,
            3 => PasswordStrengthLevel::Medium,
            4 => PasswordStrengthLevel::Strong,
            5 => PasswordStrengthLevel::VeryStrong,
            _ => PasswordStrengthLevel::VeryStrong,
        };

        PasswordStrength {
            level: strength,
            score,
            feedback,
        }
    }
}

/// 密码强度
#[derive(Debug, Clone)]
pub struct PasswordStrength {
    pub level: PasswordStrengthLevel,
    pub score: u8,
    pub feedback: Vec<String>,
}

/// 密码强度等级
#[derive(Debug, Clone, PartialEq)]
pub enum PasswordStrengthLevel {
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

/// 权限检查器
pub struct PermissionChecker {
    permissions: HashSet<String>,
}

impl PermissionChecker {
    pub fn new(permissions: Vec<String>) -> Self {
        Self {
            permissions: permissions.into_iter().collect(),
        }
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        permissions.iter().any(|p| self.has_permission(p))
    }

    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|p| self.has_permission(p))
    }
}

/// API密钥生成器
pub struct ApiKeyGenerator;

impl ApiKeyGenerator {
    /// 生成API密钥
    pub fn generate_api_key() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// 生成API密钥
    pub fn generate_secret_key() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..64)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_service() {
        let jwt_service = JwtService::new(
            "test_secret",
            "test_issuer".to_string(),
            "test_audience".to_string(),
            1,
            7,
        );

        let token = jwt_service
            .generate_access_token(
                "user123",
                "testuser",
                "test@example.com",
                vec!["user".to_string()],
                vec!["read".to_string()],
            )
            .unwrap();

        let claims = jwt_service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn test_password_service() {
        let password = "test_password";
        let hash = PasswordService::hash_password(password).unwrap();
        assert!(PasswordService::verify_password(password, &hash).unwrap());
        assert!(!PasswordService::verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_password_strength() {
        let weak_password = "123";
        let strong_password = "MyStr0ng!Pass";

        let weak_strength = PasswordService::check_password_strength(weak_password);
        let strong_strength = PasswordService::check_password_strength(strong_password);

        assert_eq!(weak_strength.level, PasswordStrengthLevel::Weak);
        assert_eq!(strong_strength.level, PasswordStrengthLevel::VeryStrong);
    }
}
