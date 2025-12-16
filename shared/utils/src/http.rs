use anyhow::Result;
use reqwest::{Client, ClientBuilder, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

/// HTTP客户端配置
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub user_agent: String,
    pub max_redirects: usize,
    pub default_headers: HashMap<String, String>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            user_agent: "TradingPlatform/1.0".to_string(),
            max_redirects: 10,
            default_headers: HashMap::new(),
        }
    }
}

/// HTTP客户端包装器
pub struct HttpClient {
    client: Client,
    config: HttpClientConfig,
}

impl HttpClient {
    /// 创建新的HTTP客户端
    pub fn new(config: HttpClientConfig) -> Result<Self> {
        let mut builder = ClientBuilder::new()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .user_agent(&config.user_agent)
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects));

        // 添加默认头部
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in &config.default_headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes())?,
                reqwest::header::HeaderValue::from_str(value)?,
            );
        }
        builder = builder.default_headers(headers);

        let client = builder.build()?;

        Ok(Self { client, config })
    }

    /// 创建默认HTTP客户端
    pub fn default() -> Result<Self> {
        Self::new(HttpClientConfig::default())
    }

    /// GET请求
    pub async fn get(&self, url: &str) -> Result<Response> {
        let response = self.client.get(url).send().await?;
        Ok(response)
    }

    /// GET请求并解析JSON
    pub async fn get_json<T>(&self, url: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.get(url).await?;
        let json = response.json::<T>().await?;
        Ok(json)
    }

    /// POST请求
    pub async fn post(&self, url: &str, body: &impl Serialize) -> Result<Response> {
        let response = self.client.post(url).json(body).send().await?;
        Ok(response)
    }

    /// POST请求并解析JSON
    pub async fn post_json<T, U>(&self, url: &str, body: &T) -> Result<U>
    where
        T: Serialize,
        U: for<'de> Deserialize<'de>,
    {
        let response = self.post(url, body).await?;
        let json = response.json::<U>().await?;
        Ok(json)
    }

    /// PUT请求
    pub async fn put(&self, url: &str, body: &impl Serialize) -> Result<Response> {
        let response = self.client.put(url).json(body).send().await?;
        Ok(response)
    }

    /// PUT请求并解析JSON
    pub async fn put_json<T, U>(&self, url: &str, body: &T) -> Result<U>
    where
        T: Serialize,
        U: for<'de> Deserialize<'de>,
    {
        let response = self.put(url, body).await?;
        let json = response.json::<U>().await?;
        Ok(json)
    }

    /// DELETE请求
    pub async fn delete(&self, url: &str) -> Result<Response> {
        let response = self.client.delete(url).send().await?;
        Ok(response)
    }

    /// DELETE请求并解析JSON
    pub async fn delete_json<T>(&self, url: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.delete(url).await?;
        let json = response.json::<T>().await?;
        Ok(json)
    }

    /// 带认证的GET请求
    pub async fn get_with_auth(&self, url: &str, token: &str) -> Result<Response> {
        let response = self
            .client
            .get(url)
            .bearer_auth(token)
            .send()
            .await?;
        Ok(response)
    }

    /// 带认证的POST请求
    pub async fn post_with_auth(&self, url: &str, body: &impl Serialize, token: &str) -> Result<Response> {
        let response = self
            .client
            .post(url)
            .json(body)
            .bearer_auth(token)
            .send()
            .await?;
        Ok(response)
    }

    /// 带自定义头部的请求
    pub async fn request_with_headers(
        &self,
        method: reqwest::Method,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<&impl Serialize>,
    ) -> Result<Response> {
        let mut request = self.client.request(method, url);

        // 添加头部
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        // 添加请求体
        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;
        Ok(response)
    }

    /// 下载文件
    pub async fn download_file(&self, url: &str) -> Result<bytes::Bytes> {
        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes)
    }

    /// 上传文件
    pub async fn upload_file(&self, url: &str, file_data: Vec<u8>, filename: &str) -> Result<Response> {
        let part = reqwest::multipart::Part::bytes(file_data)
            .file_name(filename.to_string())
            .mime_str("application/octet-stream")?;

        let form = reqwest::multipart::Form::new().part("file", part);

        let response = self.client.post(url).multipart(form).send().await?;
        Ok(response)
    }
}

/// HTTP重试客户端
pub struct RetryHttpClient {
    client: HttpClient,
    max_retries: usize,
    retry_delay: Duration,
}

impl RetryHttpClient {
    pub fn new(client: HttpClient, max_retries: usize, retry_delay: Duration) -> Self {
        Self {
            client,
            max_retries,
            retry_delay,
        }
    }

