extern crate walkdir;
#[cfg(windows)]
extern crate winres;

use walkdir::WalkDir;

use std::env;
use std::path::PathBuf;

use std::fs::copy;
use std::fs::create_dir_all;
use std::fs::File;

use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

const FILES_TO_PREPROCESS: &'static [&'static str] = &["helpers.js", "views.js"];

#[cfg(windows)]
fn handle_binary() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("static/favicon.ico");
    res.compile().expect("Failed to generate metadata");
}

#[cfg(not(windows))]
fn handle_binary() {}

fn main() {
    handle_binary();

    let output_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Copy files from static/ to build dir
    for entry in WalkDir::new("static") {
        let entry = entry.expect("Unable to read output directory");

        let output_file = output_dir.join(entry.path());

        if entry.path().is_dir() {
            create_dir_all(output_file).expect("Unable to create dir");
        } else {
            let filename = entry
                .path()
                .file_name()
                .expect("Unable to parse filename")
                .to_str()
                .expect("Unable to convert to string");

            if FILES_TO_PREPROCESS.contains(&filename) {
                // Do basic preprocessing - transcribe template string
                let source = BufReader::new(File::open(entry.path()).expect("Unable to copy file"));
                let mut target = File::create(output_file).expect("Unable to copy file");

                let mut is_template_string = false;

                for line in source.lines() {
                    let line = line.expect("Unable to read line from JS file");

                    let mut is_break = false;
                    let mut is_quote = false;

                    let mut output_line = String::new();

                    if is_template_string {
                        output_line += "\"";
                    }

                    for c in line.chars() {
                        if c == '\\' {
                            is_break = true;
                            output_line.push('\\');
                            continue;
                        }

                        if (c == '\"' || c == '\'') && !is_break && !is_template_string {
                            is_quote = !is_quote;
                        }

                        if c == '`' && !is_break && !is_quote {
                            output_line += "\"";
                            is_template_string = !is_template_string;
                            continue;
                        }

                        if c == '"' && !is_break && is_template_string {
                            output_line += "\\\"";
                            continue;
                        }

                        is_break = false;
                        output_line.push(c);
                    }

                    if is_template_string {
                        output_line += "\" +";
                    }

                    output_line.push('\n');

                    target
                        .write(output_line.as_bytes())
                        .expect("Unable to write line");
                }
            } else {
                copy(entry.path(), output_file).expect("Unable to copy file");
            }
        }
    }
}
