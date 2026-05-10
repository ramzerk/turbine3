use anyhow::{Context, Result};
use borsh::BorshSerialize;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::hash::Hash;
use spl_token::{instruction::initialize_mint, state::Mint};

const MPL_TOKEN_METADATA_ID: &str = "";
const TOKEN_NAME: &str = "baby_token";
const TOKEN_SYMBOL: &str = "BBY";
const TOKEN_URI: &str = "";

const CREATE_METADATA_V3: u8 = 33;

#[derive(BorshSerialize)]
struct DataV2 {
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
    creators: Option<Vec<Creator>>,
    collection: Option<Collection>,
    uses: Option<Uses>,
}

#[derive(BorshSerialize)]
struct Creator {
    address: Pubkey,
    verified: bool,
    share: u8,
}

#[derive(BorshSerialize)]
struct Collection {
    verified: bool,
    key: Pubkey,
}

#[derive(BorshSerialize)]
struct Uses {
    use_method: u8,
    remaining: u64,
    total: u64,
}

#[derive(BorshSerialize)]
struct CreateMetadataV3Args {
    instruction: u8,
    data: DataV2,
    is_mutable: bool,
    collection_details: Option<u8>,
}

pub fn run(rpc: &RpcClient, payer: &Keypair, mint: &Pubkey) -> Result<()> {
    let mpl_program: Pubkey = MPL_TOKEN_METADATA_ID.parse().expect("valid program id");

    let seeds = &[b"metadata".as_ref(), mpl_program.as_ref(), mint.as_ref()];
    let (metadata_pda, _) = Pubkey::find_program_address(seeds, &mpl_program);
    println!("  metadata pda: {}", metadata_pda);

    if rpc.get_account(&metadata_pda).is_ok() {
        println!("  already exists, skipping");
        return Ok(());
    }

    let mut ix_data = Vec::new();
    CreateMetadataV3Args {
        instruction: CREATE_METADATA_V3,
        data: DataV2 {
            name: TOKEN_NAME.to_string(),
            symbol: TOKEN_SYMBOL.to_string(),
            uri: TOKEN_URI.to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        is_mutable: true,
        collection_details: None,
    }
    .serialize(&mut ix_data)
    .context("failed to serialize args")?;
    let ix = Instruction {
        program_id: mpl_program,
        accounts: vec![
            AccountMeta::new(metadata_pda, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(payer.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        ],
        data: ix_data,
    };

    let blockhash = rpc
        .get_latest_blockhash()
        .context("failed to fetch blockhash")?;

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[payer], blockhash);

    let sig = rpc
        .send_and_confirm_transaction_with_spinner(&tx)
        .context("transaction failed")?;

    let account = rpc
        .get_account(&metadata_pda)
        .context("failed to fetch metadata account")?;
    anyhow::ensure!(
        account.owner == mpl_program,
        "unexpected owner: {}",
        account.owner
    );

    println!("  name  : {}", TOKEN_NAME);
    println!("  symbol: {}", TOKEN_SYMBOL);
    println!("  sig   : {}", sig);

    Ok(())
}
