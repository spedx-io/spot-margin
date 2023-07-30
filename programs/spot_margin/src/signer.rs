use bytemuck::bytes_of;

/// Function to get the seeds of the signer. We use the "spedx_signer" seed to find the value(signer's pubkey).
pub fn get_signer_seeds(nonce: &u8) -> [&[u8]; 2] {
    [b"spedx_signer".as_ref(), bytes_of(nonce)]
}