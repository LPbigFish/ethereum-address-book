#![cfg_attr(debug_assertions, allow(unused_imports))]
use std::collections::HashSet;
use std::env;
use std::io::ErrorKind;
use std::iter::once;
use std::sync::{Arc, Mutex};
use alloy::consensus::Transaction;
use alloy::eips::BlockNumberOrTag;
use alloy::network::primitives::BlockTransactionsKind;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder, ReqwestProvider};
use indicatif::ProgressIterator;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

#[derive(Deserialize, Serialize)]
struct AddressBook {
    block_height: u64,
    addresses: HashSet<[u8; 20]>,
}

impl AddressBook {
    fn new(block_height: u64, addresses: HashSet<[u8; 20]>) -> AddressBook {
        AddressBook { block_height, addresses }
    }

    fn increment_block_height(&mut self) -> u64 {
        self.block_height += 1;
        self.block_height.clone()
    }

    async fn recreate_from_file(file_name: &str) -> tokio::io::Result<AddressBook> {
        let mut file = File::open(file_name).await?;
        let mut buff = Vec::new();

        file.read_to_end(&mut buff).await?;

        if let Ok(res) = bincode::deserialize::<AddressBook>(buff.as_slice()) {
            Ok(res)
        } else {
            Err(ErrorKind::InvalidData.into())
        }
    }

    async fn insert_new_block(&mut self, provider: &ReqwestProvider) {
        if let Ok(Some(block)) = provider.get_block_by_number(
            BlockNumberOrTag::Number(self.increment_block_height()),
            BlockTransactionsKind::Full).await {
            let block_addresses: HashSet<[u8; 20]> = block.transactions.into_transactions().map(|transaction| {
                if let Some(addressed) = transaction.to() {
                    addressed.into_array()
                } else {
                    transaction.from.into_array()
                }
            }).collect();
            once(block.header.beneficiary.into_array()).chain(block_addresses).for_each(|address| {
                self.addresses.insert(address);
            });
        } else {
            println!("No block found in provider");
        }
    }

    async fn rewrite_to_file(&self) -> Result<(), std::io::Error> {
        let mut file = File::create("address_book.bin").await?;
        file.write_all(bincode::serialize(&self).expect("Gg, we ain't serializing").as_slice()).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;

    let mut address_book: AddressBook;

    if let Ok(_) = File::open("address_book.bin").await {
        address_book = AddressBook::recreate_from_file("address_book.bin").await?;
    } else {
        address_book = AddressBook::new(21395800, HashSet::new());
    }

    let rpc_url = Url::from(env::var("ALCHEMY_URL").expect("Alchemy URL not set").parse().unwrap());

    let provider = ProviderBuilder::new().on_http(rpc_url);

    while address_book.block_height < 21395817 {
        address_book.insert_new_block(&provider).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    address_book.rewrite_to_file().await?;

    let stringify_addresses = address_book.addresses.iter().map(|add| Address::from(add).to_string()).collect::<Vec<String>>();

    println!("Addresses: {:?}", stringify_addresses);
    println!("Unique Addresses: {}", stringify_addresses.len());
    println!("Block height: {}", address_book.block_height);
    Ok(())
}
