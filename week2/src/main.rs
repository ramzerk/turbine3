mod nft;
mod spl;

use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_keypair::read_keypair_file;
use solana_signer::Signer;

fn main() -> Result<()> {
    let rpc = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    let payer = read_keypair_file("devnet-wallet.json")
        .map_err(|e| anyhow::anyhow!("could not read devnet-wallet.json: {}", e))?;

    println!("payer: {}", payer.pubkey());

    println!("\n-- step 1: init mint");
    let mint = spl::spl_init::run(&rpc, &payer)?;

    println!("\n-- step 2: mint tokens");
    spl::spl_mint::run(&rpc, &payer, &mint.pubkey())?;

    println!("\n-- step 3: attach metadata");
    spl::spl_metadata::run(&rpc, &payer, &mint.pubkey())?;

    println!("\n-- step 4: transfer tokens");
    let recipient = "FECajuKAyYCEp1woG9K42iJeKCAJjKUpxzXDx9FPpfWk".parse()?;
    spl::spl_transfer::run(&rpc, &payer, &mint.pubkey(), &recipient, 1_000)?;

    println!("\n-- step 5: mint nft");
    let asset = nft::nft_mint::run(&rpc, &payer, "My Cat NFT")?;

    println!("\n-- step 6: add plugins");
    nft::nft_plugin::run(&rpc, &payer, &asset)?;

    println!("\nmint : {}", mint.pubkey());
    println!("asset: {}", asset);

    Ok(())
}
