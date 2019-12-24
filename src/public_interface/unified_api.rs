use crate::public_interface::pairing_ops::PairingApiImplementation;
use crate::public_interface::g1_ops::{G1Api, PublicG1Api};
use crate::public_interface::g2_ops::{G2Api, PublicG2Api};

use crate::errors::ApiError;

// For C style API caller has to preallocate some buffers for results 
pub const PREALLOCATE_FOR_ERROR_BYTES: usize = 256;
pub const PREALLOCATE_FOR_RESULT_BYTES: usize = 768;

use static_assertions::const_assert;
const_assert!(PREALLOCATE_FOR_RESULT_BYTES == crate::public_interface::constants::MAX_MODULUS_BYTE_LEN * 3 * 2);

#[repr(u8)]
pub enum OperationType {
    G1ADD = 1,
    G1MUL = 2,
    G1MULTIEXP = 3,
    G2ADD = 4,
    G2MUL = 5,
    G2MULTIEXP = 6,
    BLS12PAIR = 7,
    BNPAIR = 8,
    MNT4PAIR = 9,
    MNT6PAIR = 10,
}

pub const G1ADD_OPERATION_RAW_VALUE: u8 = OperationType::G1ADD as u8;
pub const G1MUL_OPERATION_RAW_VALUE: u8 = OperationType::G1MUL as u8;
pub const G1MULTIEXP_OPERATION_RAW_VALUE: u8 = OperationType::G1MULTIEXP as u8;

pub const G2ADD_OPERATION_RAW_VALUE: u8 = OperationType::G2ADD as u8;
pub const G2MUL_OPERATION_RAW_VALUE: u8 = OperationType::G2MUL as u8;
pub const G2MULTIEXP_OPERATION_RAW_VALUE: u8 = OperationType::G2MULTIEXP as u8;

pub const BLS12PAIR_OPERATION_RAW_VALUE: u8 = OperationType::BLS12PAIR as u8;
pub const BNPAI_OPERATION_RAW_VALUE: u8 = OperationType::BNPAIR as u8;
pub const MNT4PAIR_OPERATION_RAW_VALUE: u8 = OperationType::MNT4PAIR as u8;
pub const MNT6PAIR_OPERATION_RAW_VALUE: u8 = OperationType::MNT6PAIR as u8;

// This is pure rust API
pub fn perform_operation(operation: OperationType, input: &[u8]) -> Result<Vec<u8>, ApiError> {
    match operation {
        OperationType::G1ADD => {
            PublicG1Api::add_points(&input)
        },
        OperationType::G1MUL => {
            PublicG1Api::mul_point(&input)
        },
        OperationType::G1MULTIEXP => {
            PublicG1Api::multiexp(&input)
        },
        OperationType::G2ADD => {
            PublicG2Api::add_points(&input)
        },
        OperationType::G2MUL => {
            PublicG2Api::mul_point(&input)
        },
        OperationType::G2MULTIEXP => {
            PublicG2Api::multiexp(&input)
        },
        OperationType::BLS12PAIR | OperationType::BNPAIR | OperationType::MNT4PAIR | OperationType::MNT6PAIR => {
            use crate::field::*;
            use crate::public_interface::decode_utils::*;

            let modulus_limbs = {
                let (_, modulus, _) = parse_modulus_and_length(&input)?;
                let modulus_limbs = num_limbs_for_modulus(&modulus)?;

                modulus_limbs
            };

            match operation {
                OperationType::BLS12PAIR => {
                    let result: Result<Vec<u8>, ApiError> = expand_for_modulus_limbs!(modulus_limbs, PairingApiImplementation, input, pair_bls12); 

                    result
                },
                OperationType::BNPAIR => {
                    let result: Result<Vec<u8>, ApiError> = expand_for_modulus_limbs!(modulus_limbs, PairingApiImplementation, input, pair_bn); 

                    result
                },
                OperationType::MNT4PAIR => {
                    let result: Result<Vec<u8>, ApiError> = expand_for_modulus_limbs!(modulus_limbs, PairingApiImplementation, input, pair_mnt4); 

                    result
                },
                OperationType::MNT6PAIR => {
                    let result: Result<Vec<u8>, ApiError> = expand_for_modulus_limbs!(modulus_limbs, PairingApiImplementation, input, pair_mnt6); 

                    result
                },

                _ => {
                    unreachable!()
                }
            }
        }
    }
}

