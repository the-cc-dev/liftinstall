//! http.rs
//!
//! A simple wrapper around Hyper's HTTP client.

use hyper::header::ContentLength;

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
    let mut client = match build_client()?.get(url).send() {
        Ok(v) => v,
        Err(v) => return Err(format!("Failed to GET resource: {:?}", v)),
    };

    client
        .text()
        .map_err(|v| format!("Failed to get text from resource: {:?}", v))
}

/// Streams a file from a HTTP server.
pub fn stream_file<F>(url: &str, mut callback: F) -> Result<(), String>
where
    F: FnMut(Vec<u8>, u64) -> (),
{
    let mut client = match build_client()?.get(url).send() {
        Ok(v) => v,
        Err(v) => return Err(format!("Failed to GET resource: {:?}", v)),
    };

    let size = {
        let size: Option<&ContentLength> = client.headers().get();
        match size {
            Some(&ContentLength(v)) => v,
            None => 0,
        }
    };

    let mut buf = [0 as u8; 8192];
    loop {
        let len = client.read(&mut buf);
        let len = match len {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to read resource: {:?}", v)),
        };

        if len == 0 {
            break;
        }

        let buf_copy = &buf[0..len];
        let buf_copy = buf_copy.to_vec();

        callback(buf_copy, size);
    }

    Ok(())
}
