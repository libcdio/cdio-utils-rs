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

use displaydoc::Display;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;
use tracing::debug;
use winnow::{
    Parser,
    binary::{be_u16, length_take, u8},
    error::{ContextError, StrContext},
};

use crate::{
    Mmc,
    mmc::{Cdb, MmcDirection, OsError},
};

/// Routines based on MMC `READ SUB-CHANNEL`.
impl Mmc {
    /// Get the status of audio play operations.
    pub fn audio_status(&self) -> Result<MmcAudioStatus, MmcAudioStatusError> {
        let data = self.read_subchannel(AddressFormat::Lba, None)?;
        let status = parse_header(&mut data.as_slice())?;

        status.ok_or(MmcAudioStatusError::NotSupported)
    }

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

/// The status of audio play operations
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, TryFromPrimitive)]
pub enum MmcAudioStatus {
    PlayInProgress = 0x11,
    PlayPaused = 0x12,
    PlayCompleted = 0x13,
    PlayStopped = 0x14,

    #[default]
    NoStatus = 0x15,
}

/// error getting audio status via `READ SUB-CHANNEL`
#[derive(Debug, Display, Error)]
pub enum MmcAudioStatusError {
    /// The device not support reporting audio status
    NotSupported,

    /// operating system returned an error: {0}
    Os(#[from] OsError),

    /// invalid response from command
    InvalidResponse(String),
}
impl From<MmcSubchannelError> for MmcAudioStatusError {
    fn from(value: MmcSubchannelError) -> Self {
        match value {
            MmcSubchannelError::Os(os_error) => Self::Os(os_error),
            MmcSubchannelError::InvalidResponse(error) => Self::InvalidResponse(error),
        }
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

fn parse_header(input: &mut &[u8]) -> Result<Option<MmcAudioStatus>, MmcSubchannelError> {
    debug!(header = ?input, "parse_header()");
    let (_, audio_status, remainder) = (u8::<_, ContextError>, u8, length_take(be_u16))
        .context(StrContext::Label("READ SUB-CHANNEL response header"))
        .parse_next(input)?;
    *input = remainder;

    (audio_status != 0)
        .then(|| MmcAudioStatus::try_from(audio_status))
        .transpose()
        .map_err(MmcSubchannelError::from)
}

/// error from a `READ SUB-CHANNEL` command
#[non_exhaustive]
#[derive(Debug, Display, Error)]
pub enum MmcSubchannelError {
    /// operating system returned an error
    Os(#[from] OsError),

    /// invalid response from mmc command: {0}
    InvalidResponse(String),
}
impl From<ContextError> for MmcSubchannelError {
    fn from(err: ContextError) -> Self {
        Self::InvalidResponse(err.to_string())
    }
}
impl<T: TryFromPrimitive> From<TryFromPrimitiveError<T>> for MmcSubchannelError {
    fn from(err: TryFromPrimitiveError<T>) -> Self {
        Self::InvalidResponse(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use tracing::info;

    use super::*;

    #[test_log::test(test)]
    #[ignore = "requires a disc drive with mmc"]
    fn audio_status() {
        let audio_status = Mmc::new().unwrap().audio_status();
        info!(?audio_status);
        assert!(matches!(
            audio_status,
            Ok(_) | Err(MmcAudioStatusError::NotSupported)
        ));
    }
}
