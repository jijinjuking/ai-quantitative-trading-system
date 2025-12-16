use serde::{Deserialize, Serialize};

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: usize,
    pub keep_alive: u64,
    pub request_timeout: u64,
    pub shutdown_timeout: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8081,
            workers: None, // 使用CPU核心数
            max_connections: 10000,
            keep_alive: 60,
            request_timeout: 30,
            shutdown_timeout: 30,
        }
    }
}

impl ServerConfig {
    /// 获取服务器地址
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// 获取工作线程数
    pub fn worker_count(&self) -> usize {
        self.workers.unwrap_or_else(num_cpus::get)
    }

    /// 验证配置
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.port == 0 {
            return Err(anyhow::anyhow!("Server port cannot be 0"));
        }

        if self.max_connections == 0 {
            return Err(anyhow::anyhow!("Max connections must be greater than 0"));
        }

        if self.request_timeout == 0 {
            return Err(anyhow::anyhow!("Request timeout must be greater than 0"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8081);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::default();

        // 端口为0应该失败
        config.port = 0;
        assert!(config.validate().is_err());

        // 恢复有效端口
        config.port = 8081;
        assert!(config.validate().is_ok());

        // 最大连接数为0应该失败
        config.max_connections = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_server_address() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 9000,
            ..Default::default()
        };

        assert_eq!(config.address(), "127.0.0.1:9000");
    }
}
