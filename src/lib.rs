/*
MIT license
Develop by GlobalDv @2022
*/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, ext_contract, Gas, Balance, require, PanicOnDefault, promise_result_as_success};
use near_sdk::json_types::U128;
use std::collections::HashMap;

const GAS_FOR_TRANSFER: Gas = Gas(40_000_000_000_000);
const BASE_GAS: Gas = Gas(3_000_000_000_000);

#[ext_contract(ext_tranfer_ft_token)]
trait ExtTranfer {
    fn ft_transfer(&mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>
    );
}

#[ext_contract(ext_balance_ft_token)]
trait ExtBalance {
    fn ft_balance_of(self,
        account_id: AccountId,
    ) -> u128;
}

#[ext_contract(ext_internal)]
trait ExtBalance {
    fn on_block_balance_token(self,
        ft_token: String,
        amount: U128
    ) -> u128;
}

/*
Near P2P Struct
*/
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Clone)]
pub struct NearP2P {
    pub owner_id: AccountId,
    pub user_admin: AccountId,
    pub vault: AccountId,
    pub balance_block: HashMap<String, Balance>,
}


/// Implementing Struct
#[near_bindgen]
impl NearP2P {
    #[init]
    pub fn new(owner_id: AccountId, user_admin: AccountId, vault: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self { 
            owner_id: owner_id, 
            user_admin: user_admin,
            vault: vault,
            balance_block: HashMap::new(),
        }
    }

    pub fn update_new(&mut self, owner_id: AccountId, user_admin: AccountId, vault: AccountId) {
        require!(env::predecessor_account_id() == self.user_admin, "Only administrators");
        self.owner_id = owner_id;
        self.user_admin = user_admin;
        self.vault = vault;
    }

    #[payable]    
    pub fn transfer(&mut self, receiver_id: AccountId, operation_amount: u128, fee_deducted: u128, contract_ft: Option<AccountId>) {
        require!(env::attached_deposit() >= 1, "Requires attached deposit of at least 1 yoctoNEAR");
        require!(env::predecessor_account_id() == self.user_admin, "Only administrators");
        if contract_ft.is_some() {
            // transfer ft_token to owner
            ext_tranfer_ft_token::ft_transfer(
                receiver_id,
                U128(operation_amount - fee_deducted),
                None,
                contract_ft.unwrap(),
                1,
                GAS_FOR_TRANSFER,
            );
            if fee_deducted > 0 {
                // tranfer ft_token fee al vault
                ext_tranfer_ft_token::ft_transfer(
                    self.vault.clone(),
                    U128(fee_deducted),
                    None,
                    self.vault.clone(),
                    1,
                    GAS_FOR_TRANSFER,
                );
            }
        } else {
            Promise::new(receiver_id).transfer(operation_amount - fee_deducted);
            if fee_deducted > 0 {
                Promise::new(self.vault.clone()).transfer(fee_deducted);
            }
        }    
    }

    pub fn get_balance_near(self, balance_block: bool) -> Balance {
        match balance_block {
            false => env::account_balance(),
            _=> env::account_balance() - *self.balance_block.get(&"NEAR".to_string()).or(Some(&0u128)).unwrap(),
        }
    }

    pub fn get_balance_block_token(self, ft_token: String) -> Balance {
        *self.balance_block.get(&ft_token).expect("The token does not have a locked balance")
    }

    pub fn delete_contract(&mut self) {
        require!(env::predecessor_account_id() == self.user_admin, "Only administrators");
        Promise::new(AccountId::from(env::current_account_id())).delete_account(self.owner_id.clone());
    }

    pub fn block_balance_near(&mut self, amount: U128) -> bool {
        require!(env::predecessor_account_id() == self.user_admin, "Only administrators");
        let balance_block_near: Balance = *self.balance_block.get(&"near".to_string()).or(Some(&0u128)).unwrap(); //if self.balance_block.get(&"near".to_string()).is_some() { *self.balance_block.get(&"near".to_string()).unwrap() } else { 0 };
        let balance_general: Balance = env::account_balance();
        if (balance_general - balance_block_near) >= amount.0 {
            self.balance_block.insert("NEAR".to_string(), balance_block_near + amount.0);
            true
        } else {
            false
        }   
    }

    pub fn block_balance_token(&mut self,
        contract_ft: AccountId,
        ft_token: String,
        amount: U128
    ) -> Promise {
        require!(env::predecessor_account_id() == self.user_admin, "Only administrators");
        ext_balance_ft_token::ft_balance_of(
            env::current_account_id(),
            contract_ft,
            0,
            BASE_GAS,
        ).then(ext_internal::on_block_balance_token(
            ft_token,
            amount,
            env::current_account_id(),
            0,
            BASE_GAS,
        ))
    }

    #[private]
    fn on_block_balance_token(&mut self,
        ft_token: String,
        amount: U128
    ) -> bool {
        let result = promise_result_as_success();
        if result.is_none() {
            env::panic_str("Error bloquear balance token".as_ref());
        }
        let balance_block_token: Balance = if self.balance_block.get(&ft_token).is_some() { *self.balance_block.get(&ft_token).unwrap() } else { 0 };
        let balance_general: Balance = near_sdk::serde_json::from_slice::<u128>(&result.unwrap()).expect("u128");
        if (balance_general - balance_block_token) >= amount.0 {
            self.balance_block.insert(ft_token, balance_block_token + amount.0);
            true
        } else {
            false
        }
    }

}