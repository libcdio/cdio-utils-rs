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

use std::{
    ffi::{CStr, c_char},
    ops::Deref,
    ptr::{self, NonNull},
    sync::Mutex,
};

use libcdio_sys::{CdIo_t, driver_id_t, driver_id_t_DRIVER_DEVICE};

use crate::logging;

/// The Cdio type.
pub(crate) struct Cdio {
    pub(crate) cdio: NonNull<CdIo_t>,
}

impl Cdio {
    /// Create a new Cdio object with the given parameters.
    pub(crate) fn new(device: Option<&CStr>, driver: driver_id_t) -> Option<Self> {
        let source = device.map(|s| s.as_ptr()).unwrap_or(ptr::null());
        NonNull::new(Self::open(source, driver)).map(|cdio| Self { cdio })
    }

    fn open(source: *const c_char, driver: driver_id_t) -> *mut CdIo_t {
        logging::init_logger();

        // SAFETY: This invokes cdio_init(), which mutates a static variable.
        // CDIO_LAST_DRIVER_LOCK is held to prevent data races.
        let _lock = CDIO_LAST_DRIVER_LOCK.lock().unwrap();
        unsafe { libcdio_sys::cdio_open(source, driver) }
    }

    pub(crate) const DEVICE_DRIVER: driver_id_t = driver_id_t_DRIVER_DEVICE;
}

impl Deref for Cdio {
    type Target = NonNull<CdIo_t>;

    fn deref(&self) -> &Self::Target {
        &self.cdio
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

/// A lock guarding a private static named `CdIo_last_driver`. It must be held
/// before invoking any libcdio methods that modify this value.
/// As of libcdio v2.3.0, such methods are `cdio_init()` and `cdio_destroy()`.
static CDIO_LAST_DRIVER_LOCK: Mutex<()> = Mutex::new(());
