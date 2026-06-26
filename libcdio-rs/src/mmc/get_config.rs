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

//! SCSI MMC (MultiMedia Commands) GET CONFIGURATION command.

use displaydoc::Display;
use num_enum::TryFromPrimitive;
use tracing::error;

use crate::mmc::Mmc;

/// A base set of functions for specific drive/media combination.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MmcProfile {
    pub active: bool,
    pub kind: ProfileKind,
}

// WARNING: Changes to the doc comments can affect the type's display output!
/// Represents the complete set of MMC feature profiles for optical disc drives.
/// Each variant corresponds to a specific media type and recording capability.
#[repr(u16)]
#[non_exhaustive]
#[derive(
    Clone, Copy, Debug, Default, Display, Eq, Hash, Ord, PartialEq, PartialOrd, TryFromPrimitive,
)]
pub enum ProfileKind {
    /// Non-removable disk
    NonRemovable = 0x0001,

    /// Removable disk
    Removable = 0x0002,

    /// Magneto-Optical Erasable disk
    MoErasable = 0x0003,

    /// Optical Write-Once
    OpticalWriteOnce = 0x0004,

    /// Advance Storage - Magneto-Optical
    AsMo = 0x0005,

    /// CD-ROM
    CdRom = 0x0008,

    /// CD-R
    CdR = 0x0009,

    /// CD-RW
    CdRw = 0x000A,

    /// DVD-ROM
    DvdRom = 0x0010,

    /// DVD-R Sequential Recording
    DvdRSeqRec = 0x0011,

    /// DVD-RAM
    DvdRam = 0x0012,

    /// DVD-RW Restricted Overwrite
    DvdRwRo = 0x0013,

    /// DVD-RW Sequential Recording
    DvdRwSeqRec = 0x0014,

    /// DVD-R Dual Layer Sequential recording
    DvdRDlSeqRec = 0x0015,

    /// DVD-R Dual Layer Jump Recording
    DvdRDlJmpRec = 0x0016,

    /// DVD-RW Dual Layer
    DvdRwDl = 0x0017,

    /// DVD-Download Disc Recording
    DvdDownDiscRec = 0x0018,

    /// DVD+RW
    DvdPlusRw = 0x001A,

    /// DVD+R
    DvdPlusR = 0x001B,

    /// DDCD-ROM
    DdcdRom = 0x0020,

    /// DDCD-R
    DdcdR = 0x0021,

    /// DDCD-RW
    DdcdRw = 0x0022,

    /// DVD+RW Dual Layer
    DvdPlusRwDl = 0x002A,

    /// DVD+R Double Layer
    DvdPlusRDl = 0x002B,

    /// BD-ROM
    BdRom = 0x0040,

    /// BD-R Sequential Recording Mode
    BdRSeqRec = 0x0041,

    /// BD-R Random Recording Mode
    BdRRandRec = 0x0042,

    /// BD-RE
    BdRw = 0x0043,

    /// HD DVD-ROM
    HdDvdRom = 0x0050,

    /// HD DVD-R
    HdDvdR = 0x0051,

    /// HD DVD-RAM
    HdDvdRam = 0x0052,

    /// HD DVD-RW
    HdDvdRw = 0x0053,

    /// HD DVD-R Dual Layer
    HdDvdRDl = 0x0058,

    /// HD DVD-RW Dual Layer
    HdDvdRwDl = 0x0059,

    /// The Drive does not conform to any Profile
    #[default]
    NonConform = 0xFFFF,
}

// WARNING: Changes to the doc comments can affect the type's display output!
/// Physical interface standard reported by MMC.
#[repr(u32)]
#[non_exhaustive]
#[derive(
    Clone, Copy, Debug, Default, Display, Eq, Hash, Ord, PartialEq, PartialOrd, TryFromPrimitive,
)]
pub enum MmcInterface {
    #[default]
    /// Unspecified
    Unspecified = 0x0,
    /// SCSI
    Scsi = 0x1,
    /// ATAPI
    Atapi = 0x2,
    /// IEEE 1394
    Ieee1394 = 0x3,
    /// IEEE 1394A
    Ieee1394A = 0x4,
    /// Fibre Channel
    FibreChannel = 0x5,
    /// IEEE 1394B
    Ieee1394B = 0x6,
    /// Serial ATAPI
    SerialAtapi = 0x7,
    /// USB (both 1.1 and 2.0)
    Usb = 0x8,
    /// Vendor Unique
    VendorUnique = 0xffff,
}

