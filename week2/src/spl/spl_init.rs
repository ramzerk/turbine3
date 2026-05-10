use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::hash::Hash;
use spl_token::{instruction::initialize_mint, state::Mint};
pub const DECIMALS: u8 = 6;

pub fn run(rpc: &RpcClient, payer: &Keypair) -> Result<Keypair> {
    let balance = rpc
        .get_balance(&payer.pubkey())
        .context("failed to fetch balance")?;
    if balance < 10_000_000 {
        anyhow::bail!("insufficient balance: {} lamports", balance);
    }

    let mint = Keypair::new();
    println!("  mint: {}", mint.pubkey());

    let rent = rpc
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .context("failed to fetch rent")?;

    let blockhash = rpc
        .get_latest_blockhash()
        .context("failed to fetch blockhash")?;

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                rent,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &payer.pubkey(),
                None,
                DECIMALS,
            )
            .context("failed to build initialize_mint")?,
        ],
        Some(&payer.pubkey()),
        &[payer, &mint],
        blockhash,
    );

    let sig = rpc
        .send_and_confirm_transaction_with_spinner(&tx)
        .context("transaction failed")?;
    let account = rpc
        .get_account(&mint.pubkey())
        .context("failed to fetch mint account")?;
    let state = Mint::unpack(&account.data).context("failed to deserialize mint")?;

    println!("  decimals : {}", state.decimals);
    println!("  authority: {}", state.mint_authority.unwrap());
    println!("  sig      : {}", sig);

    Ok(mint)
}
