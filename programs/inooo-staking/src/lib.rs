use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_lang::solana_program::{clock};
use crate::constants::*;

declare_id!("BL3gV368of9wpkyG4p5LkpoNK6QaxhuxxV3CYzqrW6T");

mod constants {
    use anchor_lang::prelude::Pubkey;

    pub const ADMIN_KEY: Pubkey = anchor_lang::solana_program::pubkey!("3ttYrBAp5D2sTG2gaBjg8EtrZecqBQSBuFRhsqHWPYxX"); 
    pub const COLLECTION_KEY: Pubkey = anchor_lang::solana_program::pubkey!("8cAYG1NLzsuoXHWNavd7kWABZ4pdsGVGS4ZQf9TM3HfW");
    pub const REWARD_KEY: Pubkey = anchor_lang::solana_program::pubkey!("GnBw4qZs3maF2d5ziQmGzquQFnGV33NUcEujTQ3CbzP3"); 
    pub const DECIMAL: u64 = 1000000000;

    pub const STAKING_DAYS: [u8; 2] = [10, 5];
    pub const REWARDS: [u8; 2] = [7, 5];
    pub const VAULT_SEEDS: &str = "vault";
    pub const POOL_SEEDS: &str = "pool";
    pub const POOL_DATA_SEEDS: &str = "pool data";
    pub const DAY_TIME: u32 = 60;
}

#[program]
pub mod staking_nft_reward {
    use super::*;

    use anchor_lang::AccountsClose;
    
    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        let a_vault = &mut ctx.accounts.vault;
        a_vault.staked_count = 0;
        a_vault.total_reward = 0;

