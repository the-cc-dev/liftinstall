/// Serves static files from a asset directory.
extern crate mime_guess;

use assets::mime_guess::{get_mime_type, octet_stream};

macro_rules! include_files_as_assets {
    ( $field_name:ident, $( $file_name:expr ),* ) => {
        static $field_name: phf::Map<&'static str, &'static [u8]> = phf_map!(
            $(
                $file_name => include_bytes!(concat!("../static/", $file_name)),
            )*
        );
    }
}

include_files_as_assets!(
    ASSETS,
    "/index.html",
    "/css/bulma.css",
    "/css/main.css",
    "/img/logo.png",
    "/js/helpers.js",
    "/js/vue.js",
    "/js/vue.min.js"
);

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

    Some((string_mime, (*ASSETS.get(file_path)?).to_owned()))
}
