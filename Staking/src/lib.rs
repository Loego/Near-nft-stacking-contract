use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::*;
use near_sdk::{
    env, ext_contract, json_types::U128, log, near_bindgen, AccountId, PanicOnDefault, Promise,
    PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct CrossContract {
    nft_account: AccountId,
    ft_account: AccountId,
    staked: UnorderedMap<AccountId, Vector<Stake>>,
    unstaked: UnorderedMap<AccountId, Vector<u128>>,
}

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct Stake {
    timestamp: u64,
    staked_id: TokenId,
    owner_id: AccountId,
}

pub trait From<T> {
    /// Performs the conversion.
    #[must_use]
    fn from_cross_str(_: T) -> Self;
}

impl From<&str> for near_sdk::AccountId {
    /// Converts a `&mut str` into a [`String`].
    ///
    /// The result is allocated on the heap.
    #[inline]
    fn from_cross_str(s: &str) -> near_sdk::AccountId {
        s.parse().unwrap()
    }
}

// impl Default for near_sdk::AccountId {
//     /// Creates an empty `String`.
//     #[inline]
//     fn default() -> near_sdk::AccountId {
//         near_sdk::AccountId::from_cross_str("")
//     }
// }

// One can provide a name, e.g. `ext` to use for generated methods.
#[ext_contract(nftext)]
pub trait NFTCrossContract {
    fn nft_transfer(
        &self,
        sender_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> (AccountId, Option<HashMap<AccountId, u64>>);
}

#[ext_contract(ftext)]
pub trait FTCrossContract {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[near_bindgen]
impl CrossContract {
    // Default Constructor
    #[init]
    pub fn new(ft_account: AccountId, nft_account: AccountId) -> Self {
        Self {
            ft_account,
            nft_account,
            staked: UnorderedMap::new(b"staked".to_vec()),
            unstaked: UnorderedMap::new(b"unstaked".to_vec()),
        }
    }

    // pub fn deploy_status_message(&self, account_id: AccountId, amount: U128) {
    //     Promise::new(account_id)
    //         .create_account()
    //         .transfer(amount.0)
    //         .add_full_access_key(env::signer_account_pk())
    //         .deploy_contract(
    //             include_bytes!("../../status-message/res/status_message.wasm").to_vec(),
    //         );
    // }

    #[result_serializer(borsh)]
    pub fn stake(&mut self, token_id: TokenId) /*  -> PromiseOrValue<TokenId>  */
    {
        //nftext::nft_transfer_call(&self, token_id, "Stake NFT");
        let caller = env::predecessor_account_id();
        let current_timestamp = env::block_timestamp();
        //let mut _staked = self.staked.get(&caller).unwrap().clone();
        match self.staked.get(&caller) {
            Some(mut _staked) => {
                _staked.push(&Stake {
                    timestamp: current_timestamp,
                    staked_id: token_id.clone(),
                    owner_id: caller.clone(),
                });
            }
            None => {
                let mut new_vec: Vector<Stake> = Vector::new(b"new_vec".to_vec());
                new_vec.push(&Stake {
                    timestamp: current_timestamp,
                    staked_id: token_id.clone(),
                    owner_id: caller.clone(),
                });
                self.staked.insert(&caller, &new_vec);
            }
        }
        // ------------------------------------------------------

        match self.unstaked.get(&caller) {
            Some(mut _unstaked) => {
                _unstaked.push(&0);
            }
            None => {
                let new_vec: Vector<u128> = Vector::new(b"new_vec".to_vec());
                self.unstaked.insert(&caller, &new_vec);
            }
        }
        nftext::nft_transfer(
            caller.clone(),
            env::current_account_id(),
            token_id,
            Some(1u64),
            Some(String::from("memo")),
            self.nft_account.clone(), // contract account id
            1,                        // yocto NEAR to attach
            near_sdk::Gas(20000),     // gas to attach
        );
        //nftext::nft_transfer_call(&mut self, self.nft_account, "transfer nft");
    }

    #[result_serializer(borsh)]
    pub fn unstake(&mut self) {
        let owner = env::current_account_id();
        let caller = env::predecessor_account_id();
        match self.staked.get(&caller) {
            Some(mut _staked) => {
                _staked.iter().for_each(|ele| {
                    /* nftext::nft_transfer_call(
                        owner,
                        caller,
                        ele.staked_id,
                        String::from("unstake"),
                        String::from("unstake"),
                    ); */
                    if ele.owner_id == caller {
                        nftext::nft_transfer(
                            owner.clone(),
                            caller.clone(),
                            ele.staked_id,
                            Some(1u64),
                            Some(String::from("memo")),
                            self.nft_account.clone(), // contract account id
                            0,                        // yocto NEAR to attach
                            env::prepaid_gas(),       // gas to attach
                        );
                    }
                });
            }
            None => {
                log!("You didn't stake any token at all.");
            }
        }
    }

    #[result_serializer(borsh)]
    pub fn claim(&self, token_id: TokenId) {
        let caller = env::predecessor_account_id();
        match self.staked.get(&caller) {
            Some(mut _staked) => {
                _staked.iter().for_each(|ele| {
                    /* nftext::nft_transfer_call(
                        owner,
                        caller,
                        ele.staked_id,
                        String::from("unstake"),
                        String::from("unstake"),
                    ); */
                    if ele.owner_id == caller {
                        ftext::ft_transfer(
                            env::predecessor_account_id(),
                            1_000_000_000_000_000_000u128.into(),
                            Some("claim".into()),
                            self.nft_account.clone(), // contract account id
                            1,                        // yocto NEAR to attach
                            env::prepaid_gas(),       // gas to attach
                        );
                    }
                });
            }
            None => {
                log!("You are not valid claimer.");
            }
        }
    }

    #[result_serializer(borsh)]
    pub fn get_claimable(&self, token_id: TokenId) -> u128 {
        let caller = env::predecessor_account_id();
        let current_timestamp = env::block_timestamp();
        let mut staked_timestamp = 0;
        match self.staked.get(&caller) {
            Some(mut _staked) => {
                _staked.iter().for_each(|ele| {
                    if ele.staked_id == token_id {
                        staked_timestamp = ele.timestamp;
                    }
                });
                (current_timestamp - staked_timestamp).into()
            }
            None => {
                log!("{}", "Cannot get claimable amount");
                0
            }
        }
    }

    pub fn transfer_money(&mut self, account_id: AccountId, amount: u64) {
        Promise::new(account_id).transfer(amount as u128);
    }
}
