use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, Method},
    middleware::Next,
    response::Response,
};

use crate::config::CorsConfig;

/// CORS中间件
#[derive(Clone)]
pub struct CorsMiddleware {
    config: CorsConfig,
}

impl CorsMiddleware {
    pub fn new(config: CorsConfig) -> Self {
        Self { config }
    }
}

/// CORS中间件处理函数
pub async fn cors_middleware(
    config: CorsConfig,
    request: Request,
    next: Next,
) -> Response {
    // 如果CORS未启用，直接通过
    if !config.enabled {
        return next.run(request).await;
    }

    let method = request.method().clone();
    let origin = request.headers().get("origin").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    // 处理预检请求
    if method == Method::OPTIONS {
        let headers = request.headers();
        return handle_preflight_request(&config, origin.as_deref(), headers);
    }

    // 执行实际请求
    let mut response = next.run(request).await;

    // 添加CORS头部
    add_cors_headers(&mut response, &config, origin.as_deref());

    response
}

/// 处理预检请求
fn handle_preflight_request(
    config: &CorsConfig,
    origin: Option<&str>,
    headers: &HeaderMap,
) -> Response {
    let mut response = Response::builder()
        .status(200)
        .body(axum::body::Body::empty())
        .unwrap();

    // 添加CORS头部
    add_cors_headers(&mut response, config, origin);

    // 添加预检特定头部
    if let Some(requested_method) = headers.get("access-control-request-method") {
        if let Ok(method_str) = requested_method.to_str() {
            if config.allowed_methods.contains(&method_str.to_string()) {
                response.headers_mut().insert(
                    HeaderName::from_static("access-control-allow-methods"),
                    HeaderValue::from_str(&config.allowed_methods.join(", ")).unwrap(),
                );
            }
        }
    }

    if let Some(requested_headers) = headers.get("access-control-request-headers") {
        if let Ok(headers_str) = requested_headers.to_str() {
            let requested: Vec<&str> = headers_str.split(',').map(|s| s.trim()).collect();
            let allowed: Vec<String> = requested
                .into_iter()
                .filter(|h| config.allowed_headers.contains(&h.to_string()))
                .map(|s| s.to_string())
                .collect();

            if !allowed.is_empty() {
                response.headers_mut().insert(
                    HeaderName::from_static("access-control-allow-headers"),
                    HeaderValue::from_str(&allowed.join(", ")).unwrap(),
                );
            }
        }
    }

    response
}

/// 添加CORS头部
fn add_cors_headers(response: &mut Response, config: &CorsConfig, origin: Option<&str>) {
    let headers = response.headers_mut();

    // Access-Control-Allow-Origin
    if let Some(origin) = origin {
        if config.allowed_origins.contains(&"*".to_string()) || 
           config.allowed_origins.contains(&origin.to_string()) {
            headers.insert(
                HeaderName::from_static("access-control-allow-origin"),
                HeaderValue::from_str(origin).unwrap(),
            );
        }
    } else if config.allowed_origins.contains(&"*".to_string()) {
        headers.insert(
            HeaderName::from_static("access-control-allow-origin"),
            HeaderValue::from_static("*"),
        );
    }

    // Access-Control-Allow-Credentials
    if !config.allowed_origins.contains(&"*".to_string()) {
        headers.insert(
            HeaderName::from_static("access-control-allow-credentials"),
            HeaderValue::from_static("true"),
        );
    }

    // Access-Control-Max-Age
    headers.insert(
        HeaderName::from_static("access-control-max-age"),
        HeaderValue::from_str(&config.max_age.to_string()).unwrap(),
    );

    // Access-Control-Expose-Headers
    let expose_headers = vec![
        "x-request-id",
        "x-ratelimit-remaining",
        "x-ratelimit-reset",
    ];
    headers.insert(
        HeaderName::from_static("access-control-expose-headers"),
        HeaderValue::from_str(&expose_headers.join(", ")).unwrap(),
    );
}

/// CORS配置构建器
pub struct CorsConfigBuilder {
    config: CorsConfig,
}

impl CorsConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: CorsConfig::default(),
        }
    }

    pub fn allow_origin(mut self, origin: &str) -> Self {
        if !self.config.allowed_origins.contains(&origin.to_string()) {
            self.config.allowed_origins.push(origin.to_string());
        }
        self
    }

    pub fn allow_origins(mut self, origins: Vec<&str>) -> Self {
        for origin in origins {
            if !self.config.allowed_origins.contains(&origin.to_string()) {
                self.config.allowed_origins.push(origin.to_string());
            }
        }
        self
    }

    pub fn allow_method(mut self, method: &str) -> Self {
        if !self.config.allowed_methods.contains(&method.to_string()) {
            self.config.allowed_methods.push(method.to_string());
        }
        self
    }

    pub fn allow_methods(mut self, methods: Vec<&str>) -> Self {
        for method in methods {
            if !self.config.allowed_methods.contains(&method.to_string()) {
                self.config.allowed_methods.push(method.to_string());
            }
        }
        self
    }

    pub fn allow_header(mut self, header: &str) -> Self {
        if !self.config.allowed_headers.contains(&header.to_string()) {
            self.config.allowed_headers.push(header.to_string());
        }
        self
    }

    pub fn allow_headers(mut self, headers: Vec<&str>) -> Self {
        for header in headers {
            if !self.config.allowed_headers.contains(&header.to_string()) {
                self.config.allowed_headers.push(header.to_string());
            }
        }
        self
    }

    pub fn max_age(mut self, max_age: u64) -> Self {
        self.config.max_age = max_age;
        self
    }

    pub fn build(self) -> CorsConfig {
        self.config
    }
}

impl Default for CorsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_config_builder() {
        let config = CorsConfigBuilder::new()
            .allow_origin("https://example.com")
            .allow_origins(vec!["https://app.example.com", "https://admin.example.com"])
            .allow_method("GET")
            .allow_methods(vec!["POST", "PUT", "DELETE"])
            .allow_header("Content-Type")
            .allow_headers(vec!["Authorization", "X-Requested-With"])
            .max_age(7200)
            .build();

        assert!(config.allowed_origins.contains(&"https://example.com".to_string()));
        assert!(config.allowed_origins.contains(&"https://app.example.com".to_string()));
        assert!(config.allowed_methods.contains(&"GET".to_string()));
        assert!(config.allowed_methods.contains(&"POST".to_string()));
        assert!(config.allowed_headers.contains(&"Content-Type".to_string()));
        assert!(config.allowed_headers.contains(&"Authorization".to_string()));
        assert_eq!(config.max_age, 7200);
    }

    #[test]
    fn test_cors_origin_matching() {
        let config = CorsConfig {
            enabled: true,
            allowed_origins: vec!["https://example.com".to_string(), "https://app.example.com".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
            max_age: 3600,
        };

        // 测试允许的源
        assert!(config.allowed_origins.contains(&"https://example.com".to_string()));
        assert!(config.allowed_origins.contains(&"https://app.example.com".to_string()));
        
        // 测试不允许的源
        assert!(!config.allowed_origins.contains(&"https://malicious.com".to_string()));
    }

    #[test]
    fn test_wildcard_origin() {
        let config = CorsConfig {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
            max_age: 3600,
        };

        assert!(config.allowed_origins.contains(&"*".to_string()));
    }
}