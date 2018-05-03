/// Serves static files from a asset directory.
extern crate mime_guess;

use assets::mime_guess::{get_mime_type, octet_stream};

#[derive(RustEmbed)]
#[folder("static/")]
struct Assets;

/// Returns a static file based upon a given String as a Path.
///
/// file_path: String path, beginning with a /
pub fn file_from_string(file_path: &str) -> Option<(String, Vec<u8>)> {
    let guessed_mime = match file_path.rfind(".") {
        Some(ext_ptr) => {
            let ext = &file_path[ext_ptr + 1..];

            get_mime_type(ext)
        }
        None => octet_stream(),
    };

    let string_mime = guessed_mime.to_string();

    Some((string_mime, Assets::get(file_path)?))
}
