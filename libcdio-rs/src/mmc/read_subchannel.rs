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

//! Routines based on MMC `READ SUB-CHANNEL`.

use crate::{
    Mmc,
    mmc::{Cdb, MmcDirection, OsError},
};

/// Routines based on MMC `READ SUB-CHANNEL`.
impl Mmc {
    #[allow(unused)]
    /// Perform an MMC `READ SUB-CHANNEL`.
    fn read_subchannel(
        &self,
        address_format: AddressFormat,
        param: Option<SubchannelParameter>,
    ) -> Result<SubchannelData, OsError> {
        let mut data = SubchannelData::default();
        let mut cdb = Cdb::default();
        cdb[0] = OPCODE;
        if let AddressFormat::Time = address_format {
            cdb[1] = 1 << 1;
        }
        if let Some(param) = param {
            cdb[2] = 1 << 6; // subq
            cdb[3] = param.discriminant(); // parameter list
            if let SubchannelParameter::Isrc { track_number } = param {
                cdb[6] = track_number;
            }
        } else {
            // it errors if the parameter list is set to zero (i.e reserved)
            cdb[3] = 1;
        }

        cdb[7..9].copy_from_slice(&(data.len() as u16).to_be_bytes());

        self.run_command(Some(MmcDirection::Read), &mut data, cdb)?;

        return Ok(data);

        const OPCODE: u8 = 0x42;
    }
}

/// Format to use for address fields in the response of `READ SUB-CHANNEL`
#[allow(unused)]
enum AddressFormat {
    Lba,
    Time,
}

#[repr(u8)]
#[allow(unused)]
#[derive(Clone, Copy, Debug)]
enum SubchannelParameter {
    CdCurrentPosition = 0x1,
    Mcn = 0x2,
    Isrc { track_number: u8 } = 0x3,
}
impl SubchannelParameter {
    fn discriminant(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        // Source:
        // https://doc.rust-lang.org/stable/std/mem/fn.discriminant.html#accessing-the-numeric-value-of-the-discriminant
        unsafe { *(&raw const *self).cast::<u8>() }
    }
}

type SubchannelData = [u8; 24];
