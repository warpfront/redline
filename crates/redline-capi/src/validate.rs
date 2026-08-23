// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Host-side validation of retained PM4 command streams before IB creation.
//!
//! Every dispatch reads its kernel descriptor and code through addresses held
//! in SH registers (COMPUTE_PGM_LO/HI). A stream that dispatches with those
//! registers zero makes the GPU fetch from address 0: an SQC-data page fault
//! followed by MES queue-removal failures and a full device reset (see
//! ROCm/ROCm#6529). Construction guards reject zero code entries and null
//! kernargs, but state corruption and future encoder changes are invisible to
//! those guards — so the finalized stream itself is checked here, turning a
//! GPU reset into a host-side `RL_ERR_COMPILE`.

use std::fmt;

const PACKET3_SET_SH_REG: u32 = 0x76;
const PACKET3_DISPATCH_DIRECT: u32 = 0x15;

/// Legacy compute SH-register offsets, shared by the GFX10/GFX11 and GFX12
/// encoders.
const COMPUTE_PGM_LO: u32 = 0x20c;
const COMPUTE_PGM_HI: u32 = 0x20d;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StreamValidationError {
    /// A SET_SH_REG packet writes zero to COMPUTE_PGM_LO.
    ZeroProgramAddressWrite { dword_index: usize },
    /// A DISPATCH_DIRECT executes without a nonzero program address written
    /// earlier in the same stream.
    DispatchWithoutProgramAddress { dword_index: usize },
    /// The stream is not well-formed packet3 data.
    MalformedPacket { dword_index: usize },
}

impl fmt::Display for StreamValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ZeroProgramAddressWrite { dword_index } => write!(
                formatter,
                "dword {dword_index}: SET_SH_REG writes zero to COMPUTE_PGM_LO"
            ),
            Self::DispatchWithoutProgramAddress { dword_index } => write!(
                formatter,
                "dword {dword_index}: DISPATCH_DIRECT with no nonzero COMPUTE_PGM_LO/HI earlier in the stream"
            ),
            Self::MalformedPacket { dword_index } => {
                write!(formatter, "dword {dword_index}: malformed packet3 header")
            }
        }
    }
}

