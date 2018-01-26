extern crate includedir_codegen;

use includedir_codegen::Compression;

fn main() {
    includedir_codegen::start("FILES")
        .dir("static", Compression::Gzip)
        .build("data.rs")
        .unwrap();
}
