use ethnum::U256;
use web3::contract::{Contract, Options};
use web3::futures::{future, StreamExt};

extern crate dotenv;

#[tokio::main]
async fn main() -> web3::contract::Result<()> {
    dotenv::dotenv().ok();
    let goerli_api_key = std::env::var("API_GOERLI").expect("API_GOERLI must be set");
    let web3_goerli = web3::Web3::new(web3::transports::WebSocket::new(&goerli_api_key).await?);
    let web3_gananche_wss =
        web3::Web3::new(web3::transports::WebSocket::new("ws://localhost:8545").await?);
    let web3_gananche = web3::Web3::new(web3::transports::Http::new("http://localhost:8545")?);
    let event_signature = "0xcc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5";

    let filter_ganache = web3::types::FilterBuilder::default()
        .address(vec!["0xF523ac1d8b1aDcD0c97b3dF9C906B6E02Da562fB"
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

    let sub_ganache = web3_gananche_wss
        .eth_subscribe()
        .subscribe_logs(filter_ganache)
        .await?;

    let sub_ganache_logging = sub_ganache.for_each(|log| async move {
        let amount = format!("{:?}", log.unwrap().topics[2]);
        let amount_decoded = U256::from_str_radix(&amount[2..], 16).unwrap();
        println!("Amount: {:?}", amount_decoded);
    });

    sub_ganache_logging.await;

    /*     let filter_goerli = web3::types::FilterBuilder::default()
        .address(vec!["0xf6f562525D0801C243177b71E74d99e34AaA2a4F"
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

    let filter_goerli_deposit = web3::types::FilterBuilder::default()
        .address(vec!["0xf6f562525D0801C243177b71E74d99e34AaA2a4F"
            .parse()
            .unwrap()])
        .from_block(web3::types::BlockNumber::Latest)
        .topics(
            Some(vec![
                "0x5548c837ab068cf56a2c2479df0882a4922fd203edb7517321831d95078c5f62"
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

    let sub_goerli_deposit = web3_goerli
        .eth_subscribe()
        .subscribe_logs(filter_goerli_deposit)
        .await?;

    sub_goerli_deposit
        .for_each(|log| {
            //convert hexadecimal to decimal
            let topics2 = log.unwrap().topics[2];
            //convert topics2 to string
            let topics2_string = topics2.to_string();

            let (useless, topics2_without_prefix) = topics2_string.split_at(2);
            print!("Got 0x: {}", useless);
            print!("Got topics2: {}", topics2_without_prefix);

            //compare topic2
            let tp2 = "00000000000000000000000000000000000000000000000000071afd498d0000";
            if topics2_without_prefix == tp2 {
                println!("same");
            } else {
                println!("different");
            }
            let decimal = u128::from_str_radix(&topics2_without_prefix, 16).unwrap_or_else(|e| {
                panic!("Error parsing hex string: {}", e);
            });

            println!("Got topics2: {:?}", decimal);
            future::ready(())
        })
        .await;

    //execute storage_number function when event happens

    let storage_contract = Contract::from_json(
        web3_gananche.eth(),
        "0x9Da604E24B157aa0b581e58b5d3AD5719B86C843"
            .parse()
            .unwrap(),
        include_bytes!("storage.json"),
    )
    .unwrap();

    let ganache_accounts = web3_gananche.eth().accounts().await?;

    let account = ganache_accounts[0];

    let sub = sub_goerli
        .filter_map(|log| async {
            println!("Got log WITHDRAW transaction hash: {:?}", log);

            if let Ok(log) = log {
                let tx = storage_contract
                    .call("store", (27_u32,), account, Options::default())
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

    sub.await; */

    Ok(())
}
