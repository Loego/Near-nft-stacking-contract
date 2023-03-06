use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount, DEFAULT_GAS,
    STORAGE_AMOUNT,
};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::TokenId;
use near_contract_standards::non_fungible_token::Token;
use near_primitives::views::FinalExecutionStatus;
use near_primitives::transaction::{
    Action, AddKeyAction, CreateAccountAction, DeleteAccountAction, DeployContractAction,
    FunctionCallAction, SignedTransaction, TransferAction,
};
use near_units::parse_near;
use near_sdk::json_types::U128;
use near_sdk::ONE_YOCTO;
use workspaces::prelude::DevAccountDeployer;
use workspaces::{Account, Contract, DevNetwork, Worker};
extern crate cross_contract_high_level;
// Note: the struct xxxxxxContract is created by #[near_bindgen] from near-sdk in combination with
// near-sdk-sim
use cross_contract_high_level::CrossContractContract;

pub const TOKEN_ID: &str = "0";

/// # Note
/// 
/// Workspace-rs has a drawback, which a user cannot pass caller account explicitly.
/// This feature is important in NFTxxx.transfer_from() and NFTxxx.approve().
/// 
/// # TODO
/// 
/// Make test code using near_sdk_sim.


pub async fn init(
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<(Contract, Contract, Contract, UserAccount, Account, Account)> {
    println!("***************************************************** 1");
    let nft_contract =
        worker.dev_deploy(include_bytes!("../../non-fungible-token/res/non_fungible_token.wasm").to_vec()).await?;

    let ft_contract =
        worker.dev_deploy(include_bytes!("../../fungible-token/res/fungible_token.wasm").to_vec()).await?;
    println!("nft, ft contract deployed.");

    println!("***************************************************** 2");
    let staking_contract = worker.dev_deploy(include_bytes!("../res/cross_contract_high_level.wasm").to_vec()).await?;
    println!("staking contract deployed.");

    let res = nft_contract
        .call(&worker, "new_default_meta")
        .args_json((nft_contract.id(),))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));
    let res = nft_contract
        .as_account()
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("9 N"))
        .transact()
        .await?;
    println!("***************************************************** 4");
    assert!(matches!(res.details.status, FinalExecutionStatus::SuccessValue(_)));
    let alice = res.result;
    println!("subaccount alice created.");

    let res = nft_contract
        .as_account()
        .create_subaccount(&worker, "bob")
        .initial_balance(parse_near!("9 N"))
        .transact()
        .await?;
    assert!(matches!(res.details.status, FinalExecutionStatus::SuccessValue(_)));
    let bob = res.result;
    println!("subaccount bob created.");

    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_limit = u64::MAX;
    genesis.gas_price = 0;
    let master_account = init_simulator(Some(genesis));
    println!("***************************************************** 5");
    let res = staking_contract
        .call(&worker, "new")
        .args_json((ft_contract.id(), nft_contract.id()))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));
    println!("Staking contract initialized.\nres: {:#?}", res);
    return Ok((staking_contract, nft_contract, ft_contract, master_account, alice, bob));
}

