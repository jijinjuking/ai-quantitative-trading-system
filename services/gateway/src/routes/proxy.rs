use axum::{
    extract::{Path, Query, Request, State},
    http::{Method, StatusCode},
    response::Response,
};
use std::collections::HashMap;
use tracing::{debug, error};

use crate::{
    services::ServiceProxy,
    state::AppState,
};

/// GET请求代理
pub async fn proxy_get(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    Query(query): Query<HashMap<String, String>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("Proxying GET request");
    ServiceProxy::proxy_request(State(state), Path(params), Query(query), request).await
}

/// POST请求代理
pub async fn proxy_post(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    Query(query): Query<HashMap<String, String>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("Proxying POST request");
    ServiceProxy::proxy_request(State(state), Path(params), Query(query), request).await
}

/// PUT请求代理
pub async fn proxy_put(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    Query(query): Query<HashMap<String, String>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("Proxying PUT request");
    ServiceProxy::proxy_request(State(state), Path(params), Query(query), request).await
}

/// DELETE请求代理
pub async fn proxy_delete(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    Query(query): Query<HashMap<String, String>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("Proxying DELETE request");
    ServiceProxy::proxy_request(State(state), Path(params), Query(query), request).await
}

/// PATCH请求代理
pub async fn proxy_patch(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    Query(query): Query<HashMap<String, String>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("Proxying PATCH request");
    ServiceProxy::proxy_request(State(state), Path(params), Query(query), request).await
}

/// WebSocket代理
pub async fn proxy_websocket(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    request: Request,
) -> Result<Response, StatusCode> {
    debug!("Proxying WebSocket request");
    ServiceProxy::proxy_websocket(State(state), Path(params), request).await
}