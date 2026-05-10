use anyhow::{Context, Result};
use mpl_core::instructions::CreateV1Builder;
use solana_client::rpc_client::RpcClient;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::Transaction;
use solana_pubkey::Pubkey;

pub const MPL_CORE_PROGRAM_ID: &str = "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d";
pub const PLACEHOLDER_URI: &str = "https://arweave.net/placeholder-nft-metadata";

pub fn run(rpc: &RpcClient, payer: &Keypair, name: &str) -> Result<Pubkey> {
    let balance = rpc
        .get_balance(&payer.pubkey())
        .context("failed to fetch balance")?;
    if balance < 10_000_000 {
        anyhow::bail!("insufficient balance: {} lamports", balance);
    }
    let asset = Keypair::new();
    println!("  asset: {}", asset.pubkey());

    let blockhash = rpc
        .get_latest_blockhash()
        .context("failed to fetch blockhash")?;

    let tx = Transaction::new_signed_with_payer(
        &[CreateV1Builder::new()
            .asset(asset.pubkey())
            .payer(payer.pubkey())
            .name(name.to_string())
            .uri(PLACEHOLDER_URI.to_string())
            .instruction()],
        Some(&payer.pubkey()),
        &[payer, &asset],
        blockhash,
    );

    let sig = rpc
        .send_and_confirm_transaction_with_spinner(&tx)
        .context("CreateV1 failed")?;

    let account = rpc
        .get_account(&asset.pubkey())
        .context("failed to fetch asset account")?;
    let core_program: Pubkey = MPL_CORE_PROGRAM_ID.parse().expect("valid program id");
    anyhow::ensure!(
        account.owner == core_program,
        "unexpected owner: {}",
        account.owner
    );

    println!("  name: {}", name);
    println!("  sig : {}", sig);

    Ok(asset.pubkey())
}
