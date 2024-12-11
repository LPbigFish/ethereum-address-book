use std::collections::HashSet;
use std::env;
use std::iter::once;
use std::sync::{Arc, Mutex};
use alloy::consensus::Transaction;
use alloy::eips::BlockNumberOrTag;
use alloy::network::primitives::BlockTransactionsKind;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use reqwest::Url;

struct AddressBook {
    block_height: u64,
    addresses: HashSet<[u8; 20]>
}

impl AddressBook {
    fn new(block_height: u64, addresses: HashSet<[u8; 20]>) -> AddressBook {
        AddressBook { block_height, addresses }
    }

    fn increment_block_height(&mut self) -> u64 {
        self.block_height += 1;
        self.block_height.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;

    let address_book = Arc::new(Mutex::new(AddressBook::new(21381144, HashSet::new())));

    let rpc_url = Arc::new(env::var("ALCHEMY_URL").expect("Alchemy URL not set").to_string());

    let provider = ProviderBuilder::new().on_http(Url::parse(rpc_url.as_str()).expect("Http URL not set"));

    let binding = Arc::clone(&address_book);
    let mut value = binding.lock().unwrap();

    if let Ok(Some(block)) = provider.get_block_by_number(BlockNumberOrTag::Number((*value).increment_block_height()), BlockTransactionsKind::Full).await {
        let block_addresses: HashSet<[u8;20]> = block.transactions.into_transactions().map(|transaction| {
            if let Some(addressed) = transaction.to() {
                addressed.into_array()
            } else {
                transaction.from.into_array()
            }
        }).collect();
        once(block.header.beneficiary.into_array()).chain(block_addresses).for_each(| address | {
            (*value).addresses.insert(address);
        });
    } else {
        println!("No block found in provider");
    }
    let stringify_addresses = (*value).addresses.iter().map(|add| Address::from(add).to_string()).collect::<Vec<String>>();
    println!("{:?}", stringify_addresses);

    Ok(())
}
