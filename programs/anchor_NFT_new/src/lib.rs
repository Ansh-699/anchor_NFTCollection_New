use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::{
    instructions::{CreateV1CpiBuilder, VerifyCollectionV1CpiBuilder},
    program::Metadata,
    types::{Collection, CollectionDetails, DataV2, TokenStandard},
};

declare_id!("HoL8Zuf4x1yNsXcZCZiznQ46j1oxfAMjX2a5iVKTS9j");

#[program]
pub mod anchor_nft_new_v1 {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        name: String,
        uri: String,
        symbol: String,
    ) -> Result<()> {
        let data_v2 = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        CreateV1CpiBuilder::new(&ctx.accounts.token_metadata_program.to_account_info())
            .metadata(&ctx.accounts.metadata.to_account_info())
            .master_edition(Some(&ctx.accounts.master_edition.to_account_info()))
            .mint(&ctx.accounts.mint.to_account_info(), true)
            .authority(&ctx.accounts.payer.to_account_info())
            .payer(&ctx.accounts.payer.to_account_info())
            .update_authority(&ctx.accounts.payer.to_account_info(), true)
            .system_program(&ctx.accounts.system_program.to_account_info())
            .data_v2(data_v2)
            .collection_details(Some(CollectionDetails::V1 { size: 0 }))
            .token_standard(TokenStandard::NonFungible)
            .invoke()?;

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.associated_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        );
        token::mint_to(cpi_ctx, 1)?;

        let state = &mut ctx.accounts.collection_pda;
        state.collection_mint = ctx.accounts.mint.key();
        state.collection_metadata = ctx.accounts.metadata.key();
        state.collection_master_edition = ctx.accounts.master_edition.key();
        state.authority = ctx.accounts.payer.key();
        state.bump = *ctx.bumps.get("collection_pda").unwrap();

        Ok(())
    }

    pub fn create_nft(
        ctx: Context<CreateNft>,
        name: String,
        uri: String,
        symbol: String,
    ) -> Result<()> {
        let data_v2 = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: Some(Collection {
                verified: false,
                key: ctx.accounts.collection_mint.key(),
            }),
            uses: None,
        };

        CreateV1CpiBuilder::new(&ctx.accounts.token_metadata_program.to_account_info())
            .metadata(&ctx.accounts.nft_metadata.to_account_info())
            .master_edition(Some(&ctx.accounts.nft_master_edition.to_account_info()))
            .mint(&ctx.accounts.nft_mint.to_account_info(), true)
            .authority(&ctx.accounts.payer.to_account_info())
            .payer(&ctx.accounts.payer.to_account_info())
            .update_authority(&ctx.accounts.payer.to_account_info(), true)
            .system_program(&ctx.accounts.system_program.to_account_info())
            .data_v2(data_v2)
            .token_standard(TokenStandard::NonFungible)
            .invoke()?;

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_associated_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        );
        token::mint_to(cpi_ctx, 1)?;

        VerifyCollectionV1CpiBuilder::new(&ctx.accounts.token_metadata_program.to_account_info())
            .metadata(&ctx.accounts.nft_metadata.to_account_info())
            .collection_metadata(&ctx.accounts.collection_metadata.to_account_info())
            .collection_mint(&ctx.accounts.collection_mint.to_account_info())
            .collection_master_edition(&ctx.accounts.collection_master_edition.to_account_info())
            .collection_authority(&ctx.accounts.authority.to_account_info())
            .payer(&ctx.accounts.payer.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .invoke()?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + CollectionState::INIT_SPACE,
        seeds = [b"collection_state"],
        bump
    )]
    pub collection_pda: Account<'info, CollectionState>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer,
        mint::freeze_authority = payer,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub associated_token_account: Account<'info, TokenAccount>,

    /// CHECK: This is safe because we are creating it in the same instruction.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: This is safe because we are creating it in the same instruction.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [b"collection_state"],
        bump = collection_state.bump,
        has_one = authority
    )]
    pub collection_state: Account<'info, CollectionState>,

    pub authority: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = payer,
        mint::freeze_authority = payer,
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = payer,
    )]
    pub nft_associated_token_account: Account<'info, TokenAccount>,

    /// CHECK: This is safe because we are creating it in the same instruction.
    #[account(mut)]
    pub nft_metadata: UncheckedAccount<'info>,

    /// CHECK: This is safe because we are creating it in the same instruction.
    #[account(mut)]
    pub nft_master_edition: UncheckedAccount<'info>,

    /// CHECK: Address is checked via constraint.
    #[account(mut, address = collection_state.collection_metadata)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: Address is checked via constraint.
    #[account(address = collection_state.collection_mint)]
    pub collection_mint: UncheckedAccount<'info>,

    /// CHECK: Address is checked via constraint.
    #[account(address = collection_state.collection_master_edition)]
    pub collection_master_edition: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
#[derive(InitSpace)]
pub struct CollectionState {
    pub authority: Pubkey,
    pub collection_mint: Pubkey,
    pub collection_metadata: Pubkey,
    pub collection_master_edition: Pubkey,
    pub bump: u8,
}
