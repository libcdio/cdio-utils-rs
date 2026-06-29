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

use std::{ffi::CStr, mem::MaybeUninit};

use displaydoc::Display;
use libcdio_sys::{cdio_hwinfo_t, driver_id_t_DRIVER_DEVICE};
use thiserror::Error;

use crate::cdio::Cdio;

/// An interface to a disc drive.
pub struct Drive {
    pub(crate) cdio: Cdio,
}

impl Drive {
    /// Use a default connected drive.
    ///
    /// # Errors
    /// If there are no drives connected, or the drive could not be opened.
    pub fn new() -> Result<Self, DriveNotFoundError> {
        Cdio::open(None, Some(driver_id_t_DRIVER_DEVICE))
            .ok_or(DriveNotFoundError)
            .map(|cdio| Self { cdio })
    }

    /// Return the default disc device.
    /// Returns `None` if the default device could not be fetched.
    pub fn default_device(&self) -> Option<String> {
        let default_device_p = unsafe { libcdio_sys::cdio_get_default_device(self.cdio.as_ptr()) };

        if default_device_p.is_null() {
            return None;
        }

        // SAFETY: Null check has been handled above
        let default_device = unsafe { CStr::from_ptr(default_device_p) };
        let default_device = default_device.to_string_lossy().to_string();

        // SAFETY: default_device_p has been duplicated into a Rust String
        // and is not needed anymore
        unsafe {
            libcdio_sys::cdio_free(default_device_p.cast());
        }

        Some(default_device)
    }

    /// Returns a list of connected hardware devices.
    /// `None` is returned if the device list could not be fetched.
    pub fn devices(&self) -> Option<Vec<String>> {
        let devices_pp = unsafe { libcdio_sys::cdio_get_devices(driver_id_t_DRIVER_DEVICE) };
        if devices_pp.is_null() {
            return None;
        }

        let mut devices = Vec::new();
        let mut devices_pp1 = devices_pp;
        // SAFETY: The device list is NULL terminated, therefore safe to
        // dereference till NULL is reached
        while let device = unsafe { *devices_pp1 }
            && !device.is_null()
        {
            // SAFETY: device is not null and should be a valid string
            let device = unsafe { CStr::from_ptr(device) };
            devices.push(device.to_string_lossy().to_string());
            devices_pp1 = unsafe { devices_pp1.offset(1) };
        }

        // SAFETY: Device list is no has been duplicated above and is
        // not needed anymore
        unsafe {
            libcdio_sys::cdio_free_device_list(devices_pp);
        }

        Some(devices)
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
    fn default_device_test() {
        let drive = Drive::new().unwrap();
        assert!(drive.default_device().is_some());
    }

    #[test]
    #[ignore = "requires a disc drive"]
    fn devices() {
        let drive = Drive::new().unwrap();
        assert!(!drive.devices().unwrap().is_empty());
    }

    #[test]
    #[ignore = "requires a disc drive"]
    fn hardware_info() {
        Drive::new().unwrap().hardware_info().unwrap();
    }
}
