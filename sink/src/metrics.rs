use lazy_static::lazy_static;
use prometheus::{register_gauge, Gauge, TextEncoder, Encoder};
use axum::response::Response;
use axum::http::{StatusCode, header};

lazy_static! {
    pub static ref CURRENT_BLOCK_NUMBER: Gauge = register_gauge!(
        "sink_current_block_number",
        "Current block number being processed by the sink"
    )
    .expect("Failed to create current block number gauge");
}

pub async fn metrics_handler() -> Response<String> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, encoder.format_type())
        .body(String::from_utf8(buffer).unwrap())
        .unwrap()
}
