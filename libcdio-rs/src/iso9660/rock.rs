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

use std::ffi::CStr;

use file_mode::Mode;
use libcdio_sys::{bool_3way_t_nope, bool_3way_t_yep};

use crate::iso9660::{Iso9660, stat::Iso9660Stat};

/// ISO 9660 Rock Ridge extensions.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct RockRidge {
    /// Group ID
    pub group_id: u32,
    /// Number of hard links
    pub hard_links: u32,
    /// Unix file mode
    pub mode: Mode,
    /// Symlink target
    pub symlink_to: Option<String>,
    /// User ID
    pub user_id: u32,
}

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

impl Iso9660Stat {
    /// Rock Ridge extensions.
    /// `None` is returned if Rock ridge extensions are missing, or if it
    /// could not be determined.
    pub fn rock_ridge(&self) -> Option<RockRidge> {
        let rock = unsafe { (*self.stat.as_ptr()).rr };
        if rock.b3_rock != bool_3way_t_yep {
            return None;
        }

        Some(RockRidge {
            group_id: rock.st_gid,
            hard_links: rock.st_nlinks,
            mode: Mode::new(rock.st_mode, u32::MAX),
            symlink_to: {
                if rock.psz_symlink.is_null() {
                    None
                } else {
                    let symlink = unsafe { CStr::from_ptr(rock.psz_symlink) };
                    symlink
                        .to_str()
                        .ok()
                        .filter(|link| !link.is_empty())
                        .map(ToString::to_string)
                }
            },
            user_id: rock.st_uid,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::iso9660::tests::{test_joliet_file, test_rockridge_file};

    use super::*;

    #[test]
    fn have_rock_ridge() {
        let iso = Iso9660::new(test_rockridge_file()).unwrap();
        assert!(iso.have_rock_ridge(None).unwrap());
    }

    #[test]
    fn rock_ridge() {
        let iso = Iso9660::new(test_rockridge_file()).unwrap();
        let stat = iso.stat(Path::new("/COPYING")).unwrap();
        assert!(stat.rock_ridge().is_some());

        let iso = Iso9660::new(test_joliet_file()).unwrap();
        let stat = iso.stat(Path::new("/libcdio/COPYING")).unwrap();
        assert!(stat.rock_ridge().is_none());
    }

    #[test]
    fn mode() {
        let iso = Iso9660::new(test_rockridge_file()).unwrap();

        let stat = iso.stat(Path::new("/zero")).unwrap();
        let mode = stat.rock_ridge().unwrap().mode;
        assert_eq!(&mode.to_string(), "cr--r--r--");

        let stat = iso.stat(Path::new("/fd0")).unwrap();
        let mode = stat.rock_ridge().unwrap().mode;
        assert_eq!(&mode.to_string(), "br--r--r--");

        let stat = iso.stat(Path::new("/Copy2")).unwrap();
        let mode = stat.rock_ridge().unwrap().mode;
        assert_eq!(&mode.to_string(), "lr-xr-xr-x");

        let stat = iso.stat(Path::new("/copy")).unwrap();
        let mode = stat.rock_ridge().unwrap().mode;
        assert_eq!(&mode.to_string(), "dr-xr-xr-x");
    }

    #[test]
    fn symlink_to() {
        let iso = Iso9660::new(test_rockridge_file()).unwrap();

        let stat = iso.stat(Path::new("/COPYING")).unwrap();
        let rock = stat.rock_ridge().unwrap();
        assert!(rock.symlink_to.is_none());

        let stat = iso.stat(Path::new("/Copy2")).unwrap();
        let rock = stat.rock_ridge().unwrap();
        assert_eq!(rock.symlink_to.unwrap(), "COPYING");

        let stat = iso.stat(Path::new("/tmp/COPYING")).unwrap();
        let rock = stat.rock_ridge().unwrap();
        assert_eq!(rock.symlink_to.unwrap(), "../copying/COPYING");
    }

    #[test]
    fn hard_links() {
        let iso = Iso9660::new(test_rockridge_file()).unwrap();
        let stat = iso.stat(Path::new("/COPYING")).unwrap();
        let rock = stat.rock_ridge().unwrap();
        assert_eq!(rock.hard_links, 1);

        let stat = iso.stat(Path::new("/copy")).unwrap();
        let rock = stat.rock_ridge().unwrap();
        assert_eq!(rock.hard_links, 2);
    }

    #[test]
    fn user_id() {
        let iso = Iso9660::new(test_rockridge_file()).unwrap();
        let stat = iso.stat(Path::new("/COPYING")).unwrap();
        let rock = stat.rock_ridge().unwrap();
        assert_eq!(rock.user_id, 0);
    }

    #[test]
    fn group_id() {
        let iso = Iso9660::new(test_rockridge_file()).unwrap();
        let stat = iso.stat(Path::new("/COPYING")).unwrap();
        let rock = stat.rock_ridge().unwrap();
        assert_eq!(rock.group_id, 0);
    }
}
