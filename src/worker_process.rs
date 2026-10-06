//! Platform containment for the private parser subprocesses.
use std::{io, process::Child};

pub(crate) fn supported() -> bool {
    cfg!(windows)
}

#[cfg(windows)]
pub(crate) struct ChildJob(std::os::windows::io::OwnedHandle);
#[cfg(not(windows))]
pub(crate) struct ChildJob;

pub(crate) struct AttachFailure {
    pub(crate) error: io::Error,
    pub(crate) job: Option<ChildJob>,
}
impl std::fmt::Debug for AttachFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachFailure")
            .field("error", &self.error)
            .field("retained_job", &self.job.is_some())
            .finish()
    }
}

pub(crate) struct CloseFailure {
    pub(crate) error: io::Error,
    pub(crate) job: ChildJob,
}
impl std::fmt::Debug for CloseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloseFailure")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

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
    /// Caller retains both child and any returned job until cleanup is proved.
    pub(crate) fn attach_verified(child: &Child) -> Result<Self, AttachFailure> {
        use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
        use windows_sys::Win32::System::JobObjects::*;
        let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if raw.is_null() {
            return Err(AttachFailure {
                error: io::Error::last_os_error(),
                job: None,
            });
        }
        let job = Self(unsafe { OwnedHandle::from_raw_handle(raw) });
        let setup = (|| -> io::Result<()> {
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
            info.BasicLimitInformation.LimitFlags =
                JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
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
            job.verify_attached(child)
        })();
        job.finish_attach(setup)
    }
    fn finish_attach(self, result: io::Result<()>) -> Result<Self, AttachFailure> {
        match result {
            Ok(()) => Ok(self),
            Err(error) => Err(AttachFailure {
                error,
                job: Some(self),
            }),
        }
    }
    fn verify_attached(&self, child: &Child) -> io::Result<()> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::{Foundation::*, System::JobObjects::*};
        let raw = self.0.as_raw_handle();
        let mut handle_flags = 0;
        if unsafe { GetHandleInformation(raw, &mut handle_flags) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if handle_flags & HANDLE_FLAG_INHERIT != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "worker job is inheritable",
            ));
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        let mut returned = 0;
        if unsafe {
            QueryInformationJobObject(
                raw,
                JobObjectExtendedLimitInformation,
                (&mut info as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&info) as u32,
                &mut returned,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let required = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
        if returned as usize != std::mem::size_of_val(&info)
            || info.BasicLimitInformation.LimitFlags != required
            || info.ProcessMemoryLimit != 512 * 1024 * 1024
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "worker job limits differ from requested containment",
            ));
        }
        let mut member = 0;
        if unsafe { IsProcessInJob(child.as_raw_handle(), raw, &mut member) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if member == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "worker is not in the assigned job",
            ));
        }
        Ok(())
    }
    /// Terminates every associated process; the caller must still wait and query.
    pub(crate) fn terminate(&self, exit_code: u32) -> io::Result<()> {
        use std::os::windows::io::AsRawHandle;
        if unsafe {
            windows_sys::Win32::System::JobObjects::TerminateJobObject(
                self.0.as_raw_handle(),
                exit_code,
            )
        } == 0
        {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
    /// A successful query is independent of waiting for the direct child.
    pub(crate) fn active_processes(&self) -> io::Result<u32> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { std::mem::zeroed() };
        let mut returned = 0;
        if unsafe {
            QueryInformationJobObject(
                self.0.as_raw_handle(),
                JobObjectBasicAccountingInformation,
                (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                std::mem::size_of_val(&info) as u32,
                &mut returned,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if returned as usize != std::mem::size_of_val(&info) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "worker job accounting response has wrong size",
            ));
        }
        Ok(info.ActiveProcesses)
    }
    /// Success consumes ownership; failure returns the same job for cleanup retry.
    pub(crate) fn close_checked(self) -> Result<(), CloseFailure> {
        self.close_with(|raw| {
            if unsafe { windows_sys::Win32::Foundation::CloseHandle(raw) } == 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        })
    }
    // The callback must close exactly this handle on Ok and leave it owned on Err.
    fn close_with(
        self,
        close: impl FnOnce(windows_sys::Win32::Foundation::HANDLE) -> io::Result<()>,
    ) -> Result<(), CloseFailure> {
        use std::os::windows::io::AsRawHandle;
        match close(self.0.as_raw_handle()) {
            Ok(()) => {
                // CloseHandle succeeded: suppress OwnedHandle's second close.
                std::mem::forget(self);
                Ok(())
            }
            Err(error) => Err(CloseFailure { error, job: self }),
        }
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
    pub(crate) fn attach_verified(child: &Child) -> Result<Self, AttachFailure> {
        Self::attach(child).map_err(|error| AttachFailure { error, job: None })
    }
    pub(crate) fn terminate(&self, _: u32) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "worker containment is not validated on this platform",
        ))
    }
    pub(crate) fn active_processes(&self) -> io::Result<u32> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "worker containment is not validated on this platform",
        ))
    }
    pub(crate) fn close_checked(self) -> Result<(), CloseFailure> {
        Err(CloseFailure {
            error: io::Error::new(
                io::ErrorKind::Unsupported,
                "worker containment is not validated on this platform",
            ),
            job: self,
        })
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
