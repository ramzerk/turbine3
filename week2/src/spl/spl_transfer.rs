use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account;

use super::spl_init::DECIMALS;

pub fn run(
    rpc: &RpcClient,
    payer: &Keypair,
    mint: &Pubkey,
    recipient: &Pubkey,
    amount: u64,
) -> Result<()> {
    let from_ata = get_associated_token_address(&payer.pubkey(), mint);
    let to_ata = get_associated_token_address(recipient, mint);

    println!("  from: {}", from_ata);
    println!("  to  : {}", to_ata);

    let from_account = rpc.get_account(&from_ata).context("sender ata not found")?;
    let from_state =
        TokenAccount::unpack(&from_account.data).context("failed to deserialize sender ata")?;

    anyhow::ensure!(
        from_state.amount >= amount,
        "insufficient balance: have {}, need {}",
        from_state.amount,
        amount
    );

    let mut instructions = vec![];

    if rpc.get_account(&to_ata).is_err() {
        instructions.push(create_associated_token_account(
            &payer.pubkey(),
            recipient,
            mint,
            &spl_token::id(),
        ));
    }

    instructions.push(
        transfer_checked(
            &spl_token::id(),
            &from_ata,
            mint,
            &to_ata,
            &payer.pubkey(),
            &[],
            amount,
            DECIMALS,
        )
        .context("failed to build transfer_checked")?,
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

    let from_after = rpc
        .get_account(&from_ata)
        .ok()
        .and_then(|a| TokenAccount::unpack(&a.data).ok());
    let to_after = rpc
        .get_account(&to_ata)
        .ok()
        .and_then(|a| TokenAccount::unpack(&a.data).ok());

    if let Some(f) = from_after {
        println!("  sender balance   : {}", f.amount);
    }
    if let Some(t) = to_after {
        println!("  recipient balance: {}", t.amount);
    }
    println!("  sig: {}", sig);

    Ok(())
}