    /// 带重试的GET请求
    pub async fn get_with_retry(&self, url: &str) -> Result<Response> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.client.get(url).await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status().is_server_error() && attempt < self.max_retries {
                        // 服务器错误，重试
                        tokio::time::sleep(self.retry_delay).await;
                        continue;
                    } else {
                        return Ok(response);
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        tokio::time::sleep(self.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// 带重试的POST请求
    pub async fn post_with_retry(&self, url: &str, body: &impl Serialize) -> Result<Response> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.client.post(url, body).await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status().is_server_error() && attempt < self.max_retries {
                        // 服务器错误，重试
                        tokio::time::sleep(self.retry_delay).await;
                        continue;
                    } else {
                        return Ok(response);
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        tokio::time::sleep(self.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }
}

/// HTTP响应工具
pub struct ResponseUtils;

impl ResponseUtils {
    /// 检查响应状态
    pub fn check_status(response: &Response) -> Result<()> {
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "HTTP request failed with status: {}",
                response.status()
            ))
        }
    }

    /// 获取响应头
    pub fn get_header(response: &Response, name: &str) -> Option<String> {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string())
    }

    /// 获取内容类型
    pub fn get_content_type(response: &Response) -> Option<String> {
        Self::get_header(response, "content-type")
    }

    /// 检查是否为JSON响应
    pub fn is_json(response: &Response) -> bool {
        Self::get_content_type(response)
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    /// 安全解析JSON响应
    pub async fn safe_json<T>(response: Response) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !Self::is_json(&response) {
            return Err(anyhow::anyhow!("Response is not JSON"));
        }

        Self::check_status(&response)?;
        let json = response.json::<T>().await?;
        Ok(json)
    }

    /// 获取响应文本
    pub async fn get_text(response: Response) -> Result<String> {
        Self::check_status(&response)?;
        let text = response.text().await?;
        Ok(text)
    }

    /// 获取响应字节
    pub async fn get_bytes(response: Response) -> Result<bytes::Bytes> {
        Self::check_status(&response)?;
        let bytes = response.bytes().await?;
        Ok(bytes)
    }
}

/// URL工具
pub struct UrlUtils;

impl UrlUtils {
    /// 构建查询字符串
    pub fn build_query_string(params: &HashMap<String, String>) -> String {
        if params.is_empty() {
            return String::new();
        }

        let query: Vec<String> = params
            .iter()
            .map(|(key, value)| format!("{}={}", urlencoding::encode(key), urlencoding::encode(value)))
            .collect();

        format!("?{}", query.join("&"))
    }

    /// 构建URL
    pub fn build_url(base: &str, path: &str, params: Option<&HashMap<String, String>>) -> String {
        let mut url = format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'));
        
        if let Some(params) = params {
            url.push_str(&Self::build_query_string(params));
        }

        url
    }

    /// 解析查询字符串
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        for pair in query.trim_start_matches('?').split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if let (Ok(key), Ok(value)) = (urlencoding::decode(key), urlencoding::decode(value)) {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }

        params
    }

    /// 验证URL格式
    pub fn is_valid_url(url: &str) -> bool {
        reqwest::Url::parse(url).is_ok()
    }

    /// 获取域名
    pub fn get_domain(url: &str) -> Option<String> {
        reqwest::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
    }
}

/// 超时工具
pub struct TimeoutUtils;

impl TimeoutUtils {
    /// 带超时的异步操作
    pub async fn with_timeout<F, T>(future: F, duration: Duration) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        match timeout(duration, future).await {
            Ok(result) => result,
            Err(_) => Err(anyhow::anyhow!("Operation timed out")),
        }
    }

    /// 带超时的HTTP请求
    pub async fn http_with_timeout<F>(
        request: F,
        duration: Duration,
    ) -> Result<Response>
    where
        F: std::future::Future<Output = Result<Response>>,
    {
        Self::with_timeout(request, duration).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_utils() {
        let mut params = HashMap::new();
        params.insert("key1".to_string(), "value1".to_string());
        params.insert("key2".to_string(), "value with spaces".to_string());

        let query = UrlUtils::build_query_string(&params);
        assert!(query.contains("key1=value1"));
        assert!(query.contains("key2=value%20with%20spaces"));

        let url = UrlUtils::build_url("https://api.example.com", "/v1/data", Some(&params));
        assert!(url.starts_with("https://api.example.com/v1/data?"));

        assert!(UrlUtils::is_valid_url("https://example.com"));
        assert!(!UrlUtils::is_valid_url("not-a-url"));

        assert_eq!(
            UrlUtils::get_domain("https://api.example.com/path"),
            Some("api.example.com".to_string())
        );
    }

    #[test]
    fn test_query_string_parsing() {
        let query = "?key1=value1&key2=value%20with%20spaces";
        let params = UrlUtils::parse_query_string(query);
        
        assert_eq!(params.get("key1"), Some(&"value1".to_string()));
        assert_eq!(params.get("key2"), Some(&"value with spaces".to_string()));
    }

    #[tokio::test]
    async fn test_http_client_creation() {
        let config = HttpClientConfig::default();
        let client = HttpClient::new(config);
        assert!(client.is_ok());
    }
}