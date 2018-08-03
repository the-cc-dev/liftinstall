//! Serves static files from a asset directory.

extern crate mime_guess;

use assets::mime_guess::{get_mime_type, octet_stream};

macro_rules! include_files_as_assets {
    ( $target_match:expr, $( $file_name:expr ),* ) => {
        match $target_match {
            $(
                $file_name => Some(include_bytes!(concat!("../static/", $file_name)).as_ref()),
            )*
            _ => None
        }
    }
}

/// Returns a static file based upon a given String as a Path.
///
/// file_path: String path, beginning with a /
pub fn file_from_string(file_path: &str) -> Option<(String, &'static [u8])> {
    let guessed_mime = match file_path.rfind(".") {
        Some(ext_ptr) => {
            let ext = &file_path[ext_ptr + 1..];

            get_mime_type(ext)
        }
        None => octet_stream(),
    };

    let string_mime = guessed_mime.to_string();

    let contents = include_files_as_assets!(
        file_path,
        "/index.html",
        "/favicon.ico",
        "/css/bulma.min.css",
        "/css/main.css",
        "/js/helpers.js",
        "/js/vue.min.js"
    )?;

    Some((string_mime, contents))
}
