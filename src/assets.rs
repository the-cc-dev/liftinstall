/// Serves static files from a asset directory.

extern crate mime_guess;

use std::borrow::Cow;

use assets::mime_guess::{get_mime_type, octet_stream};

// Include built-in files
include!(concat!(env!("OUT_DIR"), "/data.rs"));

/// Returns a static file based upon a given String as a Path.
///
/// file_path: String path, beginning with a /
pub fn file_from_string(file_path: &str) -> Option<(String, Cow<'static, [u8]>)> {
    let guessed_mime = match file_path.rfind(".") {
        Some(ext_ptr) => {
            let ext = &file_path[ext_ptr + 1..];

            get_mime_type(ext)
        }
        None => octet_stream(),
    };

    let string_mime = guessed_mime.to_string();

    // We already get the / from the HTTP request.
    match FILES.get(&format!("static{}", file_path)) {
        Ok(val) => Some((string_mime, val)),
        // Only error is a not found one
        Err(_) => None,
    }
}
