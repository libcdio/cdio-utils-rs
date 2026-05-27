//! Device or driver related routines.
//!
//! Most methods are implemented on [`Cdio`].
//! As such, you may refer to its documentation.

use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr::{self, NonNull},
    sync::Mutex,
};

use libcdio_sys::{
    driver_id_t, driver_id_t_DRIVER_AIX, driver_id_t_DRIVER_BINCUE, driver_id_t_DRIVER_CDRDAO,
    driver_id_t_DRIVER_DEVICE, driver_id_t_DRIVER_FREEBSD, driver_id_t_DRIVER_LINUX,
    driver_id_t_DRIVER_NETBSD, driver_id_t_DRIVER_NRG, driver_id_t_DRIVER_OSX,
    driver_id_t_DRIVER_SOLARIS, driver_id_t_DRIVER_UNKNOWN, driver_id_t_DRIVER_WIN32,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::cdio::Cdio;

/// MAINTAINER NOTE:
/// A lock guarding a private static named `CdIo_last_driver`.
/// It must be held before invoking any libcdio methods
/// that lead to the mutation this value.
/// As of libcdio v2.3.0, the methods that directly mutate
/// this value are `cdio_init()` and `cdio_destroy()`.
static CDIO_LAST_DRIVER_LOCK: Mutex<()> = Mutex::new(());

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
            let _lock = CDIO_LAST_DRIVER_LOCK.lock().unwrap();
            unsafe { libcdio_sys::cdio_open(source.as_ptr(), driver_id_t::from(driver)) }
        } else {
            let _lock = CDIO_LAST_DRIVER_LOCK.lock().unwrap();
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

    /// Returns a list of connected devices, per the driver in use.
    /// # Returns
    /// `None` is returned if the device list could not be fetched.
    pub fn devices(&self) -> Option<Vec<String>> {
        let devices_pp = unsafe { libcdio_sys::cdio_get_devices(driver_id_t::from(self.driver())) };
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
}

impl Drop for Cdio {
    fn drop(&mut self) {
        let _lock = CDIO_LAST_DRIVER_LOCK.lock().unwrap();
        unsafe { libcdio_sys::cdio_destroy(self.cdio.as_ptr()) }
    }
}

impl Driver {
    /// Returns the driver name.
    pub fn name(self) -> &'static str {
        let driver_name =
            unsafe { libcdio_sys::cdio_get_driver_name_from_id(driver_id_t::from(self)) };

        // SAFETY: driver_name is a static string, therefore valid
        // and not null
        let driver_name: &'static CStr = unsafe { CStr::from_ptr(driver_name) };

        driver_name
            .to_str()
            .expect("driver names should be valid as they are hardcoded")
    }

    /// Returns a description of the driver.
    pub fn description(self) -> &'static str {
        let description = unsafe { libcdio_sys::cdio_driver_describe(driver_id_t::from(self)) };

        // SAFETY: driver_description is a static string, therefore valid
        // and not null
        let description: &'static CStr = unsafe { CStr::from_ptr(description) };

        description
            .to_str()
            .expect("driver descriptions should be valid as they are hardcoded")
    }
}

impl std::fmt::Display for Driver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
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

    #[test]
    fn driver_name() {
        assert_eq!(Driver::Linux.name(), "GNU/Linux");
        assert_eq!(Driver::OsX.name(), "macOS");
    }

    #[test]
    fn driver_description() {
        assert_eq!(
            Driver::Linux.description(),
            "GNU/Linux ioctl and MMC driver"
        );
        assert_eq!(Driver::OsX.description(), "Apple macOS driver");
    }

    #[test]
    #[ignore = "requires a cd/dvd drive"]
    fn devices() {
        let cdio = Cdio::open(None, Driver::Unknown).unwrap();
        assert!(!cdio.devices().unwrap().is_empty());
    }
}
