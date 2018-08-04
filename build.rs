#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("static/favicon.ico");
    res.compile().expect("Failed to generate metadata");
}

#[cfg(not(windows))]
fn main() {}
