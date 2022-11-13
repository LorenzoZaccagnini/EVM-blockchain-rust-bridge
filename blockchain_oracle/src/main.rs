use ethnum::U256;
use web3::contract::{Contract, Options};
use web3::futures::{future, StreamExt};

#[tokio::main]
async fn main() -> web3::contract::Result<()> {
    let web3_source_chain_ws =
        web3::Web3::new(web3::transports::WebSocket::new("ws://localhost:8545").await?);
    let web3_destination_chain =
        web3::Web3::new(web3::transports::Http::new("http://localhost:7545")?);

    let event_signature = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    let web3_destination_chain_contract = Contract::from_json(
        web3_destination_chain.eth(),
        "0x71a2f3AE2ed1aC64028218407d1797e89CDFC119"
            .parse()
            .unwrap(),
        include_bytes!("GBridgeToken.json"),
    )
    .unwrap();

    let filter_source_transfer = web3::types::FilterBuilder::default()
        .address(vec!["0x71a2f3AE2ed1aC64028218407d1797e89CDFC119"
            .parse()
            .unwrap()])
        .from_block(web3::types::BlockNumber::Latest)
        .topics(
            Some(vec![event_signature.parse().unwrap()]),
            None,
            None,
            None,
        )
        .build();

    let sub_ganache = web3_source_chain_ws
        .eth_subscribe()
        .subscribe_logs(filter_source_transfer)
        .await?;

    let ganache_accounts = web3_destination_chain.eth().accounts().await?;
    let account = ganache_accounts[0];

    let sub_ganache_logging = sub_ganache.for_each(|log| async move {
        let address = format!("{:?}", log.clone().unwrap().topics[2]);
        let address_from_raw = format!("{:?}", log.clone().unwrap().topics[1]);
        let address_from_decoded = format!("0x{}", &address_from_raw[26..66]);

        match address.as_str() {
            "0x0000000000000000000000000000000000000000000000000000000000000000" => {
                println!("Burned");
                let amount_decoded =
                    U256::from_str_radix(&hex::encode(log.clone().unwrap().data.0), 16).unwrap();
                println!("Amount burned: {}", amount_decoded);

                //mint tokens on the destination chain
                mint_tokens(amount_decoded.as_u64(), &address_from_decoded).await;

                /*                 let from_address_raw = format!("{:?}", log.clone().unwrap().topics[1]);
                let from_address_decoded = format!("0x{}", &from_address_raw[26..66]); */
                println!("Burned from: {}", address_from_decoded);
            }
            _ => {
                println!("Transferred");
            }
        }
    });

    sub_ganache_logging.await;

    Ok(())
}

async fn mint_tokens(amount: u64, account_target: &str) {
    let web3_destination_chain =
        web3::Web3::new(web3::transports::Http::new("http://localhost:7545").unwrap());

    let web3_destination_chain_contract = Contract::from_json(
        web3_destination_chain.eth(),
        "0x71a2f3AE2ed1aC64028218407d1797e89CDFC119"
            .parse()
            .unwrap(),
        include_bytes!("GBridgeToken.json"),
    )
    .unwrap();

    let ganache_accounts = web3_destination_chain.eth().accounts().await.unwrap();
    let account = ganache_accounts[0];

    //convert account_target to address
    let account_target_address =
        web3::types::Address::from_slice(&hex::decode(account_target.replace("0x", "")).unwrap());

    web3_destination_chain_contract
        .call(
            "mint",
            (account_target_address, amount),
            account,
            Options::default(),
        )
        .await
        .unwrap();
}
