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

use displaydoc::Display;
use libcdio_sys::{
    cdio_mmc_level_t_CDIO_MMC_LEVEL_1, cdio_mmc_level_t_CDIO_MMC_LEVEL_2,
    cdio_mmc_level_t_CDIO_MMC_LEVEL_3, cdio_mmc_level_t_CDIO_MMC_LEVEL_NONE,
    cdio_mmc_level_t_CDIO_MMC_LEVEL_WEIRD,
};
use num_enum::TryFromPrimitive;
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
}

/// could not find any devices that support MMC
#[non_exhaustive]
#[derive(Debug, Display, Error)]
pub struct MmcNotFoundError;

/// could not perform operation on the MMC device
#[non_exhaustive]
#[derive(Debug, Display, Error)]
pub struct MmcOperationError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a disc drive with mmc"]
    fn level() {
        Mmc::new().unwrap().level().unwrap();
    }
}
