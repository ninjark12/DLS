// Firmware flashing for the Keebio Iris LM-K (STM32G431, stm32-dfu bootloader).
//
// The user never compiles firmware — we ship a precompiled `.bin` plus a
// bundled `dfu-util` as Tauri resources and shell out to it. `dfu-util` flashes
// whichever half is currently connected and sitting in DFU bootloader mode; the
// split (left/right) distinction is purely which half the user has plugged in
// and reset, so there is deliberately no left/right parameter here.

use std::process::Command;

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

/// Bundled resource paths (declared in tauri.conf.json `bundle.resources`).
const DFU_UTIL_RESOURCE: &str = "resources/dfu-util.exe";
const FIRMWARE_RESOURCE: &str = "resources/iris_lm_dls.bin";

/// STM32 DFU bootloader USB id (STMicroelectronics DfuSe).
const STM32_DFU_ID: &str = "0483:df11";
/// STM32 internal flash origin; `:leave` reboots the MCU after download.
const STM32_FLASH_TARGET: &str = "0x08000000:leave";

/// A bundled binary counts as usable only if it exists and is non-empty. Repo
/// placeholders are zero bytes; real artifacts are not.
fn is_present(path: &std::path::Path) -> bool {
    std::fs::metadata(path).map(|m| m.len() > 0).unwrap_or(false)
}

/// Turn a finished process result into a UI-friendly `Result`.
///
/// On success, hand back stdout. On failure, `dfu-util` writes useful
/// diagnostics to both streams, so combine whatever is present into one
/// readable message rather than a generic failure.
fn interpret_flash_output(success: bool, stdout: &str, stderr: &str) -> Result<String, String> {
    if success {
        return Ok(stdout.trim().to_string());
    }

    let mut parts: Vec<&str> = Vec::new();
    let err = stderr.trim();
    let out = stdout.trim();
    if !err.is_empty() {
        parts.push(err);
    }
    if !out.is_empty() {
        parts.push(out);
    }

    if parts.is_empty() {
        Err("Flashing failed, but dfu-util produced no output.".to_string())
    } else {
        Err(parts.join("\n\n"))
    }
}

#[tauri::command]
pub fn flash_half(app: AppHandle) -> Result<String, String> {
    let dfu_util = app
        .path()
        .resolve(DFU_UTIL_RESOURCE, BaseDirectory::Resource)
        .map_err(|e| format!("Could not locate bundled dfu-util: {e}"))?;
    let firmware = app
        .path()
        .resolve(FIRMWARE_RESOURCE, BaseDirectory::Resource)
        .map_err(|e| format!("Could not locate bundled firmware: {e}"))?;

    // The repo ships zero-byte placeholders so the Tauri bundler is satisfied
    // before the real binaries exist (Phase B). Treat missing OR empty as
    // "not bundled yet" so the UI shows a clean message instead of trying to
    // execute an empty file.
    if !is_present(&dfu_util) {
        return Err(
            "Bundled dfu-util is not available in this build yet. Firmware flashing is not \
             ready — use the manual instructions for now."
                .to_string(),
        );
    }
    if !is_present(&firmware) {
        return Err(
            "Bundled firmware (.bin) is not available in this build yet. Firmware flashing is \
             not ready — use the manual instructions for now."
                .to_string(),
        );
    }

    let output = Command::new(&dfu_util)
        .args([
            "-a",
            "0",
            "-d",
            STM32_DFU_ID,
            "-s",
            STM32_FLASH_TARGET,
            "-D",
        ])
        .arg(&firmware)
        .output()
        .map_err(|e| format!("Failed to run dfu-util: {e}"))?;

    interpret_flash_output(
        output.status.success(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_returns_stdout() {
        let r = interpret_flash_output(true, "Download done.\n", "");
        assert_eq!(r, Ok("Download done.".to_string()));
    }

    #[test]
    fn failure_surfaces_stderr() {
        let r = interpret_flash_output(false, "", "No DFU capable USB device available");
        assert_eq!(r, Err("No DFU capable USB device available".to_string()));
    }

    #[test]
    fn failure_with_only_stdout_still_surfaces_it() {
        let r = interpret_flash_output(false, "dfu-util: error resetting", "");
        assert_eq!(r, Err("dfu-util: error resetting".to_string()));
    }

    #[test]
    fn failure_combines_both_streams() {
        let r = interpret_flash_output(false, "some stdout", "some stderr");
        assert_eq!(r, Err("some stderr\n\nsome stdout".to_string()));
    }

    #[test]
    fn failure_with_no_output_gives_generic_message() {
        let r = interpret_flash_output(false, "  ", "");
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("no output"));
    }
}
