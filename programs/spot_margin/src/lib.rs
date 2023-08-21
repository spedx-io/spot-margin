use anchor_lang::prelude::*;
pub mod state;
pub mod macros;
pub mod error;
pub mod math;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod spot_margin {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        quote_atoms_quoted: u64,
        post_only: bool,
        reduce_only: bool,
        iceberg: bool
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub phnx_market: UncheckedAccount<'info>,

    pub program: Program<'info, System>
}
