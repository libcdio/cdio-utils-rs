// Copyright (C) 2026 Shiva Kiran Koninty <shiva@skran.xyz>
//
// This file is part of libcdio-rs.
//
// libcdio-rs is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// libcdio-rs is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with libcdio-rs. If not, see <https://www.gnu.org/licenses/>.

//! The main Cdio type.

use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr::{self, NonNull},
    sync::Mutex,
};

use libcdio_sys::{CdIo_t, driver_id_t};

use crate::{device::Driver, logging};

/// The Cdio type.
pub struct Cdio {
    pub(crate) cdio: NonNull<CdIo_t>,
}

#[derive(Clone, Default)]
pub struct CdioBuilder<'a> {
    source: Option<&'a Path>,
    driver: Option<Driver>,
}

/// MAINTAINER NOTE:
/// A lock guarding a private static named `CdIo_last_driver`. It must be held
/// before invoking any libcdio methods that modify this value.
/// As of libcdio v2.3.0, such methods are `cdio_init()` and `cdio_destroy()`.
static CDIO_LAST_DRIVER_LOCK: Mutex<()> = Mutex::new(());

impl Cdio {
    pub fn builder<'a>() -> CdioBuilder<'a> {
        CdioBuilder::new()
    }

    /// Uses the OS driver and a default device. Returns `None` if a default
    /// device could not be found.
    pub fn new() -> Option<Self> {
        Self::open(None, None)
    }

    fn open(source: Option<&CStr>, driver: Option<Driver>) -> Option<Self> {
        logging::init_logger();

        let driver = driver.unwrap_or(Driver::Unknown);
        let source = source.map(|src| src.as_ptr()).unwrap_or(ptr::null());
        let _lock = CDIO_LAST_DRIVER_LOCK.lock().unwrap();

        // SAFETY: This method invokes cdio_init(), which modifies a static variable.
        // CDIO_LAST_DRIVER_LOCK is held to prevent data races.
        let cdio = unsafe { libcdio_sys::cdio_open(source, driver_id_t::from(driver)) };

        Some(Self {
            cdio: NonNull::new(cdio)?,
        })
    }
}

impl<'a> CdioBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// The source to read from, such as a device or image path.
    pub fn source(mut self, source: &'a Path) -> Self {
        self.source = Some(source);
        self
    }

    /// The driver to use.
    /// This is determined automatically if the source is provided.
    /// Therefore, only use this if you want to override it.
    pub fn driver(mut self, driver: Driver) -> Self {
        self.driver = Some(driver);
        self
    }

    /// Build the Cdio type with the set params.
    /// # Returns
    /// `None` if the `source` could not be read from,
    /// or the driver is not available.
    pub fn build(self) -> Option<Cdio> {
        let source = self
            .source
            .and_then(|src| src.to_str())
            .and_then(|src| CString::new(src).ok());

        Cdio::open(source.as_deref(), self.driver)
    }
}

impl Drop for Cdio {
    fn drop(&mut self) {
        let _lock = CDIO_LAST_DRIVER_LOCK.lock().unwrap();

        // SAFETY: This method invokes modifies a static variable.
        // CDIO_LAST_DRIVER_LOCK is held to prevent data races.
        unsafe { libcdio_sys::cdio_destroy(self.cdio.as_ptr()) }
    }
}
