//! Device or driver related routines.
//!
//! Most methods are implemented on [`Cdio`].
//! As such, you may refer to its documentation.

use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr::{self, NonNull},
};

use libcdio_sys::{
    driver_id_t, driver_id_t_DRIVER_AIX, driver_id_t_DRIVER_BINCUE, driver_id_t_DRIVER_CDRDAO,
    driver_id_t_DRIVER_DEVICE, driver_id_t_DRIVER_FREEBSD, driver_id_t_DRIVER_LINUX,
    driver_id_t_DRIVER_NETBSD, driver_id_t_DRIVER_NRG, driver_id_t_DRIVER_OSX,
    driver_id_t_DRIVER_SOLARIS, driver_id_t_DRIVER_UNKNOWN, driver_id_t_DRIVER_WIN32,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::cdio::Cdio;

/// Represents a cdio driver.
#[repr(u32)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum Driver {
    /// Used as input when we don't care what kind of driver to use.
    Unknown = driver_id_t_DRIVER_UNKNOWN,

    /// AIX driver.
    Aix = driver_id_t_DRIVER_AIX,

    /// FreeBSD driver – includes CAM and ioctl access.
    FreeBsd = driver_id_t_DRIVER_FREEBSD,

    /// NetBSD driver.
    NetBsd = driver_id_t_DRIVER_NETBSD,

    /// GNU/Linux driver.
    Linux = driver_id_t_DRIVER_LINUX,

    /// Sun Solaris driver.
    Solaris = driver_id_t_DRIVER_SOLARIS,

    /// Apple macOS (formerly OS X) driver.
    OsX = driver_id_t_DRIVER_OSX,

    /// Microsoft Windows driver – includes ASPI and ioctl access.
    Win32 = driver_id_t_DRIVER_WIN32,

    /// cdrdao format CD image. Listed before `Bincue` so the code prefers
    /// cdrdao over BIN/CUE when both exist.
    Cdrdao = driver_id_t_DRIVER_CDRDAO,

    /// CDRWIN BIN/CUE format CD image. Listed before `Nrg` so the code
    /// prefers BIN/CUE over NRG when both exist.
    BinCue = driver_id_t_DRIVER_BINCUE,

    /// Nero NRG format CD image.
    Nrg = driver_id_t_DRIVER_NRG,

    /// A composite of the above drivers; should be used last.
    Device = driver_id_t_DRIVER_DEVICE,
}

impl Cdio {
    /// Sets up to read from place specified by `source` and `driver`.
    /// If source is `None`, uses the the default driver.
    /// # Returns
    /// - The cdio object on success.
    /// - `None` on error or no device, or if `source` is invalid.
    pub fn open(source: Option<&Path>, driver: Driver) -> Option<Self> {
        let cdio = if let Some(source) = source {
            let source = CString::new(source.to_str()?).ok()?;
            unsafe { libcdio_sys::cdio_open(source.as_ptr(), driver_id_t::from(driver)) }
        } else {
            unsafe { libcdio_sys::cdio_open(ptr::null(), driver as driver_id_t) }
        };

        Some(Self {
            cdio: NonNull::new(cdio)?,
        })
    }

    /// Return the default CD device.
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

    /// Returns the currently used driver.
    pub fn driver(&self) -> Driver {
        let driver_id = unsafe { libcdio_sys::cdio_get_driver_id(self.cdio.as_ptr()) };

        Driver::try_from(driver_id).expect("cdio->driver_id should be initialized and valid")
    }
}

impl Drop for Cdio {
    fn drop(&mut self) {
        unsafe { libcdio_sys::cdio_destroy(self.cdio.as_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cue_file() -> &'static Path {
        Path::new("../test-data/isofs-m1.cue")
    }

    #[test]
    fn cdio_open() {
        Cdio::open(Some(test_cue_file()), Driver::Unknown).unwrap();
    }

    #[test]
    #[ignore = "requires a cd/dvd drive"]
    fn default_device_test() {
        let cdio = Cdio::open(None, Driver::Device).unwrap();
        assert!(cdio.default_device().is_some());
    }

    #[test]
    fn driver() {
        let cdio = Cdio::open(Some(test_cue_file()), Driver::Unknown).unwrap();
        assert_eq!(cdio.driver(), Driver::BinCue);
    }
}
