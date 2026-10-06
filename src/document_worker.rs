//! One-shot semantic document worker. START precedes all source/frame/parser work.
use crate::{
    document_budget::ActorArena,
    document_model::{Error, ErrorKind, Result},
    document_obsidian::Control,
    document_protocol as ipc,
};
use std::{
    io::{Read, Write},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
#[cfg(windows)]
fn start_gate(raw: usize) -> Result<()> {
    use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle};
    if raw == 0 || raw == usize::MAX {
        return Err(Error::new(ErrorKind::Containment));
    }
    // Only the controller's inherited read handle enters this private subprocess.
    let mut file = unsafe { std::fs::File::from_raw_handle(raw as *mut std::ffi::c_void) };
    let mut byte = [0; 1];
    file.read_exact(&mut byte)
        .map_err(|_| Error::new(ErrorKind::Containment))?;
    if byte != [1]
        || file
            .read(&mut byte)
            .map_err(|_| Error::new(ErrorKind::Containment))?
            != 0
    {
        return Err(Error::new(ErrorKind::Containment));
    }
    let handle = file.as_raw_handle();
    let closed = unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) } != 0;
    if closed {
        let _ = file.into_raw_handle();
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Containment))
    }
}
/// Hidden worker entry; no route/network/source file arguments.
pub fn run_private_stdio(generation: u64, request: u64, start_handle: usize) -> Result<()> {
    #[cfg(not(windows))]
    {
        let _ = (generation, request, start_handle);
        return Err(Error::new(ErrorKind::UnsupportedPlatform));
    }
    #[cfg(windows)]
    {
        start_gate(start_handle)?;
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let arena = ActorArena::new();
        let mut buffer = ipc::FrameBuffer::new(&arena)?;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        ipc::send_boot(&mut out, generation, request)?;
        out.flush().map_err(|_| Error::new(ErrorKind::InvalidIpc))?;
        let (source, profile) = ipc::receive_source(
            &mut std::io::stdin().lock(),
            generation,
            request,
            &mut buffer,
            &ctl,
        )?;
        match crate::document_parser::project(source, profile, arena, &ctl) {
            Ok(candidate) => ipc::send_model(&candidate, &mut out, &mut buffer, &ctl),
            Err(error) => ipc::send_failure(&mut out, generation, request, error),
        }
    }
}