        Ok(())
    }

    pub fn stake(ctx: Context<StakeContext>, method: u8) -> Result<()> {
        let a_user = &ctx.accounts.user;
        let a_statistic = &mut ctx.accounts.vault;
        let a_pool = &mut ctx.accounts.pool;
        let a_pool_data = &mut ctx.accounts.pool_data;
        let a_mint = &ctx.accounts.mint;
        let a_token_from = &ctx.accounts.token_from;
        let a_token_to = &ctx.accounts.token_to;
        let a_token_program = &ctx.accounts.token_program;

        let clock = clock::Clock::get().unwrap();

        if method > 1 {
            return Err(error!(CustomError::InvalidMethod));
        }

        let m_data = &mut ctx.accounts.metadata.try_borrow_data()?;
        let metadata = mpl_token_metadata::state::Metadata::deserialize(&mut &m_data[..])?;

        let collection_not_proper = metadata
            .data
            .creators
            .as_ref()
            .unwrap()
            .iter()
            .filter(|item|{
                    COLLECTION_KEY == item.address && item.verified
            })
            .count() == 0;
        if collection_not_proper || metadata.mint != ctx.accounts.mint.key() {
            return Err(error!(CustomError::InvalidNft));
        }

        let cpi_ctx = CpiContext::new(
            a_token_program.to_account_info(),
            token::Transfer {
                from: a_token_from.to_account_info(),
                to: a_token_to.to_account_info(),
                authority: a_user.to_account_info()
            }
        );
        token::transfer(cpi_ctx, 1)?;

        a_statistic.staked_count += 1;

        a_pool.user = a_user.to_account_info().key();
        a_pool.staked_count += 1;

        a_pool_data.user = a_user.to_account_info().key();
        a_pool_data.mint = a_mint.to_account_info().key();
        a_pool_data.start_time = clock.unix_timestamp as u32;
        a_pool_data.method = method;

        Ok(())
    }

    pub fn claim(ctx: Context<ClaimContext>) -> Result<()> {
        let a_user = &ctx.accounts.user;
        let a_vault = &mut ctx.accounts.vault;
        let a_pool = &mut ctx.accounts.pool;
        let a_pool_data = &mut ctx.accounts.pool_data;
        let a_token_from = &ctx.accounts.token_from;
        let a_token_to = &ctx.accounts.token_from;
        let a_reward_from = &ctx.accounts.reward_from;
        let a_reward_to = &ctx.accounts.reward_to;
        let a_token_program = &ctx.accounts.token_program;

        let clock = clock::Clock::get().unwrap();

        if a_pool_data.start_time + STAKING_DAYS[a_pool_data.method as usize] as u32 * DAY_TIME > clock.unix_timestamp as u32 {
            return Err(error!(CustomError::NotAllowClaiming));
        }

        let days = (clock.unix_timestamp as u32 - a_pool_data.start_time) / DAY_TIME;
        let reward_amount = REWARDS[a_pool_data.method as usize] as u64 * days as u64 * DECIMAL; 

        let (_pool, pool_bump) =
            Pubkey::find_program_address(&[
                POOL_SEEDS.as_ref(), 
                a_user.to_account_info().key.as_ref()
        ], ctx.program_id);

        let pool_seeds = &[
            POOL_SEEDS.as_ref(),
            a_user.to_account_info().key.as_ref(),    
            &[pool_bump],
        ];

        let pool_signer = &[&pool_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer (
            a_token_program.to_account_info(),
            token::Transfer {
                from: a_token_from.to_account_info(),
                to: a_token_to.to_account_info(),
                authority: a_pool.to_account_info()
            },
            pool_signer
        );
        
        token::transfer(cpi_ctx, 1)?;
        
        let (_vault, vault_bump) =
            Pubkey::find_program_address(&[
                VAULT_SEEDS.as_ref(), 
        ], ctx.program_id);

        let vault_seeds = &[
            VAULT_SEEDS.as_bytes().as_ref(),
            &[vault_bump],
        ];

        let vault_signer = &[&vault_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer (
            a_token_program.to_account_info(),
            token::Transfer {
                from: a_reward_from.to_account_info(),
                to: a_reward_to.to_account_info(),
                authority: a_vault.to_account_info()
            },
            vault_signer
        );

        token::transfer(cpi_ctx, reward_amount)?;

        a_vault.staked_count -= 1;
        a_vault.total_reward += reward_amount;

        a_pool.staked_count -= 1;
        a_pool.total_reward += reward_amount;

        a_pool_data.close(a_user.to_account_info())?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeContext<'info> {
    #[account(init, seeds = [VAULT_SEEDS.as_ref()], bump, payer = admin, space = 8 + 4 + 8)]
    pub vault: Account<'info, Vault>,
    #[account(mut, constraint = admin.key() == ADMIN_KEY)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct StakeContext<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(init_if_needed, seeds = [POOL_SEEDS.as_ref(), user.key().as_ref()], bump, payer = user, space = 8 + 32 + 4 + 8)]
    pub pool: Account<'info, Pool>,
    #[account(init_if_needed, seeds = [POOL_DATA_SEEDS.as_ref(), user.key().as_ref(), mint.key().as_ref()], bump, payer = user, space = 8 + 32 + 32 + 1 + 4)]
    pub pool_data: Account<'info, PoolData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    /// CHECK: it's not dangerous
    pub metadata: AccountInfo<'info>,
    #[account(mut, constraint = token_from.mint == mint.key() && token_from.owner == user.key() && token_from.amount == 1)]
    pub token_from: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = token_to.mint == mint.key() && token_to.owner == pool.key() && token_to.amount == 1)]
    pub token_to: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ClaimContext<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub pool_data: Account<'info, PoolData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut, constraint = token_from.mint == mint.key() && token_from.owner == pool.key() && token_from.amount == 1)]
    pub token_from: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = token_from.mint == mint.key() && token_from.owner == user.key() && token_from.amount == 1)]
    pub token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = reward_from.mint == REWARD_KEY && reward_from.owner == vault.key() && reward_from.amount == 1)]
    pub reward_from: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = reward_to.mint == REWARD_KEY && reward_to.owner == user.key())]
    pub reward_to: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>
}

#[account]
pub struct Vault {
    pub staked_count: u32,
    pub total_reward: u64
}

#[account]
pub struct Pool {
    pub user: Pubkey,
    pub staked_count: u32,
    pub total_reward: u64
}

#[account]
pub struct PoolData {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub method: u8,
    pub start_time: u32
}

#[error_code]
pub enum CustomError {
    #[msg("Invalid Method.")]
    InvalidMethod,
    #[msg("Invalid Nft.")]
    InvalidNft,
    #[msg("Not allow staking.")]
    NotAllowStaking,
    #[msg("Not allow claiming.")]
    NotAllowClaiming
}