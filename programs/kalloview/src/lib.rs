use anchor_lang::prelude::*;
use solana_program::program::invoke_signed;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Review {
    pub product_name: String,
    pub category: String,
    pub rating: u8,
    pub comment: String,
}

#[program]
pub mod kalloview {
    use super::*;

    #[state]
    #[account]
    #[derive(AccountSerialize, AccountDeserialize, Clone)]
    pub struct KalloViewState {
        pub reviews: Vec<Review>,
    }

    #[account]
    #[derive(AccountSerialize, AccountDeserialize, Clone)]
    pub struct ReviewAccount {
        pub product_name: String,
        pub category: String,
        pub rating: u8,
        pub comment: String,
        pub user: Pubkey,
    }

    #[derive(Accounts)]
    pub struct StoreReview<'info> {
        #[account(init, payer = user, space = 8 + 32 + 32 + 1 + 128, seeds = [b"review".as_ref(), user.key().as_ref()], bump)]
        pub review_account: Account<'info, ReviewAccount>,
        #[account(mut, signer)]
        pub user: AccountInfo<'info>,
        pub system_program: AccountInfo<'info>,
        pub rent: Sysvar<'info, Rent>,
    }

    #[derive(Accounts)]
    pub struct GetReviews<'info> {
        #[account(has_one = authority)]
        pub authority: AccountInfo<'info>,
        pub reviews: QueryVec<'info, ReviewAccount>,
    }

    pub fn store_review(ctx: Context<StoreReview>, product_name: String, category: String, rating: u8, comment: String) -> ProgramResult {
        let rent = Rent::get()?;
        let rent_amount = rent.minimum_balance(ctx.accounts.review_account.data_len() as usize);

        if ctx.accounts.user.lamports() < rent_amount {
            return Err(NotEnoughFunds.into());
        }

        **ctx.accounts.user.lamports.borrow_mut() -= rent_amount;
        **ctx.accounts.program.lamports.borrow_mut() += rent_amount;

        let mut review_account = ReviewAccount {
            product_name,
            category,
            rating,
            comment,
            user: *ctx.accounts.user.key,
        };

        ctx.accounts.review_account.product_name = review_account.product_name.clone();
        ctx.accounts.review_account.category = review_account.category.clone();
        ctx.accounts.review_account.rating = review_account.rating;
        ctx.accounts.review_account.comment = review_account.comment.clone();
        ctx.accounts.review_account.user = review_account.user;

        let mut state = ctx.accounts.state.load_mut()?;
        state.reviews.push(review_account);

        // Check if the user has a Kallo token account; if not, create one
        let user_token_account = ctx.accounts.kallo_token_account.clone();
        if user_token_account.is_empty() {
            let create_token_account_instruction = create_token_account(
                &ctx.accounts.token_program.key,
                &ctx.accounts.kallo_token_mint.key,
                &ctx.accounts.user.key,
                &ctx.accounts.user.key,
                &ctx.accounts.program.key,
                &ctx.accounts.system_program.key,
                &ctx.accounts.rent.key,
                spl_token::native_mint::id(),
                0,
            )?;
            invoke_signed(
                &create_token_account_instruction,
                &[ctx.accounts.user.clone()],
                &[ctx.accounts.program.clone()],
            )?;
        }

        // Token transfer logic
        let token_program = ctx.accounts.token_program.clone();
        let amount: u64 = 10; // Number of Kallo tokens to transfer
        let transfer_instruction = transfer(
            &token_program.key,
            &ctx.accounts.kallo_token_account.key,
            &ctx.accounts.user.key,
            &ctx.accounts.program.key,
            &[],
            amount,
        )?;
        invoke_signed(
            &transfer_instruction,
            &[ctx.accounts.user.clone()],
            &[ctx.accounts.program.clone()],
        )?;

        Ok(())
    }

    pub fn get_reviews(ctx: Context<GetReviews>) -> ProgramResult {
        for review in &ctx.accounts.reviews {
            // Do something with each review, e.g., return them to the caller
        }
        Ok(())
    }

    // Define your error codes using Anchor's #[error] macro
    #[error]
    pub enum ErrorCode {
        #[msg("Not enough funds to cover rent")]
        NotEnoughFunds,
    }

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.reviews = vec![]; // Initialize the reviews array
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 0, seeds = [b"state".as_ref()], bump)]
    pub state: Account<'info, KalloViewState>,
    pub user: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
}
