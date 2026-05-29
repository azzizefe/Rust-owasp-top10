// crates/web/src/middleware/request_id.rs

use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use tracing::Instrument;

pub async fn correlation_id(mut request: Request<Body>, next: Next) -> Response {
    // 1. Header'lardan X-Request-Id ara, yoksa yeni üret
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Rand kullanarak u128 hex tabanında benzersiz correlation ID üretilir
            format!("{:x}", rand::random::<u128>())
        });

    let header_val = HeaderValue::from_str(&request_id).unwrap_or_else(|_| HeaderValue::from_static(""));

    // 2. Request Extensions'a ekle (gerekirse handler'lar veya diğer katmanlar okuyabilsin)
    request.extensions_mut().insert(header_val.clone());

    // 3. Tracing span oluştur ve isteği bu span'e bağla
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
    );

    // İsteği bu span context'i altında çalıştır (Log korelasyonu sağlar)
    let mut response = next.run(request).instrument(span).await;

    // 4. Yanıt header'ına X-Request-Id ekle
    response.headers_mut().insert("x-request-id", header_val);

    response
}
