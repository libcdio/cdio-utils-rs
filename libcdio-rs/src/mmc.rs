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

use crate::cdio::Cdio;

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

impl Cdio {
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
    #[ignore = "requires a cd/dvd drive"]
    fn mmc_level() {
        let cdio = Cdio::new().unwrap();
        assert!(cdio.mmc_level().is_some());
    }
}
