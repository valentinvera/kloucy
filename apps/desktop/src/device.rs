//! Android devices for the Device surface.
//!
//! Kloucy drives the SDK's own tools: `adb` lists and controls running devices,
//! and the emulator's `-list-avds` names the ones that can be started. Device
//! screens come back as PNG bytes that the surface draws as an image.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// The Android tooling Kloucy found on this machine.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Devices {
    /// The `adb` command to use.
    pub adb: String,
    /// Emulators that can be started.
    pub avds: Vec<String>,
    /// The emulator binary, when the SDK has one.
    pub emulator: Option<String>,
    /// Running devices, by serial.
    pub running: Vec<String>,
}

/// Look for the SDK's tools and the devices they know about.
pub fn scan() -> Devices {
    let adb = sdk_tool("platform-tools/adb").unwrap_or_else(|| "adb".to_string());
    let emulator = sdk_tool("emulator/emulator");
    let avds = emulator.as_deref().map(list_avds).unwrap_or_default();
    let running = list_devices(&adb);

    Devices {
        adb,
        avds,
        emulator,
        running,
    }
}

/// Start `avd` without a window: the surface shows its screen as a still
/// preview, and a window would only steal focus from the workbench.
pub fn start(emulator: &str, avd: &str) -> Result<(), String> {
    Command::new(emulator)
        .args([
            "-avd",
            avd,
            "-no-window",
            "-no-audio",
            "-no-boot-anim",
            "-no-snapshot-save",
            "-gpu",
            "swiftshader_indirect",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|_| "Kloucy could not start the emulator.".to_string())
}

/// Stop the running device with `serial`.
pub fn stop(adb: &str, serial: &str) -> Result<(), String> {
    let output = Command::new(adb)
        .args(["-s", serial, "emu", "kill"])
        .output()
        .map_err(|_| "Kloucy could not run adb.".to_string())?;

    if output.status.success() {
        return Ok(());
    }

    Err("The device did not stop.".to_string())
}

/// Grab the current screen of `serial` as PNG bytes.
pub fn screenshot(adb: &str, serial: &str) -> Result<Vec<u8>, String> {
    let output = Command::new(adb)
        .args(["-s", serial, "exec-out", "screencap", "-p"])
        .output()
        .map_err(|_| "Kloucy could not ask the device for a screenshot.".to_string())?;

    if !output.status.success() {
        return Err("The device did not answer with a screenshot.".to_string());
    }

    if !output.stdout.starts_with(b"\x89PNG") {
        return Err("The device returned no image.".to_string());
    }

    Ok(output.stdout)
}

/// The pixel size in a PNG's header, so the surface can scale it without a
/// decoder of its own.
pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

    if bytes.len() < 24 || &bytes[..8] != SIGNATURE {
        return None;
    }

    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);

    (width > 0 && height > 0).then_some((width, height))
}

/// The SDK root, from the environment or the usual per-user location.
fn sdk_root() -> Option<PathBuf> {
    std::env::var_os("ANDROID_HOME")
        .or_else(|| std::env::var_os("ANDROID_SDK_ROOT"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join("Android/Sdk"))
        })
}

/// A tool inside the SDK, when it exists.
fn sdk_tool(relative: &str) -> Option<String> {
    let path = sdk_root()?.join(relative);

    path.is_file().then(|| path.to_string_lossy().into_owned())
}

/// The AVD names the emulator reports.
fn list_avds(emulator: &str) -> Vec<String> {
    let Ok(output) = Command::new(emulator).arg("-list-avds").output() else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    parse_avds(&String::from_utf8_lossy(&output.stdout))
}

/// The serials `adb devices` reports as ready.
fn list_devices(adb: &str) -> Vec<String> {
    let Ok(output) = Command::new(adb).arg("devices").output() else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    parse_devices(&String::from_utf8_lossy(&output.stdout))
}

/// Read ready devices out of `adb devices`.
fn parse_devices(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let serial = fields.next()?.trim();
            let state = fields.next()?.trim();

            (state == "device" && !serial.is_empty() && !serial.starts_with("List of"))
                .then(|| serial.to_string())
        })
        .collect()
}

/// Read AVD names out of `emulator -list-avds`, skipping its log lines.
fn parse_avds(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("INFO")
                && !line.starts_with("WARNING")
                && !line.starts_with("ERROR")
        })
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_png_size() {
        let mut header = b"\x89PNG\r\n\x1a\n".to_vec();
        header.extend_from_slice(&[0, 0, 0, 13]);
        header.extend_from_slice(b"IHDR");
        header.extend_from_slice(&900_u32.to_be_bytes());
        header.extend_from_slice(&2000_u32.to_be_bytes());

        assert_eq!(png_size(&header), Some((900, 2000)));
        assert_eq!(png_size(b"not a png"), None);
        assert_eq!(png_size(&header[..20]), None);
    }

    #[test]
    fn reads_ready_devices() {
        let output = "List of devices attached\nemulator-5554\tdevice\nR5CT123\tunauthorized\n";

        assert_eq!(parse_devices(output), vec!["emulator-5554".to_string()]);
    }

    #[test]
    fn skips_emulator_log_lines() {
        let output = "medium_phone\nINFO    | Storing crashdata\nPixel_9_API_36\n\n";

        assert_eq!(
            parse_avds(output),
            vec!["medium_phone".to_string(), "Pixel_9_API_36".to_string()]
        );
    }
}
