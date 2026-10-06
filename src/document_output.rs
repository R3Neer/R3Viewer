//! Complete, noninteractive document preparation. No stdout, reaper or source reopening.
use crate::{
    config::PresentationConfig,
    document_budget::ActorArena,
    document_controller::{ControllerPoll, DocumentController, IdentityLatch},
    document_inline,
    document_model::{Error as ModelError, ErrorKind as ModelErrorKind, Profile},
    document_source::DocumentSource,
    document_text,
    input::SourceOrigin,
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

pub const OUTPUT_LIMIT: usize = 256 * 1024 * 1024;
#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Document,
    Source,
}
#[derive(Debug)]
pub struct PreparedOutput {
    pub content: String,
    pub mode: Mode,
    pub diagnostic: Option<&'static str>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Cancelled,
    Limit,
    Preparation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub cleanup_pending: bool,
    pub original_failure: Option<ModelError>,
}
impl Error {
    pub fn exit_code(self) -> u8 {
        if self.kind == ErrorKind::Cancelled {
            130
        } else {
            1
        }
    }
    pub fn diagnostic(self) -> &'static str {
        if self.kind == ErrorKind::Cancelled {
            "preparación de Documento cancelada"
        } else if self.cleanup_pending {
            "no se puede publicar Documento ni Fuente: cierre del worker pendiente"
        } else if self.kind == ErrorKind::Limit {
            "no se puede preparar la salida completa del documento dentro del presupuesto"
        } else {
            "no se pudo obtener autoridad para preparar Documento"
        }
    }
}
fn failure(kind: ErrorKind, pending: bool, cause: Option<ModelError>, cancelled: bool) -> Error {
    Error {
        kind: if cancelled || cause.is_some_and(|e| e.kind == ModelErrorKind::Cancelled) {
            ErrorKind::Cancelled
        } else {
            kind
        },
        cleanup_pending: pending,
        original_failure: cause,
    }
}
fn gate(
    controller: &DocumentController,
    cancel: &AtomicBool,
    cause: Option<ModelError>,
    observed_cancel: bool,
) -> Result<(), Error> {
    let cancelled = observed_cancel
        || cancel.load(Ordering::Acquire)
        || cause.is_some_and(|e| e.kind == ModelErrorKind::Cancelled);
    let pending = controller.pending_cleanup();
    if cancelled || pending {
        Err(failure(ErrorKind::Preparation, pending, cause, cancelled))
    } else {
        Ok(())
    }
}
fn text_error(
    error: document_text::Error,
    controller: &DocumentController,
    cancel: &AtomicBool,
    cause: Option<ModelError>,
) -> Error {
    failure(
        ErrorKind::Limit,
        controller.pending_cleanup(),
        cause,
        error == document_text::Error::Cancelled || cancel.load(Ordering::Acquire),
    )
}
fn fallback(
    original: &Arc<String>,
    config: &PresentationConfig,
    cancel: &AtomicBool,
    limit: usize,
    controller: &DocumentController,
    cause: ModelError,
) -> Result<PreparedOutput, Error> {
    gate(controller, cancel, Some(cause), false)?;
    let content = document_inline::source_fallback(original, config.line_numbers, cancel, limit)
        .map_err(|e| text_error(e, controller, cancel, Some(cause)))?;
    gate(controller, cancel, Some(cause), false)?;
    let diagnostic = match cause.kind {
        ModelErrorKind::SourceLimit => {
            "no se pudo preparar Documento; se muestra Fuente completa: tamaño de fuente"
        }
        ModelErrorKind::UnsupportedPlatform => {
            "no se pudo preparar Documento; se muestra Fuente completa: plataforma no acreditada"
        }
        ModelErrorKind::Timeout => {
            "no se pudo preparar Documento; se muestra Fuente completa: plazo del worker"
        }
        _ => "no se pudo preparar Documento; se muestra Fuente completa: fallo del worker",
    };
    Ok(PreparedOutput {
        content,
        mode: Mode::Source,
        diagnostic: Some(diagnostic),
    })
}
/// The original String moves into one Arc at the caller; every stage shares that allocation.
/// Polling waits only here, outside any interactive pump, under the controller's single deadline.
#[allow(clippy::too_many_arguments)] // Explicit source identity, config, cancellation and output quota.
pub fn prepare(
    executable: PathBuf,
    original: Arc<String>,
    origin: SourceOrigin,
    columns: u16,
    config: &PresentationConfig,
    cancel: Arc<AtomicBool>,
    output_limit: usize,
) -> Result<PreparedOutput, Error> {
    let mut controller = DocumentController::new();
    gate(&controller, &cancel, None, false)?;
    let source = match DocumentSource::new(original.clone(), origin, 1, 1, &cancel) {
        Ok(source) => Arc::new(source),
        Err(cause) => {
            return fallback(&original, config, &cancel, output_limit, &controller, cause);
        }
    };
    gate(&controller, &cancel, None, false)?;
    if let Err(cause) = controller.prepare(
        executable,
        source,
        Profile::Extended,
        ActorArena::new(),
        cancel.clone(),
        Arc::new(IdentityLatch::new(1, 1)),
    ) {
        // A BUSY/Pending rejection grants no authority and cannot authorize a fallback.
        if cause.kind == ModelErrorKind::PendingCleanup {
            return Err(failure(
                ErrorKind::Preparation,
                controller.pending_cleanup(),
                Some(cause),
                cancel.load(Ordering::Acquire),
            ));
        }
        return fallback(&original, config, &cancel, output_limit, &controller, cause);
    }
    let mut observed_cancel = false;
    loop {
        if cancel.load(Ordering::Acquire) && !observed_cancel {
            observed_cancel = true;
            controller.cancel();
        }
        match controller.poll() {
            ControllerPoll::Pending => thread::sleep(Duration::from_millis(2)),
            ControllerPoll::Ready(document) => {
                gate(&controller, &cancel, None, observed_cancel)?;
                let rendered = document_inline::render(
                    Arc::new(document),
                    columns,
                    config,
                    &cancel,
                    output_limit,
                )
                .map_err(|e| text_error(e, &controller, &cancel, None))?;
                gate(&controller, &cancel, None, observed_cancel)?;
                return Ok(PreparedOutput {
                    content: rendered.content,
                    mode: match rendered.mode {
                        document_inline::Mode::Document => Mode::Document,
                        document_inline::Mode::Source => Mode::Source,
                    },
                    diagnostic: rendered.diagnostic,
                });
            }
            ControllerPoll::Failed(cause) => {
                gate(&controller, &cancel, Some(cause), observed_cancel)?;
                return fallback(&original, config, &cancel, output_limit, &controller, cause);
            }
            ControllerPoll::Idle | ControllerPoll::CleanupComplete { .. } => {
                let cause = ModelError::new(ModelErrorKind::Internal);
                return Err(failure(
                    ErrorKind::Preparation,
                    controller.pending_cleanup(),
                    Some(cause),
                    observed_cancel || cancel.load(Ordering::Acquire),
                ));
            }
        }
    }
}

#[cfg(all(test, windows))]
pub(crate) mod tests;
