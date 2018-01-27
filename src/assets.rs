/// Serves static files from a asset directory.

use std::borrow::Cow;

use hyper::header::ContentType;

use mime_guess::{get_mime_type, octet_stream};

// Include built-in files
include!(concat!(env!("OUT_DIR"), "/data.rs"));

/// Returns a static file based upon a given String as a Path.
///
/// file_path: String path, beginning with a /
pub fn file_from_string(file_path : &str) -> Option<(ContentType, Cow<'static, [u8]>)> {
    let guessed_mime = match file_path.rfind(".") {
        Some(ext_ptr) => {
            let ext = &file_path[ext_ptr + 1 ..];

            get_mime_type(ext)
        },
        None => octet_stream()
    };

    let string_mime = guessed_mime.to_string();

    let content_type = ContentType(string_mime.parse().unwrap());

    // We already get the / from the HTTP request.
    match FILES.get(&format!("static{}", file_path)) {
        Ok(val) => Some((content_type, val)),
        // Only error is a not found one
        Err(_) => None
    }
}
