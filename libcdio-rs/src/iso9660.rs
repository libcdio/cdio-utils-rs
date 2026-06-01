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

//! ISO 9660 filesystem related routines.

use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr::NonNull,
};

use crate::logging::init_logger;

/// The main ISO 9660 type
pub struct Iso9660 {
    pub(crate) ptr: NonNull<libcdio_sys::iso9660_t>,
}

impl Iso9660 {
    /// Open an ISO 9660 image for reading at given `path`.
    /// Returns `None` on error.
    pub fn new(path: &Path) -> Option<Self> {
        let path = CString::new(path.to_str()?).ok()?;

        Self::open(&path)
    }

    fn open(path: &CStr) -> Option<Self> {
        init_logger();

        // SAFETY: path is duplicated by the method, so its safe to drop afterwards
        let iso9660_ptr = unsafe { libcdio_sys::iso9660_open(path.as_ptr()) };

        Some(Self {
            ptr: NonNull::new(iso9660_ptr)?,
        })
    }
}

impl Drop for Iso9660 {
    fn drop(&mut self) {
        let _ = unsafe { libcdio_sys::iso9660_close(self.ptr.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(test)]
    fn new() {
        let iso = Iso9660::new(Path::new("../test-data/rock-ridge.iso"));
        assert!(iso.is_some());
    }
}
