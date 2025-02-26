/*
    Daemon contract general tests
*/
mod common;
use cosmwasm_std::Addr;

use speculoos::prelude::*;

#[test]
fn general() {
    let (sender, mut contract) = common::contract::start();

    asserting!("address is not present")
        .that(&contract.address())
        .is_err();

    asserting!("upload_if_needed is ok")
        .that(&contract.upload_if_needed())
        .is_ok();

    asserting!("latest_is_uploaded is true")
        .that(&contract.latest_is_uploaded().unwrap())
        .is_true();

    let init_msg = common::contract::get_init_msg(&sender);

    let _ = contract.instantiate(&init_msg, Some(&Addr::unchecked(sender)), Some(&vec![]));

    asserting!("address is present")
        .that(&contract.address())
        .is_ok();

    asserting!("migrate_if_needed is none")
        .that(
            &contract
                .migrate_if_needed(&cw20_base::msg::MigrateMsg {})
                .unwrap(),
        )
        .is_none();

    asserting!("is_running_latest is true")
        .that(&contract.is_running_latest().unwrap())
        .is_true();

    let _ = contract.upload();

    asserting!("is_running_latest is false")
        .that(&contract.is_running_latest().unwrap())
        .is_false();

    asserting!("migrate_if_needed is some")
        .that(
            &contract
                .migrate_if_needed(&cw20_base::msg::MigrateMsg {})
                .unwrap(),
        )
        .is_some();

    asserting!("code_id is ok")
        .that(&contract.code_id())
        .is_ok();
}
