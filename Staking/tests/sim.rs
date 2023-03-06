use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount, DEFAULT_GAS,
};
use near_units::parse_near;

extern crate fungible_token;
extern crate non_fungible_token;
extern crate cross_contract_high_level;

use fungible_token::ContractContract as FTContract;
use non_fungible_token::ContractContract as NFTContract;
use cross_contract_high_level::CrossContractContract;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    TOKEN_FT => "../fungible-token/res/fungible_token.wasm",
    TOKEN_NFT => "../non-fungible-token/res/non_fungible_token.wasm",
    TOKEN_STAKING => "res/cross_contract_high_level.wasm",
}

/// # Note
/// 
/// Tested mint, transfer, approve, transfer_from, and stake.
/// 
/// # TODO
/// 

fn init() -> (UserAccount, 
    UserAccount,
    UserAccount,
    ContractAccount<FTContract>, 
    ContractAccount<NFTContract>, 
    ContractAccount<CrossContractContract>) 
{
    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_limit = u64::MAX;
    genesis.gas_price = 0;
    let root = init_simulator(Some(genesis));

    let alice = root.create_user(
        "alice".parse().unwrap(),
        to_yocto("100") // initial balance
    );

    let bob = root.create_user(
        "bob".parse().unwrap(),
        to_yocto("100") // initial balance
    );
    let ft_account = deploy! {
        contract: FTContract,
        contract_id: "ft_contract",
        bytes: &TOKEN_FT,
        signer_account: root
    };
    let nft_account = deploy! {
        contract: NFTContract,
        contract_id: "nft_contract",
        bytes: &TOKEN_NFT,
        signer_account: root,
        init_method: new_default_meta(root.account_id())
    };
    let staking_account = deploy! {
        contract: CrossContractContract,
        contract_id: "staking_contract",
        bytes: &TOKEN_STAKING,
        signer_account: root,
        init_method: new(ft_account.account_id(), nft_account.account_id())
    };
    (root, alice, bob, ft_account, nft_account, staking_account)
}

#[test]
fn test_sim_transfer() {
    let (root, alice, bob, ft_account, nft_account, staking_account ) = init();

    let token_metadata = TokenMetadata {
        title: Some("Olympus Mons".into()),
        description: Some("The tallest mountain in the charted solar system".into()),
        media: None,
        media_hash: None,
        copies: Some(1u64),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    };

    let res = call!(
        root,
        nft_account.nft_mint("0".parse().unwrap(), alice.account_id().clone(), token_metadata),
        parse_near!("7 mN"),
        DEFAULT_GAS
    );
    println!("");
    println!(" -- 1 :: {:?}", res);
    println!("*********** mint to alice succeed. ");

    let res = call!(
        alice,
        nft_account.nft_approve("0".parse().unwrap(), staking_account.account_id().clone(), Option::<String>::None),
        parse_near!("7 mN"),
        DEFAULT_GAS
    );
    println!("");
    println!(" -- 2 :: {:?}", res);
    println!("*********** alice approved to staking contract. ");

    let res = call!(
        staking_account.user_account,
        nft_account.nft_transfer(staking_account.account_id(), "0".parse().unwrap(), Option::<u64>::None, Option::<String>::None),
        1,
        DEFAULT_GAS
    );
    println!("");
    println!(" -- 3 :: {:?}", res);

    let res = call!(
        staking_account.user_account,
        nft_account.nft_transfer_from_to(staking_account.account_id(), bob.account_id(), "0".parse().unwrap(), Option::<u64>::None, Option::<String>::None),
        1,
        DEFAULT_GAS
    );
    println!("");
    println!(" -- 4 :: {:?}", res);
    println!("*********** transfer from staking contract to bob succeed. ");

    // let res = call!(
    //     bob,
    //     nft_account.nft_approve("0".parse().unwrap(), staking_account.account_id().clone(), Option::<String>::None),
    //     parse_near!("7 mN"),
    //     DEFAULT_GAS
    // );
    // println!("");
    // println!(" -- 5 :: {:?}", res);
    // println!("*********** bob approved staking contract. ");

    // let res = call!(
    //     staking_account.user_account,
    //     nft_account.nft_transfer_from_to(bob.account_id(), staking_account.account_id(), "0".parse().unwrap(), Option::<u64>::None, Option::<String>::None),
    //     1,
    //     DEFAULT_GAS
    // );
    let res = call!(
        bob,
        staking_account.stake("0".parse().unwrap()),
        0,
        DEFAULT_GAS
    );
    println!("");
    println!(" -- 5 :: {:?}", res);
    println!("*********** staking succeed.");

}
