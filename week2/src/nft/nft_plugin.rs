use anyhow::{Context, Result};
use mpl_core::{
    instructions::AddPluginV1Builder,
    types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginAuthority, Royalties, RuleSet},
};
use solana_client::rpc_client::RpcClient;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::Transaction;
use solana_pubkey::Pubkey;

pub fn run(rpc: &RpcClient, payer: &Keypair, asset: &Pubkey) -> Result<()> {
    add_plugin(
        rpc,
        payer,
        asset,
        Plugin::Royalties(Royalties {
            basis_points: 500,
            creators: vec![mpl_core::types::Creator {
                address: payer.pubkey(),
                percentage: 100,
            }],
            rule_set: RuleSet::None,
        }),
        None,
        "royalties",
    )?;
    add_plugin(
        rpc,
        payer,
        asset,
        Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
        Some(PluginAuthority::Address {
            address: payer.pubkey(),
        }),
        "freeze delegate",
    )?;
    add_plugin(
        rpc,
        payer,
        asset,
        Plugin::Attributes(Attributes {
            attribute_list: vec![
                Attribute {
                    key: "level".to_string(),
                    value: "1".to_string(),
                },
                Attribute {
                    key: "rarity".to_string(),
                    value: "common".to_string(),
                },
                Attribute {
                    key: "type".to_string(),
                    value: "cat".to_string(),
                },
            ],
        }),
        None,
        "attributes",
    )?;

    Ok(())
}

fn add_plugin(
    rpc: &RpcClient,
    payer: &Keypair,
    asset: &Pubkey,
    plugin: Plugin,
    authority: Option<PluginAuthority>,
    label: &str,
) -> Result<()> {
    let mut builder = AddPluginV1Builder::new()
        .asset(*asset)
        .payer(payer.pubkey())
        .plugin(plugin);

    if let Some(auth) = authority {
        builder = builder.init_authority(auth);
    }

    let blockhash = rpc
        .get_latest_blockhash()
        .with_context(|| format!("failed to fetch blockhash for: {}", label))?;

    let tx = Transaction::new_signed_with_payer(
        &[builder.instruction()],
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );

    let sig = rpc
        .send_and_confirm_transaction_with_spinner(&tx)
        .with_context(|| format!("AddPluginV1 failed for: {}", label))?;

    println!("  {} : {}", label, sig);
    Ok(())
}
