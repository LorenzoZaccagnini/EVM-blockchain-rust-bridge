use web3::contract::{Contract, Options};
use web3::futures::{future, StreamExt};

extern crate dotenv;

#[tokio::main]
async fn main() -> web3::contract::Result<()> {
    dotenv::dotenv().ok();
    let goerli_api_key = std::env::var("API_GOERLI").expect("API_GOERLI must be set");
    let mumbai_api_key = std::env::var("API_MUMBAI").expect("API_MUMBAI must be set");

    let web3_goerli = web3::Web3::new(web3::transports::WebSocket::new(&goerli_api_key).await?);

    let web3_mumbai = web3::Web3::new(web3::transports::WebSocket::new(&mumbai_api_key).await?);
    let web3_gananche = web3::Web3::new(web3::transports::Http::new("http://localhost:8545")?);

    //todo listen to events withdraw
    let filter_goerli = web3::types::FilterBuilder::default()
        .address(vec!["0x518637C89E2cAF08aB52e717eD55B987E3790e46"
            .parse()
            .unwrap()])
        .from_block(web3::types::BlockNumber::Latest)
        .topics(
            Some(vec![
                "0x9b1bfa7fa9ee420a16e124f794c35ac9f90472acc99140eb2f6447c714cad8eb"
                    .parse()
                    .unwrap(),
            ]),
            Some(vec![]),
            None,
            None,
        )
        .build();

    let sub_goerli = web3_goerli
        .eth_subscribe()
        .subscribe_logs(filter_goerli)
        .await?;

    //execute storage_number function when event happens

    let storage_contract = Contract::from_json(
        web3_gananche.eth(),
        "0xba114f724115E2f82Ae1CEF88A171995e49918c6"
            .parse()
            .unwrap(),
        include_bytes!("storage.json"),
    )
    .unwrap();

    let ganache_accounts = web3_gananche.eth().accounts().await?;

    let account = ganache_accounts[0];

    let sub = sub_goerli
        .filter_map(|log| async {
            println!("Got log transaction hash: {:?}", log);
            if let Ok(log) = log {
                let tx = storage_contract
                    .call("store", (42_u32,), account, Options::default())
                    .await;
                println!("TxHash: {:?}", tx);
                let number_stored: u64 = storage_contract
                    .query("number", (), None, Options::default(), None)
                    .await
                    .unwrap();
                println!("result {:?}", number_stored);
                Some(())
            } else {
                None
            }
        })
        .for_each(|_| async {});

    sub.await;

    Ok(())
}

async fn storage_number(
    web3: web3::Web3<web3::transports::Http>,
    contract_address: &str,
) -> web3::contract::Result<u64> {
    println!("loading contract");

    let storage_contract = Contract::from_json(
        web3.eth(),
        contract_address.parse().unwrap(),
        include_bytes!("storage.json"),
    )
    .unwrap();

    let accounts = web3.eth().accounts().await?;

    let tx = storage_contract
        .call("store", (42_u32,), accounts[0], Options::default())
        .await?;
    println!("TxHash: {}", tx);

    let storage_number: u64 = storage_contract
        .query("number", (), None, Options::default(), None)
        .await
        .unwrap();

    println!("storage_number: {}", storage_number);

    Ok(storage_number)
}