// this is C interface
#[no_mangle]
pub extern "C" fn c_perform_operation(
    op: ::std::os::raw::c_char,
    i: *const ::std::os::raw::c_char,
    i_len: u32,
    o: *mut ::std::os::raw::c_char,
    o_len: *mut u32,
    err: *mut ::std::os::raw::c_char,
    char_len: *mut u32) -> u32 
{            
    use std::io::Write;

    let op_u8: u8 = unsafe { std::mem::transmute(op) };
    let err_out_i8: &mut [i8] = unsafe { std::slice::from_raw_parts_mut(err, PREALLOCATE_FOR_ERROR_BYTES) };
    let mut err_out: &mut [u8] = unsafe { std::mem::transmute(err_out_i8) };

    let operation = match op_u8 {
        G1ADD_OPERATION_RAW_VALUE => {
            OperationType::G1ADD
        },
        G1MUL_OPERATION_RAW_VALUE => {
            OperationType::G1MUL
        },
        G1MULTIEXP_OPERATION_RAW_VALUE => {
            OperationType::G1MULTIEXP
        },
        G2ADD_OPERATION_RAW_VALUE => {
            OperationType::G2ADD
        },
        G2MUL_OPERATION_RAW_VALUE => {
            OperationType::G2MUL
        },
        G2MULTIEXP_OPERATION_RAW_VALUE => {
            OperationType::G2MULTIEXP
        },
        BLS12PAIR_OPERATION_RAW_VALUE => {
            OperationType::BLS12PAIR
        },
        BNPAI_OPERATION_RAW_VALUE => {
            OperationType::BNPAIR
        },
        MNT4PAIR_OPERATION_RAW_VALUE => {
            OperationType::MNT4PAIR
        },
        MNT6PAIR_OPERATION_RAW_VALUE => {
            OperationType::MNT6PAIR
        },
        _ => {
            let written = err_out.write(b"Unknown operation type\0");
            if let Ok(bytes_written) = written {
                unsafe { *char_len = bytes_written as u32 };
            } else {
                unsafe { *char_len = 0u32 };
            }

            return 1u32;
        }
    };

    let input_i8: & [i8] = unsafe { std::slice::from_raw_parts(i, i_len as usize) };
    let input: &[u8] = unsafe { std::mem::transmute(input_i8) };

    let raw_out_i8: &mut [i8] = unsafe { std::slice::from_raw_parts_mut(o, PREALLOCATE_FOR_ERROR_BYTES) };
    let mut raw_out: &mut [u8] = unsafe { std::mem::transmute(raw_out_i8) };

    let result = perform_operation(operation, input);

    match result {
        Ok(result) => {
            let written = raw_out.write(result.as_ref());
            if let Ok(bytes_written) = written {
                unsafe { *o_len = bytes_written as u32 };
                return 0u32;
            }

            let written = err_out.write(b"Failed to write the result\0");
            if let Ok(bytes_written) = written {
                unsafe { *char_len = bytes_written as u32 };
            } else {
                unsafe { *char_len = 0u32 };
            }

            return 1u32;
        },
        Err(error) => {
            use std::error::Error;

            let err_description = error.description();
            let written = err_out.write(err_description.as_bytes());
            if let Ok(bytes_written) = written {
                unsafe { *char_len = bytes_written as u32 };
            } else {
                unsafe { *char_len = 0u32 };
            }

            return 1u32;
        }
    }
} 