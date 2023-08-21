use anchor_lang::prelude::*;
use bytes::BytesMut;
use crate::state::oracle::Price;
use bytemuck::bytes_of;

pub fn create_account_info<'a>(
    key: &'a Pubkey,
    is_writable: bool,
    lamports: &'a mut u64,
    bytes: &'a mut [u8],
    owner: &'a Pubkey,
) -> AccountInfo<'a> {
    AccountInfo::new(key, false, is_writable, lamports, bytes, owner, false, 0)
}

pub fn get_account_bytes<T: bytemuck::Pod>(account: &mut T) -> BytesMut {
    let mut bytes = BytesMut::new();
    let data = bytemuck::bytes_of_mut(account);
    bytes.extend_from_slice(data);
    bytes
}

pub fn get_test_pyth_price(price: i64, exponent: i32) -> Price {
    let mut pyth_price = Price::default();
    pyth_price.aggregator.price = price;
    pyth_price.twap = price;
    pyth_price.exponent = exponent;
    pyth_price
}

/// Function to get the seeds of the signer. We use the "signer" seed.
pub fn get_signer_seeds(nonce: &u8) -> [&[u8]; 2] {
    [b"signer".as_ref(), bytes_of(nonce)]
}