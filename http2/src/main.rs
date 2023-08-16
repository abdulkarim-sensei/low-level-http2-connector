use bytes::BytesMut;
use hyper::body::to_bytes;
use hyper::client::{Client, HttpConnector};
use hyper::header::HeaderMap;
use hyper::{Body, Request, StatusCode};
use hyper_tls::HttpsConnector;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

pub struct HttpClient2 {
    client: Arc<Mutex<Client<HttpsConnector<HttpConnector>>>>,
    base_url: Url,
    request_buffer: BytesMut,
    headers: HeaderMap,
    response_buffer: BytesMut,
    // Add more fields as needed
}

impl HttpClient2 {
    pub fn new(client: Arc<Mutex<Client<HttpsConnector<HttpConnector>>>>, base_url: Url) -> Self {
        Self {
            client,
            base_url,
            request_buffer: BytesMut::with_capacity(1024),
            headers: HeaderMap::new(),
            response_buffer: BytesMut::new(),
        }
    }

    pub async fn send_request(&mut self) -> Result<BytesMut> {
        let full_url = self.base_url.clone().to_string();
        let request = Request::builder()
            .method("POST")
            .uri(full_url)
            .body(Body::from(self.request_buffer.clone().freeze()))
            .unwrap();

        let response_body = {
            let client = self.client.lock().await;
            let response = client
                .request(request)
                .await
                .map_err(|e| Error::new(ErrorKind::Other, e))?;

            if response.status() != StatusCode::OK {
                return Err(Error::new(ErrorKind::Other, "Request failed"));
            }

            let bytes = to_bytes(response.into_body())
                .await
                .map_err(|e| Error::new(ErrorKind::Other, e))?;
            BytesMut::from(bytes.as_ref())
        };

        self.populate_response_buffer(response_body.clone()); // Populate response buffer
        Ok(response_body)
    }

    fn populate_response_buffer(&mut self, response: BytesMut) {
        self.response_buffer.extend_from_slice(&response);
    }
}

impl Write for HttpClient2 {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.request_buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Read for HttpClient2 {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.response_buffer.is_empty() {
            return Ok(0); // Nothing to read yet
        }

        let bytes_to_copy = buf.len().min(self.response_buffer.len());
        buf[..bytes_to_copy].copy_from_slice(&self.response_buffer[..bytes_to_copy]);
        self.response_buffer = self.response_buffer.split_off(bytes_to_copy);

        Ok(bytes_to_copy)
    }
}

fn main() {
    // Could perform a test here, through httpBin or similar.
}
