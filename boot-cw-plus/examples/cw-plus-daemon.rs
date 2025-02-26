use boot_core::*;
use boot_core::{networks::LOCAL_JUNO, DaemonOptionsBuilder};
use boot_cw_plus::{Cw20ExecuteMsgFns, Cw20QueryMsgFns, CwPlus};
use cw20::Cw20Coin;
use std::sync::Arc;
use tokio::runtime::Runtime;

// shows how to deploy CwPlus to a real environment
// Requires a running local junod with grpc enabled
// run `cargo run --features daemon --example cw-plus-daemon`
pub fn script() -> anyhow::Result<()> {
    // create the tokio runtime
    let rt = Arc::new(Runtime::new().unwrap());
    // use the cosmos chain registry for gRPC url sources.
    let _chain_data = rt.block_on(RegistryChainData::fetch("juno".into(), None))?;
    let options = DaemonOptionsBuilder::default()
        // or provide `chain_data`
        .network(LOCAL_JUNO)
        // specify a custom deployment ID
        .deployment_id("v0.1.0")
        .build()?;

    // get sender form .env file mnemonic
    let (_sender, chain) = instantiate_daemon_env(&rt, options)?;
    // identical code to the mock example
    cw20_example(chain)
}

fn cw20_example<Chain: CwEnv>(chain: Chain) -> anyhow::Result<()> {
    let sender = chain.sender();
    // Upload the cw-plus contracts
    let cw_plus = CwPlus::store_on(chain.clone())?;
    // get the cw20_base contract
    let cw20_base = cw_plus.cw20_base;
    // instantiate an instance of it
    let cw20_init_msg = cw20_base::msg::InstantiateMsg {
        decimals: 6,
        name: "Test Token".to_string(),
        initial_balances: vec![Cw20Coin {
            address: sender.to_string(),
            amount: 1000000u128.into(),
        }],
        marketing: None,
        mint: None,
        symbol: "TEST".to_string(),
    };
    cw20_base.instantiate(&cw20_init_msg, None, None)?;

    // send some tokens
    let cw20_send_msg = cw20_base::msg::ExecuteMsg::Transfer {
        recipient: "juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y".to_string(),
        amount: 100u128.into(),
    };
    cw20_base.execute(&cw20_send_msg, None)?;

    // query the balance of the recipient
    let query_msg = cw20_base::msg::QueryMsg::Balance {
        address: "juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y".to_string(),
    };
    let _balance: cw20::BalanceResponse = cw20_base.query(&query_msg)?;

    // query balance after init
    // notice that this query is generated by the macro and not defined in the object itself!
    let balance = cw20_base.balance(sender.to_string())?;
    assert_eq!(balance.balance.u128(), 999900u128.into());

    // Send with the macro-generated function
    let transfer_resp = cw20_base.transfer(100u128.into(), "recipient_addr".to_string())?;
    assert_eq!(
        // index the response
        transfer_resp.event_attr_value("wasm", "amount")?,
        100.to_string()
    );

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = script() {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));
        ::std::process::exit(1);
    }
}
