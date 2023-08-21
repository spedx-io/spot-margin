use anchor_lang::prelude::*;
pub mod state;
pub mod macros;
pub mod error;
pub mod math;

declare_id!("AKiHde3YE4KWPPPYbHSP8nbKcZ37DVQUrfgfAsFXziwv");

#[program]
pub mod spot_margin {
    use super::*;

    pub fn initialize(
        _ctx: Context<Initialize>,
        _quote_atoms_quoted: u64,
        _post_only: bool,
        _reduce_only: bool,
        _iceberg: bool
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
/// CHECK: Checked in instruction creation
    pub phnx_market: UncheckedAccount<'info>,

    pub program: Program<'info, System>
}
