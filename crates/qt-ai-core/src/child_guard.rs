//! Buộc tiến trình agy sống chết theo app. Đóng app khi đang dịch, app crash hay bị tắt bằng Task Manager mà
//! không có lớp này thì agy chạy mồ côi: dịch tiếp, ăn quota, giữ folder truyện (không xoá được) — đã gặp
//! 2026-10-21: đóng app khi 6 truyện đang dịch, 6 agy chạy tiếp không ai theo dõi.
//!
//! - Windows: mọi agy gắn vào một Job Object `KILL_ON_JOB_CLOSE`; handle job giữ tới hết đời tiến trình app nên
//!   app chết vì bất cứ lý do gì thì Windows đóng handle và giết cả job (kể cả tiến trình cháu agy sinh ra).
//! - Linux: agy chạy trong nhóm tiến trình riêng (Dừng giết cả nhóm, không sót tiến trình cháu) và nhận SIGKILL
//!   khi app chết (`PR_SET_PDEATHSIG`; cháu thì không — muốn chắc tuyệt đối phải dùng cgroup).
//! - macOS: nhóm tiến trình riêng; app chết đột ngột thì không có cơ chế tự giết.

use std::process::{Child, Command};

/// Chuẩn bị lệnh trước khi spawn (Unix: nhóm tiến trình riêng + chết theo app).
pub fn prepare(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
        #[cfg(target_os = "linux")]
        {
            // SAFETY: chỉ gọi prctl/getppid (async-signal-safe) giữa fork và exec.
            let parent = std::process::id() as libc::pid_t;
            unsafe {
                command.pre_exec(move || {
                    if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    // App đã chết trước khi prctl kịp có hiệu lực → tự thoát thay vì chạy mồ côi.
                    if libc::getppid() != parent {
                        libc::_exit(1);
                    }
                    Ok(())
                });
            }
        }
    }
    #[cfg(not(unix))]
    let _ = command;
}

/// Gắn tiến trình vừa spawn vào job của app (Windows). Lỗi thì bỏ qua: mất lưới an toàn chứ không hỏng phiên.
pub fn bind(child: &Child) {
    #[cfg(windows)]
    windows_job::assign(child);
    #[cfg(not(windows))]
    let _ = child;
}

/// Giết tiến trình và mọi tiến trình con của nó (nút Dừng). Windows `taskkill /T`; Unix giết cả nhóm tiến trình.
pub fn kill_tree(child: &mut Child) {
    #[cfg(windows)]
    {
        let _ = crate::process::quiet_command("taskkill").args(["/PID", &child.id().to_string(), "/T", "/F"]).output();
    }
    #[cfg(unix)]
    {
        // SAFETY: kill với pid âm = gửi tín hiệu cho cả nhóm tiến trình `prepare` đã tạo (pgid = pid của agy).
        unsafe {
            libc::kill(-(child.id() as libc::pid_t), libc::SIGKILL);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(windows)]
mod windows_job {
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    /// Handle job lưu dạng số (HANDLE là con trỏ, không Send/Sync); không bao giờ đóng — để Windows đóng khi app chết.
    fn job() -> Option<HANDLE> {
        static JOB: OnceLock<Option<usize>> = OnceLock::new();
        JOB.get_or_init(|| {
            // SAFETY: tạo job vô danh, đặt cờ giết-khi-đóng; struct khởi tạo zero đúng kích thước Windows yêu cầu.
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if job.is_null() {
                    return None;
                }
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                let ok = SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const core::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                );
                (ok != 0).then_some(job as usize)
            }
        })
        .map(|job| job as HANDLE)
    }

    pub fn assign(child: &Child) {
        if let Some(job) = job() {
            // SAFETY: handle tiến trình của `child` còn sống suốt lời gọi.
            unsafe {
                AssignProcessToJobObject(job, child.as_raw_handle() as HANDLE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::quiet_command;
    use std::time::{Duration, Instant};

    fn long_running() -> Command {
        if cfg!(windows) {
            let mut command = quiet_command("cmd");
            command.args(["/C", "ping -n 30 127.0.0.1 >NUL"]);
            command
        } else {
            let mut command = quiet_command("sh");
            command.args(["-c", "sleep 30"]);
            command
        }
    }

    #[test]
    fn kill_tree_giet_ngay_tien_trinh_da_buoc_vao_app() {
        let mut command = long_running();
        prepare(&mut command);
        let mut child = command.spawn().unwrap();
        bind(&child);
        let started = Instant::now();
        kill_tree(&mut child);
        assert!(started.elapsed() < Duration::from_secs(10), "không đợi hết 30 giây");
        assert!(child.try_wait().unwrap().is_some(), "đã thoát");
    }
}
