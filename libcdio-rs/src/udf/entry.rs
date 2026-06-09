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

//! UDF file/directory entry.

use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

use file_mode::Mode;
use libcdio_sys::udf_dirent_s;
use time::OffsetDateTime;

use crate::udf::Udf;

/// A UDF file/directory entry.
pub struct UdfEntry<'a> {
    entry: NonNull<udf_dirent_s>,
    // udf_dirent_s internally holds references to udf_t
    // thus it is valid for only as long as its parent
    // udf_t is
    _phantom: PhantomData<&'a udf_dirent_s>,
}

impl Udf {
    /// Return the root entry of the filesystem.
    /// `None` is returned on error.
    pub fn root(&self) -> Option<UdfEntry<'_>> {
        // SAFETY: The returned value will be owned by UdfEntry
        let entry = unsafe { libcdio_sys::udf_get_root(self.udf.as_ptr(), true, 0) };

        Some(UdfEntry::new(NonNull::new(entry)?))
    }

    /// Return the root entry of the filesystem, from the given partition.
    /// `None` is returned on error.
    pub fn root_from_partition(&self, partition: u16) -> Option<UdfEntry<'_>> {
        let entry = unsafe { libcdio_sys::udf_get_root(self.udf.as_ptr(), false, partition) };

        Some(UdfEntry::new(NonNull::new(entry)?))
    }
}

impl UdfEntry<'_> {
    /// Return the modification time.
    /// Returns `None` in case the value is invalid.
    pub fn modify_time(&self) -> Option<OffsetDateTime> {
        // SAFETY: Returns -1 in case the value is invalid, checked immediately below
        let time = unsafe { libcdio_sys::udf_get_modification_time(self.entry.as_ptr()) };
        if time == -1 {
            return None;
        }

        OffsetDateTime::from_unix_timestamp(time).ok()
    }

    /// Return the filename.
    /// `None` is returned if the filename has non UTF-8 characters, or on an unexpected error.
    pub fn filename(&self) -> Option<&str> {
        const CURRENT_DIR_FILENAME: &str = ".";

        // SAFETY: self.entry is non null, therefore this method should not return null
        let filename = unsafe { libcdio_sys::udf_get_filename(self.entry.as_ptr()) };
        if filename.is_null() {
            tracing::error!("udf_get_filename() returned an unexpected NULL");
            return None;
        }
        let filename = unsafe { CStr::from_ptr(filename) };
        // filename returns an empty string after opening the root directory.
        // this probably represents "."
        if filename.is_empty() {
            return Some(CURRENT_DIR_FILENAME);
        }

        filename.to_str().ok()
    }

    /// Return the next entry, or `None` on reaching end of file or on error.
    pub fn next(self) -> Option<Self> {
        // SAFETY: This always frees the passed entry, therefore prevent self's destructor
        // from running
        let next_entry = unsafe { libcdio_sys::udf_readdir(self.entry.as_ptr()) };
        std::mem::forget(self);

        NonNull::new(next_entry).map(Self::new)
    }

    /// Open `self` and return the first entry.
    /// Returns `None` if `self` is not a directory, or on error.
    // TODO: Add unit test, need a UDF file with directory that works with libcdio
    pub fn open_dir(&self) -> Option<Self> {
        let sub_entry = unsafe { libcdio_sys::udf_opendir(self.entry.as_ptr()) };

        Some(Self::new(NonNull::new(sub_entry)?))
    }

    /// Is the entry a directory.
    pub fn is_dir(&self) -> bool {
        unsafe { libcdio_sys::udf_is_dir(self.entry.as_ptr()) }
    }

    /// Return the file length.
    pub fn file_length(&self) -> u64 {
        // SAFETY: entry is not null, making this function infallible
        unsafe { libcdio_sys::udf_get_file_length(self.entry.as_ptr()) }
    }

    /// Return the POSIX file mode.
    pub fn mode(&self) -> Mode {
        let mode = unsafe { libcdio_sys::udf_get_posix_filemode(self.entry.as_ptr()) };
        Mode::new(mode, u32::MAX)
    }

    fn new(entry: NonNull<udf_dirent_s>) -> Self {
        Self {
            entry,
            _phantom: PhantomData,
        }
    }
}

impl Drop for UdfEntry<'_> {
    fn drop(&mut self) {
        // SAFETY: Infallible function
        let _ = unsafe { libcdio_sys::udf_dirent_free(self.entry.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use crate::udf::tests::test_udf_file;

    use super::*;

    #[test]
    fn root() {
        let udf = Udf::new(test_udf_file()).unwrap();
        udf.root().unwrap();
    }

    #[test]
    fn root_from_partition() {
        let udf = Udf::new(test_udf_file()).unwrap();
        udf.root_from_partition(0).unwrap();
    }

    #[test]
    fn modify_time() {
        let udf = Udf::new(test_udf_file()).unwrap();
        let modify_time = udf.root().unwrap().modify_time().unwrap();
        assert_eq!(modify_time, datetime!(2014-02-20 1:26:20.0 +00:00:00));
    }

    #[test]
    fn filename() {
        let udf = Udf::new(test_udf_file()).unwrap();
        let root = udf.root().unwrap();
        assert_eq!(root.filename().unwrap(), "/");
    }

    #[test]
    fn next() {
        let udf = Udf::new(test_udf_file()).unwrap();
        let root = udf.root().unwrap();
        let next = root.next().unwrap();
        assert_eq!(next.filename().unwrap(), ".");

        let next = next.next().unwrap();
        assert_eq!(next.filename().unwrap(), "FéжΘvrier");
    }

    #[test]
    fn is_dir() {
        let udf = Udf::new(test_udf_file()).unwrap();
        let root = udf.root().unwrap();
        assert!(root.is_dir());
    }

    #[test]
    fn file_length() {
        let udf = Udf::new(test_udf_file()).unwrap();
        let root = udf.root().unwrap();
        let file = root.next().unwrap().next().unwrap();
        assert_eq!(file.file_length(), 10);
    }

    #[test]
    fn mode() {
        let udf = Udf::new(test_udf_file()).unwrap();
        let root = udf.root().unwrap();
        let entry = root.next().unwrap();
        let entry = entry.next().unwrap();
        assert_eq!(&entry.mode().to_string(), "-r-xr-xr-x");
    }
}
