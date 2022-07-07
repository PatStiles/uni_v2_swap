use std::env;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use web3::contract::{Contract, Options};
use web3::types::{Address, H160, U256};

fn get_valid_timestamp(future_millis: u128) -> u128 {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let time_millis = since_epoch.as_millis().checked_add(future_millis).unwrap();
    time_millis
}

//Enable async run time via tokio
#[tokio::main]
async fn main() -> web3::Result<()> {
    //read .env file
    dotenv::dotenv().ok();
    //create websocket connection
    let websocket = web3::transports::WebSocket::new(&env::var("ALCHEMY_RINKEBY").unwrap()).await?;
    let web3s = web3::Web3::new(websocket);
    let mut accounts = web3s.eth().accounts().await?;
    accounts.push(H160::from_str(&env::var("ACCOUNT_ADDRESS").unwrap()).unwrap());
    println!("Accounts: {:?}", accounts);
    let wei_conv: U256 = U256::exp10(18);
    for account in &accounts {
        let balance = web3s.eth().balance(*account, None).await?;
        println!(
            "Eth balance of {:?}: {}",
            account,
            balance.checked_div(wei_conv).unwrap()
        );
    }

    let router02_addr = Address::from_str("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D").unwrap();
    let router02_contract = Contract::from_json(
        web3s.eth(),
        router02_addr,
        include_bytes!("router02_abi.json"),
    )
    .unwrap();
    let weth_addr: Address = router02_contract
        .query("WETH", (), None, Options::default(), None)
        .await
        .unwrap();
    println!("WETH address: {:?}", &weth_addr);

    let dai_address = Address::from_str("0x34270631F44C24fc320283347c38515798fA4388").unwrap();
    let valid_timestamp = get_valid_timestamp(300000);
    println!("timemillis: {}", valid_timestamp);
    let out_gas_estimate = router02_contract
        .estimate_gas(
            "swapExactETHForTokens",
            (
                U256::from_dec_str("1000").unwrap(),
                vec![weth_addr, dai_address],
                accounts[0],
                U256::from_dec_str(&valid_timestamp.to_string()).unwrap(),
            ),
            accounts[0],
            Options {
                value: Some(U256::exp10(18).checked_div(20.into()).unwrap()),
                gas: Some(500_000.into()),
                ..Default::default()
            },
        )
        .await
        .expect("Error");
    println!("estimated gas amount: {}", out_gas_estimate);
    let gas_price = web3s.eth().gas_price().await.unwrap();
    println!("gas price: {}", gas_price);
    Ok(())
}
