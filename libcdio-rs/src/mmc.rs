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
//!
//! Most methods are implemented on [`Cdio`].
//! As such, you may refer to its documentaiton page.

use libcdio_sys::{
    cdio_mmc_level_t_CDIO_MMC_LEVEL_1, cdio_mmc_level_t_CDIO_MMC_LEVEL_2,
    cdio_mmc_level_t_CDIO_MMC_LEVEL_3, cdio_mmc_level_t_CDIO_MMC_LEVEL_NONE,
    cdio_mmc_level_t_CDIO_MMC_LEVEL_WEIRD,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::drive::Drive;

/// Represents the MMC Level.
#[repr(u32)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum MmcLevel {
    /// Unknown non standard MMC
    Weird = cdio_mmc_level_t_CDIO_MMC_LEVEL_WEIRD,

    /// MMC-1
    Mmc1 = cdio_mmc_level_t_CDIO_MMC_LEVEL_1,

    /// MMC-2
    Mmc2 = cdio_mmc_level_t_CDIO_MMC_LEVEL_2,

    /// MMC-3
    Mmc3 = cdio_mmc_level_t_CDIO_MMC_LEVEL_3,
    // CDIO_MMC_LEVEL_NONE can be represented using an Option
    // and therefore, can be omitted here
}

impl Drive {
    /// Get the MMC level supported by the device.
    /// Returns `None` if the device doesn't support MMC.
    pub fn mmc_level(&self) -> Option<MmcLevel> {
        let mmc_level = unsafe { libcdio_sys::mmc_get_drive_mmc_cap(self.cdio.as_ptr()) };
        if mmc_level == cdio_mmc_level_t_CDIO_MMC_LEVEL_NONE {
            return None;
        }

        let mmc_level = MmcLevel::try_from(mmc_level)
            .expect("mmc_get_drive_mmc_cap should return a valid mmc_level_t");

        Some(mmc_level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a disc drive with mmc"]
    fn mmc_level() {
        let drive = Drive::new().unwrap();
        assert!(drive.mmc_level().is_some());
    }
}
