use anchor_lang::prelude::*;

use crate::{
    error::SpedXSpotResult,
    math::{
        constants::{
            FEE_DENOMINATOR,
            FEE_PERCENTAGE_DENOMINATOR,
            PERCENTAGE_PRECISION_U64
        },
        
    },
    state::traits::Size,
};

#[account]
#[derive(Default)]
