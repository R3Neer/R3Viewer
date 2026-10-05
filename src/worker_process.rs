//! Platform containment for the private parser subprocesses.
use std::{io, process::Child};

pub(crate) fn supported() -> bool {
    cfg!(windows)
}

#[cfg(windows)]
pub(crate) struct ChildJob(std::os::windows::io::OwnedHandle);
#[cfg(not(windows))]
pub(crate) struct ChildJob;

#[cfg(windows)]
impl ChildJob {
    pub(crate) fn attach(child: &Child) -> io::Result<Self> {
        use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
        use windows_sys::Win32::System::JobObjects::*;
        // The handle is not inheritable; only the controller owns the lease/job.
        let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
        // Windows committed memory, not RSS and not total product memory.
        info.ProcessMemoryLimit = 512 * 1024 * 1024;
        if unsafe {
            SetInformationJobObject(
                raw,
                JobObjectExtendedLimitInformation,
                (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&info) as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if unsafe { AssignProcessToJobObject(raw, child.as_raw_handle()) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(handle))
    }
    pub(crate) fn close(self) {
        drop(self.0);
    }
}
#[cfg(not(windows))]
impl ChildJob {
    pub(crate) fn attach(_: &Child) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "worker memory containment is not validated on this platform",
        ))
    }
    pub(crate) fn close(self) {}
}

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReapState {
    Pending,
    Reaped,
    Failed,
}
/// A receipt, never a blocking wait or ownership of the process.
#[derive(Debug, Clone)]
pub struct ReapTicket(Arc<AtomicU8>);
impl ReapTicket {
    pub fn reaped() -> Self {
        Self(Arc::new(AtomicU8::new(1)))
    }
    pub fn state(&self) -> ReapState {
        match self.0.load(Ordering::Acquire) {
            0 => ReapState::Pending,
            1 => ReapState::Reaped,
            _ => ReapState::Failed,
        }
    }
    pub(crate) fn pending() -> Self {
        Self(Arc::new(AtomicU8::new(0)))
    }
    pub(crate) fn complete(&self, success: bool) {
        self.0.store(if success { 1 } else { 2 }, Ordering::Release);
    }
}
