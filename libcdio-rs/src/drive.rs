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

//! Routines related to CD/DVD drives.

use std::{
    ffi::{CStr, OsString},
    mem::MaybeUninit,
    path::PathBuf,
};

use displaydoc::Display;
use libcdio_sys::cdio_hwinfo_t;
use thiserror::Error;

use crate::cdio::Cdio;

/// An interface to a disc drive.
pub struct Drive {
    pub(crate) cdio: Cdio,
}

impl Drive {
    /// Get a list of connected drives.
    pub fn drives() -> Vec<PathBuf> {
        let drive_list = unsafe { libcdio_sys::cdio_get_devices(Cdio::DEVICE_DRIVER) };
        if drive_list.is_null() {
            return vec![];
        }

        let mut drives = Vec::new();
        let mut ptr = drive_list;
        // SAFETY: The device list is NULL terminated, therefore safe to
        // dereference till NULL is reached
        while let drive = unsafe { *ptr }
            && !drive.is_null()
        {
            // SAFETY: null check performed; the value represents a path, thus an os string
            drives.push(PathBuf::from(unsafe {
                OsString::from_encoded_bytes_unchecked(CStr::from_ptr(drive).to_bytes().to_vec())
            }));
            ptr = unsafe { ptr.offset(1) };
        }

        // SAFETY: drive_list has been cloned above, thus safe to free
        unsafe {
            libcdio_sys::cdio_free_device_list(drive_list);
        }

        drives
    }

    /// Use a default connected drive.
    ///
    /// # Errors
    /// If there are no drives connected, or the drive could not be opened.
    pub fn new() -> Result<Self, DriveNotFoundError> {
        Cdio::new(None, Cdio::DEVICE_DRIVER)
            .ok_or(DriveNotFoundError)
            .map(|cdio| Self { cdio })
    }

    /// Returns hardware information of the drive.
    ///
    /// # Errors
    /// If an underlying operation errored, or if the drive is unavailable.
    pub fn hardware_info(&self) -> Result<HardwareInfo, DriveOperationError> {
        let mut hwinfo: MaybeUninit<cdio_hwinfo_t> = MaybeUninit::uninit();
        let ret = unsafe { libcdio_sys::cdio_get_hwinfo(self.cdio.as_ptr(), hwinfo.as_mut_ptr()) };
        if !ret {
            return Err(DriveOperationError);
        }

        // SAFETY: cdio_get_hwinfo() returned true, therefore hwinfo should be initialized
        let hwinfo = unsafe { hwinfo.assume_init() };

        // SAFETY: The strings are null terminated
        unsafe {
            let model = CStr::from_ptr(hwinfo.psz_model.as_ptr());
            let vendor = CStr::from_ptr(hwinfo.psz_vendor.as_ptr());
            let revision = CStr::from_ptr(hwinfo.psz_revision.as_ptr());

            Ok(HardwareInfo {
                model: model.to_string_lossy().trim_end().to_string(),
                vendor: vendor.to_string_lossy().trim_end().to_string(),
                revision: revision.to_string_lossy().trim_end().to_string(),
            })
        }
    }
}

/// could not find any drives
#[non_exhaustive]
#[derive(Debug, Display, Error)]
pub struct DriveNotFoundError;

/// could not perform operation on the drive
#[non_exhaustive]
#[derive(Debug, Display, Error)]
pub struct DriveOperationError;

/// Hardware information returned by a cdio driver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareInfo {
    pub model: String,
    pub vendor: String,
    pub revision: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a disc drive"]
    fn drives() {
        assert!(!Drive::drives().is_empty());
    }

    #[test]
    #[ignore = "requires a disc drive"]
    fn hardware_info() {
        Drive::new().unwrap().hardware_info().unwrap();
    }
}
