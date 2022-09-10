use web3::futures::{future, StreamExt};

extern crate dotenv;

#[tokio::main]
async fn main() -> web3::contract::Result<()> {
    dotenv::dotenv().ok();
    let goerli_api_key = std::env::var("API_GOERLI").expect("API_GOERLI must be set");
    let mumbai_api_key = std::env::var("API_MUMBAI").expect("API_MUMBAI must be set");

    let web3_goerli = web3::Web3::new(web3::transports::WebSocket::new(&goerli_api_key).await?);

    let web3_mumbai = web3::Web3::new(web3::transports::WebSocket::new(&mumbai_api_key).await?);

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
            None,
            None,
            None,
        )
        .build();

    let sub_goerli = web3_goerli
        .eth_subscribe()
        .subscribe_logs(filter_goerli)
        .await?;

    sub_goerli
        .for_each(|log| {
            println!(
                "Got log transaction hash: {:?}",
                log.unwrap().transaction_hash
            );
            future::ready(())
        })
        .await;

    Ok(())
}
