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

//! ISO 9660 Rock Ridge extensions.

use libcdio_sys::{bool_3way_t_nope, bool_3way_t_yep};

use crate::iso9660::Iso9660;

impl Iso9660 {
    /// Checks if any file has Rock Ridge extensions. Returns `None` on error.
    /// This can be time consuming, therefore `file_limit` can be provided to
    /// limit the number of files to scan.
    pub fn have_rock_ridge(&self, file_limit: Option<u64>) -> Option<bool> {
        let file_limit = file_limit.unwrap_or(u64::MAX);
        let result = unsafe { libcdio_sys::iso9660_have_rr(self.ptr.as_ptr(), file_limit) };

        #[allow(non_upper_case_globals)]
        match result {
            bool_3way_t_yep => Some(true),
            bool_3way_t_nope => Some(false),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::iso9660::tests::test_rockridge_file;

    use super::*;

    #[test]
    fn have_rock_ridge() {
        let iso = Iso9660::new(test_rockridge_file()).unwrap();
        assert!(iso.have_rock_ridge(None).unwrap());
    }
}
