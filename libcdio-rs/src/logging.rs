//! libcdio logging routines.
//!
//! The tracing crate is used to emit logs. The "log" feature of tracing is
//! enabled, therefore it should also emit logs in the "log" format if
//! a tracing subscriber is not used.
//!
//! Make sure to call [`init_logger()`].

use std::{
    ffi::{CStr, c_char},
    sync::Once,
};

use libcdio_sys::{
    cdio_log_level_t, cdio_log_level_t_CDIO_LOG_ASSERT, cdio_log_level_t_CDIO_LOG_DEBUG,
    cdio_log_level_t_CDIO_LOG_ERROR, cdio_log_level_t_CDIO_LOG_INFO,
    cdio_log_level_t_CDIO_LOG_WARN,
};

/// Configures libcdio to emit tracing (or "log" crate) logs.
/// This should be called before using any other methods.
pub fn init_logger() {
    LOG_HANDLER.call_once(|| unsafe {
        libcdio_sys::cdio_log_set_handler(Some(log_handler));

        // libcdio doesn't call our log handler unless its log level is
        // appropriate. Since the current log level is chosen by log
        // subscribers, hack around by setting libcdio's log level to debug.
        libcdio_sys::cdio_loglevel_default = cdio_log_level_t_CDIO_LOG_DEBUG;
    });
}

static LOG_HANDLER: Once = Once::new();

/// Handle logs via tracing.
extern "C" fn log_handler(level: cdio_log_level_t, message: *const c_char) {
    if message.is_null() {
        return;
    }
    // SAFETY: message is not null, and is backed by an array on the
    // caller's stack.
    let message = unsafe { CStr::from_ptr(message) };
    let Ok(message) = message.to_str() else {
        return;
    };
    #[allow(non_upper_case_globals)]
    match level {
        cdio_log_level_t_CDIO_LOG_ASSERT | cdio_log_level_t_CDIO_LOG_ERROR => {
            tracing::error!(message)
        }
        cdio_log_level_t_CDIO_LOG_WARN => tracing::warn!(message),
        cdio_log_level_t_CDIO_LOG_INFO => tracing::info!(message),
        _ => tracing::debug!(message),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(test)]
    fn log_test() {
        init_logger();
        unsafe {
            libcdio_sys::cdio_log(cdio_log_level_t_CDIO_LOG_ASSERT, c"assert msg".as_ptr());
            libcdio_sys::cdio_log(cdio_log_level_t_CDIO_LOG_ERROR, c"error msg".as_ptr());
            libcdio_sys::cdio_log(cdio_log_level_t_CDIO_LOG_WARN, c"warn msg".as_ptr());
            libcdio_sys::cdio_log(cdio_log_level_t_CDIO_LOG_INFO, c"info msg".as_ptr());
            libcdio_sys::cdio_log(cdio_log_level_t_CDIO_LOG_DEBUG, c"debug msg".as_ptr());
        }
    }
}
