//! Natives/platform specific interactions.

#[cfg(windows)]
mod natives {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    use std::ffi::CString;

    use logging::LoggingErrors;

    use std::env;

    include!(concat!(env!("OUT_DIR"), "/interop.rs"));

    // Needed here for Windows interop
    #[allow(unsafe_code)]
    pub fn create_shortcut(
        name: &str,
        description: &str,
        target: &str,
        args: &str,
        working_dir: &str,
    ) -> Result<String, String> {
        let source_file = format!(
            "{}\\Microsoft\\Windows\\Start Menu\\Programs\\{}.lnk",
            env::var("APPDATA").log_expect("APPDATA is bad, apparently"),
            name
        );

        info!("Generating shortcut @ {:?}", source_file);

        let native_target_dir = CString::new(source_file.clone())
            .log_expect("Error while converting to C-style string");
        let native_description =
            CString::new(description).log_expect("Error while converting to C-style string");
        let native_target =
            CString::new(target).log_expect("Error while converting to C-style string");
        let native_args = CString::new(args).log_expect("Error while converting to C-style string");
        let native_working_dir =
            CString::new(working_dir).log_expect("Error while converting to C-style string");

        let shortcutResult = unsafe {
            saveShortcut(
                native_target_dir.as_ptr(),
                native_description.as_ptr(),
                native_target.as_ptr(),
                native_args.as_ptr(),
                native_working_dir.as_ptr(),
            )
        };

        match shortcutResult {
            0 => Ok(source_file),
            _ => Err(format!(
                "Windows gave bad result while creating shortcut: {:?}",
                shortcutResult
            )),
        }
    }
}

#[cfg(not(windows))]
mod natives {
    pub fn create_shortcut(
        name: &str,
        description: &str,
        target: &str,
        args: &str,
        working_dir: &str,
    ) -> Result<String, String> {
        // TODO: no-op
        warn!("create_shortcut is stubbed!");

        Ok("".to_string())
    }
}

pub use self::natives::*;