#[tokio::test]
async fn test_nft() -> anyhow::Result<()>  {
    let worker = workspaces::sandbox();
    let initial_balance = U128::from(parse_near!("9 N"));
    let (staking_contract, nft_contract, ft_contract, master_account, alice, bob) = init(&worker).await?;
    
    println!("***************************************************** 6");
    let owner_tokens: Vec<Token> = nft_contract
        .call(&worker, "nft_tokens_for_owner")
        .args_json((alice.id(), Option::<U128>::None, Option::<u64>::None))?
        .view()
        .await?
        .json()?;
    assert_eq!(owner_tokens.len(), 0);
    println!("alice has 0 token.");
    
    println!("***************************************************** 7");
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

    let res = nft_contract
        .call(&worker, "nft_mint")
        .args_json((TOKEN_ID, nft_contract.id(), token_metadata))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("7 mN"))
        .transact()
        .await?;
    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));
    println!("nft minted to nft contract.");

    println!("***************************************************** 8");
    let owner_tokens: Vec<Token> = nft_contract
        .call(&worker, "nft_tokens_for_owner")
        .args_json((nft_contract.id(), Option::<U128>::None, Option::<u64>::None))?
        .view()
        .await?
        .json()?;
    assert_eq!(owner_tokens.len(), 1);
    println!("nft contract has 1 token");
    assert_eq!(owner_tokens.get(0).unwrap().token_id, "0".to_string());


    println!("***************************************************** 9");
    let res = ft_contract
        .call(&worker, "new_default_meta")
        .args_json((alice.id(), initial_balance))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));
    

    println!("***************************************************** 10");
    let res = ft_contract.call(&worker, "ft_total_supply").view().await?;
    assert_eq!(res.json::<U128>()?, initial_balance);

    let root_balance = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json((nft_contract.id(),))?
        .view()
        .await?
        .json::<U128>()?;

    assert_eq!(root_balance, U128::from(parse_near!("0 N")));
    
    println!("***************************************************** 11");
    let res = nft_contract
        .call(&worker, "nft_approve")
        .args_json((TOKEN_ID, staking_contract.id(), Option::<String>::None))?
        .gas(300_000_000_000_000)
        .deposit(510000000000000000000)
        .transact()
        .await?;
    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));
    println!("nft contract approved staking contract.");

    let res = nft_contract
        .call(&worker, "nft_transfer")
        .args_json((
            alice.id(),
            TOKEN_ID,
            Option::<u64>::None,
            Some("simple transfer".to_string()),
        ))?
        .gas(300_000_000_000_000)
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));
    println!("nft contract transfered his token to alice.");

    println!("***************************************************** 12");
    let owner_tokens: Vec<Token> = nft_contract
        .call(&worker, "nft_tokens_for_owner")
        .args_json((alice.id(), Option::<U128>::None, Option::<u64>::None))?
        .view()
        .await?
        .json()?;
    assert_eq!(owner_tokens.len(), 1);
    println!("alice has 1 token.");
    println!("res: {:#?}", owner_tokens);

    println!("***************************************************** 13");
    let nft_contract_account_id: workspaces::AccountId = nft_contract
        .as_account()
        .id()
        .to_string()
        .parse()?;

    let res = alice
        .call(&worker, nft_contract_account_id, "nft_approve")
        .args_json((TOKEN_ID, bob.id(), Option::<String>::None))?
        .gas(300_000_000_000_000)
        .deposit(510000000000000000000)
        .transact()
        .await?;
    println!("alice approved bob. \nres: {:#?}", res);

    
    println!("***************************************************** 14");
    let nft_contract_account_id: workspaces::AccountId = nft_contract
        .as_account()
        .id()
        .to_string()
        .parse()?;

    let res = alice
        .call(&worker, nft_contract_account_id, "nft_transfer")
        .args_json((
            bob.id(), 
            TOKEN_ID, 
            Option::<u64>::None,
            Some("simple transfer".to_string()),
        ))?
        .gas(300_000_000_000_000)
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    println!("alice transfered to bob. \nres: {:#?}", res);

    println!("***************************************************** 15");
    let owner_tokens: Vec<Token> = nft_contract
        .call(&worker, "nft_tokens_for_owner")
        .args_json((bob.id(), Option::<U128>::None, Option::<u64>::None))?
        .view()
        .await?
        .json()?;
    println!("bob has one token.");
    assert_eq!(owner_tokens.len(), 1);


    println!("***************************************************** 16");
    let nft_contract_account_id: workspaces::AccountId = nft_contract
        .as_account()
        .id()
        .to_string()
        .parse()?;

    let staking_contract_account_id: workspaces::AccountId = staking_contract
        .as_account()
        .id()
        .to_string()
        .parse()?;

    let res = bob
        .call(&worker, nft_contract_account_id, "nft_approve")
        .args_json((TOKEN_ID, staking_contract_account_id.clone(), Option::<String>::None))?
        .gas(300_000_000_000_000)
        .deposit(510000000000000000000)
        .transact()
        .await?;
    println!("bob approved staking contract.");
    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));


    println!("***************************************************** 17");
    let nft_contract_account_id: workspaces::AccountId = nft_contract
        .as_account()
        .id()
        .to_string()
        .parse()?;
    
    let res = bob
        .call(&worker, staking_contract_account_id.clone(), "stake")
        .args_json((TOKEN_ID,))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    // let res = bob
    //     .call(&worker, staking_contract_account_id.clone(), "test_transfer")
    //     .args_json((bob.id(), staking_contract_account_id.clone(), TOKEN_ID, Option::<u64>::None, Option::<String>::None))?
    //     .gas(300_000_000_000_000)
    //     .transact()
    //     .await?;

    println!("staking result: {:#?}", res);

    println!("***************************************************** 18");
    let owner_tokens: Vec<Token> = nft_contract
        .call(&worker, "nft_tokens_for_owner")
        .args_json((staking_contract_account_id, Option::<U128>::None, Option::<u64>::None))?
        .view()
        .await?
        .json()?;
    println!("staking contract has one token.");
    // println!("{:#?}", owner_tokens);
    assert_eq!(owner_tokens.len(), 1);

    Ok(()) 
}
