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

use std::{marker::PhantomData, ptr::NonNull};

use libcdio_sys::udf_dirent_s;

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

        Some(UdfEntry {
            entry: NonNull::new(entry)?,
            _phantom: PhantomData,
        })
    }

    /// Return the root entry of the filesystem, from the given partition.
    /// `None` is returned on error.
    pub fn root_from_partition(&self, partition: u16) -> Option<UdfEntry<'_>> {
        let entry = unsafe { libcdio_sys::udf_get_root(self.udf.as_ptr(), false, partition) };

        Some(UdfEntry {
            entry: NonNull::new(entry)?,
            _phantom: PhantomData,
        })
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
}
