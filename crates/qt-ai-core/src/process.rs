//! Spawn tiến trình con không bật cửa sổ console trên Windows. App GUI (windows_subsystem = "windows")
//! không có console; con là console app (agy.exe, tasklist, taskkill) sẽ được Windows cấp console mới →
//! cửa sổ đen nhảy lên. CREATE_NO_WINDOW chặn điều đó, stdout/stderr vẫn pipe được bình thường.

use std::ffi::OsStr;
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn quiet_command(program: impl AsRef<OsStr>) -> Command {
    // `mut` chỉ dùng trong khối cfg(windows); nền khác sẽ báo unused_mut nếu không allow.
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_command_van_chay_va_bat_duoc_output() {
        let output = if cfg!(windows) {
            quiet_command("cmd").args(["/C", "echo hi"]).output().unwrap()
        } else {
            quiet_command("echo").arg("hi").output().unwrap()
        };
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("hi"));
    }
}
