use bytemuck::bytes_of;

/// Function to get the seeds of the signer. We use the "signer" seed.
pub fn get_signer_seeds(nonce: &u8) -> [&[u8]; 2] {
    [b"signer".as_ref(), bytes_of(nonce)]
}