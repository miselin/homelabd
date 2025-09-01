use std::{convert::Infallible, fs, net::SocketAddr, path::PathBuf};

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use log::{info, warn};
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::config::Config;
use crate::metrics;

const BINARY_PATH: &str = "/proc/self/exe"; // For self-serve

pub struct HttpServer {
    config: Arc<Config>,
}

impl HttpServer {
    pub fn new(config: Arc<Config>) -> Self {
        HttpServer { config }
    }

    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::new(self.config.http_bind_ip, self.config.http_port);

        let server = TcpListener::bind(&addr).await?;
        info!("HTTP server listening on http://{}", addr);

        let service = {
            let this = Arc::clone(&self);
            service_fn(move |req: Request<hyper::body::Incoming>| {
                let this = Arc::clone(&this);
                async move { this.route(req).await }
            })
        };

        loop {
            let (stream, _) = server.accept().await?;

            let io = TokioIo::new(stream);
            let service = service.clone();

            tokio::task::spawn(async move {
                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    warn!("Failed to serve connection: {}", e);
                }
            });
        }
    }

    async fn route(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, Infallible> {
        match req.uri().path() {
            "/metrics" => {
                let encoder = TextEncoder::new();
                let metric_families = metrics::gather();
                let mut buffer = Vec::new();
                encoder.encode(&metric_families, &mut buffer).unwrap();
                Ok(Response::builder()
                    .header("Content-Type", "text/plain; version=0.0.4")
                    .body(Full::new(Bytes::from(buffer)))
                    .unwrap())
            }
            "/homelabd" => {
                let bin =
                    fs::read(PathBuf::from(BINARY_PATH)).unwrap_or_else(|_| b"error".to_vec());
                Ok(Response::builder()
                    .header("Content-Type", "application/octet-stream")
                    .body(Full::new(Bytes::from(bin)))
                    .unwrap())
            }
            _ => Ok(Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("Not Found")))
                .unwrap()),
        }
    }
}