/// Methods related to the `GET CONFIGURATION` command.
impl Mmc {
    /// Return type to request data pertaining only to a single feature,
    /// identified by a number.
    const GET_CONF_RET_TYPE_TWO: u32 = 0x2;
    const RESP_BUF_SIZE: usize = 4096;
    const TIMEOUT_MILLIS: u32 = 6000;
    const FEAT_DESC_INDEX: usize = 8;

    /// Profiles supported by the drive (`000h`).
    /// Returns `None` on error.
    pub fn profiles(&self) -> Option<Vec<MmcProfile>> {
        let mut profiles = Vec::new();
        const GET_CONF_FEAT_PROF_LIST: u32 = 0x0;
        let mut buf = [0_u8; Self::RESP_BUF_SIZE];
        let data_len = self.get_configuration(&mut buf, GET_CONF_FEAT_PROF_LIST)?;

        let mut prof_desc = Self::FEAT_DESC_INDEX + 4;
        while prof_desc < data_len {
            let prof_num = read_u16(&buf[prof_desc..]);
            let Ok(kind) = ProfileKind::try_from(prof_num)
                .inspect_err(|err| error!(?err, prof_num, "invalid profile number from mmc"))
            else {
                continue;
            };
            profiles.push(MmcProfile {
                active: buf[prof_desc + 2] & 0b1 != 0,
                kind,
            });
            prof_desc += 4;
        }

        Some(profiles)
    }

    /// The physical interface in use (`001h`).
    /// <div class="warning">
    ///
    /// **NOTE**: It is possible that more than one physical interface exists between the Host and Drive, e.g., an
    /// IEEE1394 Host connecting to an ATAPI bridge to an ATAPI Drive. The Drive may not be aware of
    /// interfaces beyond the ATAPI.
    ///
    /// </div>
    /// `None` is returned on error.
    pub fn interface(&self) -> Option<MmcInterface> {
        const GET_CONF_FEAT_CORE: u32 = 0x1;
        let mut buf = [0_u8; Self::RESP_BUF_SIZE];
        let _ = self.get_configuration(&mut buf, GET_CONF_FEAT_CORE)?;

        let iface_num = read_u32(&buf[Self::FEAT_DESC_INDEX + 4..]);
        MmcInterface::try_from(iface_num)
            .inspect_err(|err| error!(?err, iface_num, "got invalid interface value from mmc"))
            .ok()
    }

    /// Run MMC `GET CONFIGURATION`, requesting information on `feature`.
    /// Returns the size of the data in the allocated buffer, or `None` on error.
    fn get_configuration(&self, buf: &mut [u8], feature: u32) -> Option<usize> {
        let retval = unsafe {
            libcdio_sys::mmc_get_configuration(
                self.cdio.cdio.as_ptr(),
                buf.as_mut_ptr().cast(),
                buf.len() as u32,
                Self::GET_CONF_RET_TYPE_TWO,
                feature,
                Self::TIMEOUT_MILLIS,
            )
        };

        if retval != 0 {
            error!(retval, "libcdio C mmc_get_configuration() returned error");
            return None;
        };

        let data_len = read_u32(buf) as usize;
        // data_len doesn't include its own length..
        if data_len + size_of::<u32>() > buf.len() {
            error!(
                data_len,
                buflen = buf.len(),
                "mmc response data length exceeds input buffer"
            );
            return None;
        }

        Some(data_len)
    }
}

/// Parse a big endian u16 out of the next two bytes.
fn read_u16(val: &[u8]) -> u16 {
    let val = *val
        .first_chunk()
        .expect("mmc data buffer should be sufficiently large");
    u16::from_be_bytes(val)
}
/// Parse a big endian u32 out of the next four bytes.
fn read_u32(val: &[u8]) -> u32 {
    let val = *val
        .first_chunk()
        .expect("mmc data buffer should be sufficiently large");
    u32::from_be_bytes(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a disc drive with mmc"]
    fn profiles() {
        let mmc = Mmc::new().unwrap();
        assert!(!mmc.profiles().unwrap().is_empty());
    }

    #[test]
    #[ignore = "requires a disc drive with mmc"]
    fn interface() {
        let mmc = Mmc::new().unwrap();
        mmc.interface().unwrap();
    }
}
