/// Serves static files from a asset directory.

use std::borrow::Cow;

// Include built-in files
include!(concat!(env!("OUT_DIR"), "/data.rs"));

/// Returns a static file based upon a given String as a Path.
///
/// file_path: String path, beginning with a /
pub fn file_from_string(file_path : &str) -> Option<(Option<&'static str>, Cow<'static, [u8]>)> {
    let content_type = match file_path.rfind(".") {
        Some(ext_ptr) => {
            let ext = &file_path[ext_ptr + 1 ..];

            // Basic extension matching
            match ext {
                "html" => Some("text/html"),
                "css" => Some("text/css"),
                "js" => Some("application/javascript"),
                "png" => Some("image/png"),
                _ => None
            }
        },
        None => None
    };

    // We already get the / from the HTTP request.
    match FILES.get(&format!("static{}", file_path)) {
        Ok(val) => Some((content_type, val)),
        // Only error is a not found one
        Err(_) => None
    }
}
