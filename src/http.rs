//! http.rs
//!
//! A simple wrapper around Hyper's HTTP client.

use reqwest::header::CONTENT_LENGTH;

use std::io::Read;
use std::time::Duration;

use reqwest::Client;

/// Builds a customised HTTP client.
pub fn build_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|x| format!("Unable to build cient: {:?}", x))
}

/// Downloads a text file from the specified URL.
pub fn download_text(url: &str) -> Result<String, String> {
    let mut client = build_client()?
        .get(url)
        .send()
        .map_err(|x| format!("Failed to GET resource: {:?}", x))?;

    client
        .text()
        .map_err(|v| format!("Failed to get text from resource: {:?}", v))
}

/// Streams a file from a HTTP server.
pub fn stream_file<F>(url: &str, mut callback: F) -> Result<(), String>
where
    F: FnMut(Vec<u8>, u64) -> (),
{
    let mut client = build_client()?
        .get(url)
        .send()
        .map_err(|x| format!("Failed to GET resource: {:?}", x))?;

    let size = match client.headers().get(CONTENT_LENGTH) {
        Some(ref v) => v
            .to_str()
            .map_err(|x| format!("Content length header was invalid: {:?}", x))?
            .parse()
            .map_err(|x| format!("Failed to parse content length: {:?}", x))?,
        None => 0,
    };

    let mut buf = [0 as u8; 8192];
    loop {
        let len = client
            .read(&mut buf)
            .map_err(|x| format!("Failed to read resource: {:?}", x))?;

        if len == 0 {
            break;
        }

        let buf_copy = &buf[0..len];
        let buf_copy = buf_copy.to_vec();

        callback(buf_copy, size);
    }

    Ok(())
}
