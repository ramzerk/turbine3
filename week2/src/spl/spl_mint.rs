use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::hash::Hash;
use solana_program_pack::Pack;
use spl_token::{instruction::initialize_mint, state::Mint};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{instruction::mint_to, state::Account as TokenAccount};

const MINT_AMOUNT: u64 = 1_000_000;

pub fn run(rpc: &RpcClient, payer: &Keypair, mint: &Pubkey) -> Result<()> {
    let ata = get_associated_token_address(&payer.pubkey(), mint);
    println!("  ata: {}", ata);

    let mut instructions = vec![];

    if rpc.get_account(&ata).is_err() {
        instructions.push(create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            mint,
            &spl_token::id(),
        ));
    }

    instructions.push(
        mint_to(
            &spl_token::id(),
            mint,
            &ata,
            &payer.pubkey(),
            &[],
            MINT_AMOUNT,
        )
        .context("failed to build mint_to")?,
    );

    let blockhash = rpc
        .get_latest_blockhash()
        .context("failed to fetch blockhash")?;

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );

    let sig = rpc
        .send_and_confirm_transaction_with_spinner(&tx)
        .context("transaction failed")?;

    let account = rpc.get_account(&ata).context("failed to fetch ata")?;
    let state = TokenAccount::unpack(&account.data).context("failed to deserialize ata")?;

    println!(
        "  balance: {} raw ({} tokens)",
        state.amount,
        state.amount as f64 / 1_000_000.0
    );
    println!("  sig    : {}", sig);

    Ok(())
}
