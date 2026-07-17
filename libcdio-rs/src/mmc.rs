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

//! SCSI MMC (MultiMedia Commands) routines.

use std::{
    ffi::{CString, NulError, OsString},
    path::PathBuf,
};

pub use get_config::*;
pub use get_event_status::*;
pub use read_subchannel::*;

mod get_config;
mod get_event_status;
mod read_subchannel;

use displaydoc::Display;
use libcdio_sys::{
    cdio_mmc_direction_t, cdio_mmc_level_t_CDIO_MMC_LEVEL_1, cdio_mmc_level_t_CDIO_MMC_LEVEL_2,
    cdio_mmc_level_t_CDIO_MMC_LEVEL_3, cdio_mmc_level_t_CDIO_MMC_LEVEL_NONE,
    cdio_mmc_level_t_CDIO_MMC_LEVEL_WEIRD,
};
use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

use crate::cdio::Cdio;

/// An interface for SCSI MMC commands.
pub struct Mmc {
    cdio: Cdio,
}

/// Represents the MMC Level.
#[non_exhaustive]
#[repr(u32)]
#[derive(
    Clone, Debug, Default, Display, Eq, Hash, Ord, PartialEq, PartialOrd, TryFromPrimitive,
)]
pub enum MmcLevel {
    #[default]
    /// Unknown
    Unknown = cdio_mmc_level_t_CDIO_MMC_LEVEL_WEIRD,
    /// MMC-1
    Mmc1 = cdio_mmc_level_t_CDIO_MMC_LEVEL_1,
    /// MMC-2
    Mmc2 = cdio_mmc_level_t_CDIO_MMC_LEVEL_2,
    /// MMC-3
    Mmc3 = cdio_mmc_level_t_CDIO_MMC_LEVEL_3,
}

impl Mmc {
    /// Use a default device.
    ///
    /// # Errors
    /// If an MMC capable device could not be found.
    pub fn new() -> Result<Mmc, MmcNotFoundError> {
        Cdio::new(None, Cdio::DEVICE_DRIVER)
            .map(|cdio| Self { cdio })
            .filter(|mmc| mmc.level().is_ok())
            .ok_or(MmcNotFoundError)
    }

    /// Use the provided device.
    ///
    /// # Errors
    /// If there are no devices with MMC connected, or the device could not be
    /// opened.
    pub fn with_device(device: PathBuf) -> Result<Mmc, WithDeviceError> {
        let device = CString::new(device.into_os_string().into_encoded_bytes()).map_err(|err| {
            WithDeviceError {
                device: os_string_from_bytes_safe(err.clone().into_vec()).into(),
                source: WithDeviceErrorKind::DeviceHasNullChar(err),
            }
        })?;
        let Some(cdio) = Cdio::new(Some(&device), Cdio::DEVICE_DRIVER) else {
            return Err(WithDeviceError {
                device: os_string_from_bytes_safe(device.into_bytes()).into(),
                source: WithDeviceErrorKind::CouldNotOpenDevice,
            });
        };
        let mmc = Self { cdio };
        return mmc.level().map(|_| mmc).map_err(|_| WithDeviceError {
            device: os_string_from_bytes_safe(device.into_bytes()).into(),
            source: WithDeviceErrorKind::MmcNotSupported,
        });

        fn os_string_from_bytes_safe(bytes: Vec<u8>) -> OsString {
            // SAFETY: the bytes originate from an OsString
            unsafe { OsString::from_encoded_bytes_unchecked(bytes) }
        }
    }

    /// Get the MMC level supported by the drive.
    ///
    /// # Errors
    /// If an underlying operation failed, or if the device is unavailable.
    pub fn level(&self) -> Result<MmcLevel, MmcOperationError> {
        let mmc_level = unsafe { libcdio_sys::mmc_get_drive_mmc_cap(self.cdio.as_ptr()) };
        if mmc_level == cdio_mmc_level_t_CDIO_MMC_LEVEL_NONE {
            return Err(MmcOperationError);
        }

        Ok(MmcLevel::try_from(mmc_level)
            .expect("mmc_get_drive_mmc_cap should return a valid mmc_level_t"))
    }

    fn run_command(
        &self,
        direction: Option<MmcDirection>,
        buf: &mut [u8],
        cdb: Cdb,
    ) -> Result<(), OsError> {
        let direction = direction
            .map(cdio_mmc_direction_t::from)
            .unwrap_or(libcdio_sys::mmc_direction_s_SCSI_MMC_DATA_NONE);
        let cdb = libcdio_sys::mmc_cdb_s { field: cdb };
        let ret = unsafe {
            libcdio_sys::mmc_run_cmd(
                self.cdio.as_ptr(),
                DEFAULT_TIMEOUT_MS,
                &cdb,
                direction,
                buf.len() as u32,
                buf.as_mut_ptr().cast(),
            )
        };
        if ret < 0 {
            return Err(OsError::from(ret));
        }

        return Ok(());

        const DEFAULT_TIMEOUT_MS: u32 = 6000;
    }
}
type Cdb = [u8; 12];

/// error opening MMC device at `{device}`
#[derive(Debug, Display, Error)]
pub struct WithDeviceError {
    pub device: PathBuf,
    pub source: WithDeviceErrorKind,
}
/// Error kind of [`WithDeviceError`]
#[derive(Debug, Display, Error)]
pub enum WithDeviceErrorKind {
    /// device path contains null character
    DeviceHasNullChar(NulError),
    /// could not open device
    CouldNotOpenDevice,
    /// device does not support MMC
    MmcNotSupported,
}

/// could not find any devices that support MMC
#[non_exhaustive]
#[derive(Debug, Display, Error)]
pub struct MmcNotFoundError;

/// could not perform operation on the MMC device
#[non_exhaustive]
#[derive(Debug, Display, Error)]
pub struct MmcOperationError;

/// Direction of MMC data transfer
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, IntoPrimitive)]
enum MmcDirection {
    #[default]
    Read = libcdio_sys::mmc_direction_s_SCSI_MMC_DATA_READ,
    #[allow(unused)]
    Write = libcdio_sys::mmc_direction_s_SCSI_MMC_DATA_WRITE,
}

/// operating system error
#[repr(i32)]
#[non_exhaustive]
#[derive(Debug, Display, Error, FromPrimitive)]
pub enum OsError {
    /// other error: {0}
    #[num_enum(catch_all)]
    Other(i32),
    /// unsupported operation
    Unsupported = libcdio_sys::driver_return_code_t_DRIVER_OP_UNSUPPORTED,
    /// operation not permitted
    OperationNotPermitted = libcdio_sys::driver_return_code_t_DRIVER_OP_NOT_PERMITTED,
    /// bad parameter
    BadParameter = libcdio_sys::driver_return_code_t_DRIVER_OP_BAD_PARAMETER,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a disc drive with mmc"]
    fn with_device() {
        Mmc::with_device(PathBuf::from("/dev/cdrom")).unwrap();
    }
    #[test]
    #[ignore = "requires a disc drive with mmc"]
    fn level() {
        Mmc::new().unwrap().level().unwrap();
    }
}