/// Walk one retained command stream, tracking the last COMPUTE_PGM_LO/HI
/// values written by SET_SH_REG packets (mirroring hardware SH-register
/// persistence within an indirect buffer, which the stateful elision relies
/// on). Rejects any stream that can reach the shader with a zero program
/// address.
pub(crate) fn validate_dispatch_stream(dwords: &[u32]) -> Result<(), StreamValidationError> {
    let mut pgm_lo: u32 = 0;
    let mut pgm_hi: u32 = 0;
    let mut cursor = 0_usize;
    while cursor < dwords.len() {
        let header = dwords[cursor];
        // packet3: type 3 in bits 31:30, body-dword-count-minus-one in 29:16,
        // opcode in 15:8. Only packet3 is emitted by these encoders.
        if header >> 30 != 3 {
            return Err(StreamValidationError::MalformedPacket {
                dword_index: cursor,
            });
        }
        let body = ((header >> 16) & 0x3fff) as usize + 1;
        let opcode = (header >> 8) & 0xff;
        let end = cursor + 1 + body;
        if end > dwords.len() {
            return Err(StreamValidationError::MalformedPacket {
                dword_index: cursor,
            });
        }
        match opcode {
            PACKET3_SET_SH_REG => {
                if body < 1 {
                    return Err(StreamValidationError::MalformedPacket {
                        dword_index: cursor,
                    });
                }
                let first = dwords[cursor + 1];
                for (offset, value) in dwords[cursor + 2..end].iter().copied().enumerate() {
                    let register = first + offset as u32;
                    if register == COMPUTE_PGM_LO {
                        if value == 0 {
                            return Err(StreamValidationError::ZeroProgramAddressWrite {
                                dword_index: cursor + 2 + offset,
                            });
                        }
                        pgm_lo = value;
                    } else if register == COMPUTE_PGM_HI {
                        pgm_hi = value;
                    }
                }
            }
            PACKET3_DISPATCH_DIRECT => {
                if pgm_lo == 0 && pgm_hi == 0 {
                    return Err(StreamValidationError::DispatchWithoutProgramAddress {
                        dword_index: cursor,
                    });
                }
            }
            _ => {}
        }
        cursor = end;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet3(opcode: u32, body: &[u32]) -> Vec<u32> {
        let mut words = vec![(3 << 30) | (((body.len() as u32) - 1) << 16) | (opcode << 8) | 2];
        words.extend_from_slice(body);
        words
    }

    fn set_sh_reg(first: u32, values: &[u32]) -> Vec<u32> {
        let mut body = vec![first];
        body.extend_from_slice(values);
        packet3(PACKET3_SET_SH_REG, &body)
    }

    fn dispatch_direct() -> Vec<u32> {
        packet3(PACKET3_DISPATCH_DIRECT, &[4, 1, 1, 0x0f])
    }

    #[test]
    fn accepts_acquire_then_program_then_dispatch() {
        let mut stream = packet3(0x58, &[0, u32::MAX, 0xff, 0, 0, 4, 0x1ffff]); // ACQUIRE_MEM
        stream.extend(set_sh_reg(COMPUTE_PGM_LO, &[0x0012_3400, 0xab]));
        stream.extend(dispatch_direct());
        assert_eq!(validate_dispatch_stream(&stream), Ok(()));
    }

    #[test]
    fn rejects_dispatch_without_any_program_write() {
        let mut stream = packet3(0x58, &[0, u32::MAX, 0xff, 0, 0, 4, 0x1ffff]);
        stream.extend(dispatch_direct());
        let dispatch_index = stream.len() - 5;
        assert_eq!(
            validate_dispatch_stream(&stream),
            Err(StreamValidationError::DispatchWithoutProgramAddress {
                dword_index: dispatch_index
            })
        );
    }

    #[test]
    fn rejects_zero_program_address_write() {
        let mut stream = set_sh_reg(COMPUTE_PGM_LO, &[0x0012_3400, 0xab]);
        let zero_write_index = stream.len() + 2;
        stream.extend(set_sh_reg(COMPUTE_PGM_LO, &[0]));
        assert_eq!(
            validate_dispatch_stream(&stream),
            Err(StreamValidationError::ZeroProgramAddressWrite {
                dword_index: zero_write_index
            })
        );
    }

    /// #6529 construction-class shape: a valid first dispatch, then a
    /// SET_SH_REG that zeroes COMPUTE_PGM_LO (as a bad mid-IB restore would).
    /// The walker must refuse at the zero write — before any second
    /// DISPATCH_DIRECT can fetch from address 0.
    ///
    /// Distinct from `rejects_zero_program_address_write` (zero write with no
    /// prior dispatch) and `rejects_dispatch_without_any_program_write`
    /// (dispatch with never a program write). Catches a regression that only
    /// rejects zero PGM_LO when no prior nonzero value was tracked.
    #[test]
    fn rejects_zero_pgm_lo_write_after_first_dispatch() {
        let mut stream = set_sh_reg(COMPUTE_PGM_LO, &[0x0012_3400, 0xab]);
        stream.extend(dispatch_direct());
        let zero_write_index = stream.len() + 2;
        stream.extend(set_sh_reg(COMPUTE_PGM_LO, &[0]));
        // Second dispatch is present so a lax walker that only checked the
        // final SH state at each DISPATCH would still see the zero — but the
        // correct behaviour is to fail at the write itself.
        stream.extend(dispatch_direct());
        assert_eq!(
            validate_dispatch_stream(&stream),
            Err(StreamValidationError::ZeroProgramAddressWrite {
                dword_index: zero_write_index
            })
        );
    }

    #[test]
    fn elided_second_dispatch_survives_since_state_persists() {
        // Mirrors the stateful encoder: the second dispatch of the same kernel
        // emits no PGM write, and the hardware register retains the value.
        let mut stream = set_sh_reg(COMPUTE_PGM_LO, &[0x0012_3400, 0xab]);
        stream.extend(dispatch_direct());
        stream.extend(packet3(0x46, &[0x407])); // EVENT_WRITE boundary
        stream.extend(dispatch_direct());
        assert_eq!(validate_dispatch_stream(&stream), Ok(()));
    }

    #[test]
    fn rejects_malformed_header_and_truncated_body() {
        assert_eq!(
            validate_dispatch_stream(&[0x1234_5678]),
            Err(StreamValidationError::MalformedPacket { dword_index: 0 })
        );
        let mut stream = set_sh_reg(COMPUTE_PGM_LO, &[0x0012_3400, 0xab]);
        stream.push((3 << 30) | (7 << 16) | (PACKET3_SET_SH_REG << 8) | 2); // claims 8 body dwords
        assert!(matches!(
            validate_dispatch_stream(&stream),
            Err(StreamValidationError::MalformedPacket { .. })
        ));
    }
}
