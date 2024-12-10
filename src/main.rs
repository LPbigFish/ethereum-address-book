use std::collections::HashSet;
use std::env;
use alloy::eips::BlockNumberOrTag;
use alloy::network::primitives::BlockTransactionsKind;
use alloy::providers::{Provider, ProviderBuilder};
use tokio::sync::RwLock;
use reqwest::Url;

struct AddressBook {
    block_height: u64,
    addresses: RwLock<HashSet<[u8; 20]>>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;

    let rpc_url = env::var("ALCHEMY_URL")?;
    let provider = ProviderBuilder::new().on_http(Url::parse(rpc_url.as_str()).unwrap());

    if let Some(block) = provider.get_block_by_number(BlockNumberOrTag::Number(21374922), BlockTransactionsKind::Full)
        .await? {
        println!("Block Hash: {:?}", block.transactions.as_transactions().unwrap()[0].block_number.unwrap());
    }

    Ok(())
}
