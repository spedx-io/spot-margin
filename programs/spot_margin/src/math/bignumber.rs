//! Big Number Data Types

#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::manual_range_contains)]

use crate::error::ErrorCode::BigNumberConversionError;
use borsh::{BorshDeserialize, BorshSerialize};
use std::borrow::BorrowMut;
use std::convert::TryInto;
use std::io::{
    Error,
    ErrorKind,
    Write,
    Read
};
use std::mem::size_of;
use uint::construct_uint;

use crate::error::SpedXSpotResult;

// macros
macro_rules! impl_borsh_serialize_for_bn {
    ($type: ident) => {
        impl BorshSerialize for $type {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
                // converting the bytes to little endian bytes
                let mut bytes = self.to_le_bytes();
                // writing the data to the blockchain
                writer.write_all(&mut bytes)
            }
        }
    };
}

macro_rules! impl_borsh_deserialize_for_bn {
    ($type: ident) => {
        impl BorshDeserialize for $type {
            #[inline]
            fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                // the parameter buf is an array of u8 bytes. if the size of buf is more than the size of the type,
                // then we declare it as invalid input, as the deserialized value is more than the pre-deserialized value
                if buf.len() < size_of::<$type>() {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        "Unexpected length of input"
                    ));
                }

                // we convert the type from little endian bytes 
                let res = $type::from_le_bytes(buf[..size_of::<$type>()].try_into().unwrap());
                // we update the buf to the remaining bytes
                *buf = &buf[size_of::<$type>()..];
                Ok(res)
            }

            fn deserialize_reader<R: Read>(_: &mut R) -> std::io::Result<Self> {
                todo!()
            }
        }
    };
}

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}
// conversions functions
impl U256 {
    /// Converts a u256 to u64
    pub fn to_u64(self) -> Option<u64> {
        self.try_into().map_or_else(|_| None, Some)
    }

    /// Converts a u256 to u64
    pub fn try_to_u64(self) -> SpedXSpotResult<u64> {
        self.try_into().map_err(|_| BigNumberConversionError)
    }

    /// Converts a u128 to u64
    pub fn to_u128(self) -> Option<u128> {
        self.try_to_u128().map_or_else(|_| None, Some)
    }

    /// Converts a u128 to u64
    pub fn try_to_u128(self) -> SpedXSpotResult<u128> {
        self.try_into().map_err(|_| BigNumberConversionError)
    }

    /// Converts from little endian bytes 
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        U256::from_little_endian(&bytes)
    }

    /// Converts to little endian bytes
    pub fn to_le_bytes(self) -> [u8; 32] {
        // creating a new buffer of a vector type with a least capacity of the size of U256 struct
        let mut buf: Vec<u8> = Vec::with_capacity(size_of::<Self>());
        // converting the buffer to little endian type
        self.to_little_endian(buf.borrow_mut());

        // creating a new array of 32 bytes
        let mut bytes = [0u8; 32];
        // copies all elements of the `src`, which is the buf into the bytes. We have to ensure that both are of the same length
        bytes.copy_from_slice(buf.as_slice());
        bytes
    }
}

impl_borsh_deserialize_for_bn!(U256);
impl_borsh_serialize_for_bn!(U256);

construct_uint! {
    /// 192-bit unsigned integer
    pub struct U192(3);
}

impl U192 {
    /// Convert u192 to u64
    pub fn to_u64(self) -> Option<u64> {
        self.try_to_u64().map_or_else(|_| None, Some)
    }

    /// Convert u192 to u64
    pub fn try_to_u64(self) -> SpedXSpotResult<u64> {
        self.try_into().map_err(|_| BigNumberConversionError)
    }

    /// Convert u192 to u128
    pub fn to_u128(self) -> Option<u128> {
        self.try_to_u128().map_or_else(|_| None, Some)
    } 

    /// Convert u192 to u128
    pub fn try_to_u128(self) -> SpedXSpotResult<u128> {
        self.try_into().map_err(|_| BigNumberConversionError)
    }

    /// Convert from little endian bytes
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        U192::from_little_endian(&bytes)
    }

    /// Convert to little endian bytes
    pub fn to_le_bytes(self) -> [u8;32] {
        // creating a new buffer of a vector type with a least capacity of the size of U192 struct
        let mut buf = Vec::with_capacity(size_of::<Self>());
        // converting the buffer to little endian type
        self.to_little_endian(buf.borrow_mut());

        // creating a new array of 32 bytes
        let mut bytes = [0u8; 32];
        // copies all elements of the `src`, which is the buf into the bytes. We have to ensure that both are of the same length
        bytes.copy_from_slice(buf.as_slice());
        bytes
    }
}

impl_borsh_deserialize_for_bn!(U192);
impl_borsh_serialize_for_bn!(U192);