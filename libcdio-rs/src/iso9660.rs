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

use bitflags::bitflags;
use libcdio_sys::{
    iso_extension_enum_s_ISO_EXTENSION_HIGH_SIERRA,
    iso_extension_enum_s_ISO_EXTENSION_JOLIET_LEVEL1,
    iso_extension_enum_s_ISO_EXTENSION_JOLIET_LEVEL2,
    iso_extension_enum_s_ISO_EXTENSION_JOLIET_LEVEL3,
    iso_extension_enum_s_ISO_EXTENSION_ROCK_RIDGE,
};

use crate::logging::init_logger;

/// The main ISO 9660 type
pub struct Iso9660 {
    pub(crate) ptr: NonNull<libcdio_sys::iso9660_t>,
}

bitflags! {
    /// ISO 9660 Extensions.
    /// # Examples
    /// ```rust, no_run
    /// use libcdio_rs::iso9660::Iso9660Extensions;
    /// // pick HighSierra and RockRidge
    /// let extensions = Iso9660Extensions::HighSierra & Iso9660Extensions::RockRidge;
    /// // pick everything except RockRidge
    /// let extensions = Iso9660Extensions::all() - Iso9660Extensions::RockRidge;
    /// // pick nothing
    /// let extensions = Iso9660Extensions::empty();
    /// ```
    #[derive(Clone, Copy, Debug)]
    pub struct Iso9660Extensions: u8 {
        const HighSierra = iso_extension_enum_s_ISO_EXTENSION_HIGH_SIERRA as _;
        const JolietLevel1 = iso_extension_enum_s_ISO_EXTENSION_JOLIET_LEVEL1 as _;
        const JolietLevel2 = iso_extension_enum_s_ISO_EXTENSION_JOLIET_LEVEL2 as _;
        const JolietLevel3 = iso_extension_enum_s_ISO_EXTENSION_JOLIET_LEVEL3 as _;
        const RockRidge = iso_extension_enum_s_ISO_EXTENSION_ROCK_RIDGE as _;
    }
}

impl Iso9660 {
    /// Open an ISO 9660 image for reading at given `path`, with all iso9660
    /// extension flags enabled. Returns `None` on error.
    pub fn new(path: &Path) -> Option<Self> {
        let path = CString::new(path.to_str()?).ok()?;

        Self::open(&path, Iso9660Extensions::all())
    }

    fn open(path: &CStr, extensions: Iso9660Extensions) -> Option<Self> {
        init_logger();

        // SAFETY: path is duplicated by the method, so its safe to drop afterwards
        let iso9660_ptr =
            unsafe { libcdio_sys::iso9660_open_ext(path.as_ptr(), extensions.bits()) };

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
