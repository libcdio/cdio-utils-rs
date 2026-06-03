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

//! ISO 9660 stat like object.

use std::{
    ffi::{CStr, CString},
    path::Path,
    ptr::NonNull,
};

use libcdio_sys::iso9660_stat_s;

use crate::iso9660::{Iso9660, ds};

/// ISO 9660 file/directory metadata.
pub struct Iso9660Stat {
    pub(crate) stat: NonNull<iso9660_stat_s>,
}

impl Iso9660 {
    /// Read directory at `path` and return a list of [`Iso9660Stat`].
    /// Returns `None` on error.
    pub fn read_dir(&self, path: &Path) -> Option<Vec<Iso9660Stat>> {
        let path = CString::new(path.to_str()?).ok()?;
        let dirlist = unsafe { libcdio_sys::iso9660_ifs_readdir(self.ptr.as_ptr(), path.as_ptr()) };
        if dirlist.is_null() {
            return None;
        }
        // SAFETY: dirlist is not null and the data will be owned by `Iso9660Stat`.
        let dirlist = unsafe { ds::cdiolist_to_vec(dirlist) };
        let dirlist = dirlist
            .into_iter()
            .filter_map(|stat| {
                Some(Iso9660Stat {
                    stat: NonNull::new(stat.cast())?,
                })
            })
            .collect();

        Some(dirlist)
    }
}

impl Iso9660Stat {
    /// Returns the raw filename of the entry.
    /// Returns `None` if the filename has non UTF-8 characters or on error.
    pub fn filename_raw(&self) -> Option<&str> {
        // SAFETY: self.entry is not null since its behind a NonNull<T>
        let name = unsafe { (*self.stat.as_ptr()).filename.as_ptr() };
        if name.is_null() {
            return None;
        };
        // SAFETY: The filename should be a null terminated string
        let name = unsafe { CStr::from_ptr(name).to_str() };

        name.inspect_err(|err| tracing::error!(%err)).ok()
    }
}

impl Drop for Iso9660Stat {
    fn drop(&mut self) {
        unsafe { libcdio_sys::iso9660_stat_free(self.stat.as_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::iso9660::{
        Iso9660,
        tests::{test_joliet_file, test_rockridge_file},
    };

    #[test]
    fn read_dir() {
        let iso = Iso9660::new(test_joliet_file()).unwrap();
        let entries = iso.read_dir(Path::new("/")).unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn filename() {
        let iso = Iso9660::new(test_rockridge_file()).unwrap();
        let entries = iso.read_dir(Path::new("/")).unwrap();
        let names: Vec<_> = entries.iter().map(|e| e.filename_raw().unwrap()).collect();
        assert_eq!(
            &names,
            &[".", "..", "copy", "Copy2", "COPYING", "fd0", "tmp", "zero"]
        );
    }
}
