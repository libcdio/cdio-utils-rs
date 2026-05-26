//! The main Cdio type.

use std::ptr::NonNull;

use libcdio_sys::CdIo_t;

/// The Cdio type.
pub struct Cdio {
    pub(crate) cdio: NonNull<CdIo_t>,
}
