/// http.rs
///
/// A simple wrapper around Hyper's HTTP client.

use hyper::header::ContentLength;

use reqwest;

use std::io::Read;

/// Streams a file from a HTTP server.
pub fn stream_file<F>(url : String, callback : F) -> Result<(), String>
            // |data : Vec<u8>, total : u64|
    where F: Fn(Vec<u8>, u64) -> () {
    let mut client = match reqwest::get(&url) {
        Ok(v) => v,
        Err(v) => return Err(format!("Failed to GET resource: {:?}", v)),
    };

    let size = {
        let size : Option<&ContentLength> = client.headers().get();
        match size {
            Some(&ContentLength(v)) => v,
            None => 0
        }
    };

    let mut buf = [0 as u8; 8192];
    loop {
        let len = client.read(&mut buf);
        let len = match len {
            Ok(v) => v,
            Err(v) => return Err(format!("Failed to read resource: {:?}", v))
        };

        if len == 0 {
            break;
        }

        let buf_copy = &buf[0 .. len];
        let buf_copy = buf_copy.to_vec();

        callback(buf_copy, size);
    }

    Ok(())
}
