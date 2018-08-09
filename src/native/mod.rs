//! Natives/platform specific interactions.

#[cfg(windows)]
mod natives {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    use std::ffi::CString;

    use logging::LoggingErrors;

    use std::env;
    use std::process::Command;

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

    /// Cleans up the installer
    pub fn burn_on_exit() {
        let current_exe = env::current_exe().log_expect("Current executable could not be found");
        let path = current_exe
            .parent()
            .log_expect("Parent directory of executable could not be found");

        // Need a cmd workaround here.
        let tool = path.join("maintenancetool.exe");
        let tool = tool
            .to_str()
            .log_expect("Unable to convert tool path to string")
            .replace(" ", "\\ ");

        let log = path.join("installer.log");
        let log = log
            .to_str()
            .log_expect("Unable to convert log path to string")
            .replace(" ", "\\ ");

        let target_arguments = format!("ping 127.0.0.1 -n 3 > nul && del {} {}", tool, log);

        info!("Launching cmd with {:?}", target_arguments);

        Command::new("C:\\Windows\\system32\\cmd.exe")
            .arg("/C")
            .arg(&target_arguments)
            .spawn()
            .log_expect("Unable to start child process");
    }
}

#[cfg(not(windows))]
mod natives {
    use std::fs::remove_file;

    use std::env;

    use logging::LoggingErrors;

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

    /// Cleans up the installer
    pub fn burn_on_exit() {
        let current_exe = env::current_exe().log_expect("Current executable could not be found");
        let path = current_exe
            .parent()
            .log_expect("Parent directory of executable could not be found");

        // Thank god for *nix platforms
        if let Err(e) = remove_file(path.join("/maintenancetool")) {
            // No regular logging now.
            eprintln!("Failed to delete maintenancetool: {:?}", e);
        };

        if let Err(e) = remove_file(path.join("/installer.log")) {
            // No regular logging now.
            eprintln!("Failed to delete installer log: {:?}", e);
        };
    }
}

pub use self::natives::*;
