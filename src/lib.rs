/*
MIT license
Develop by GlobalDv @2022
*/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, ext_contract, Gas, Balance, require, PanicOnDefault, promise_result_as_success};
use near_sdk::json_types::U128;
use std::collections::HashMap;

const BASE_GAS: Gas = Gas(3_000_000_000_000);
const CONSUMO_STORAGE_NEAR_SUBCONTRACT: u128 = 1484390000000000000000000;

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
trait ExtBlockBalance {
    fn on_block_balance_token(&mut self,
        ft_token: String,
        amount: U128
    ) -> u128;
}



/*
Near P2P Struct
*/
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
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

    #[payable]    
    pub fn transfer(&mut self,
        receiver_id: AccountId,
        operation_amount: U128,
        fee_deducted: U128,
        contract_ft: Option<AccountId>,
        retiro: bool,
        ft_token: String
    ) {
        require!(env::attached_deposit() >= 1, "Requires attached deposit of at least 1 yoctoNEAR");
        require!(env::predecessor_account_id() == self.user_admin, "Only administrators");
        if contract_ft.is_some() {
            ext_tranfer_ft_token::ft_transfer(
                receiver_id,
                U128(operation_amount.0 - fee_deducted.0),
                None,
                contract_ft.clone().unwrap(),
                1,
                BASE_GAS,
            );
            if fee_deducted.0 > 0 {
                ext_tranfer_ft_token::ft_transfer(
                    self.vault.clone(),
                    U128(fee_deducted.0),
                    None,
                    contract_ft.unwrap(),
                    1,
                    BASE_GAS,
                );
            }
        } else {
            Promise::new(receiver_id).transfer(operation_amount.0 - fee_deducted.0);
            if fee_deducted.0 > 0 {
                Promise::new(self.vault.clone()).transfer(fee_deducted.0);
            }
        }
        if !retiro {
            let balance_block: Balance = *self.balance_block.get(&ft_token).or(Some(&0u128)).unwrap();
            self.balance_block.insert(ft_token, balance_block - operation_amount.0);
        }    
    }

    pub fn get_balance_near(self, balance: String) -> Balance {
        let balance_general = balance_general(env::account_balance());
        let balance_bloqueado = *self.balance_block.get(&"NEAR".to_string()).or(Some(&0u128)).unwrap();
        let balance_libre = balance_general - balance_bloqueado;
        
        match balance.as_str() {
            "genral" => balance_general,
            "libre" => balance_libre,
            _=> balance_bloqueado,
        }
    }

    pub fn get_balance_block_total(self) -> Balance {
        let mut balance_bloqueado = 0;
        self.balance_block.iter().for_each(|(_k, v)| {
            balance_bloqueado += v; 
        });
        balance_bloqueado
    }

    pub fn get_balance_block_token(self, ft_token: String) -> Balance {
        *self.balance_block.get(&ft_token).or(Some(&0u128)).unwrap()
    }

    pub fn delete_contract(&mut self) {
        require!(env::predecessor_account_id() == self.user_admin, "Only administrators");
        Promise::new(AccountId::from(env::current_account_id())).delete_account(self.owner_id.clone());
    }

    pub fn block_balance_near(&mut self, amount: U128) -> bool {
        require!(env::predecessor_account_id() == self.user_admin, "Only administrators");
        let balance_general = balance_general(env::account_balance());
        let balance_block_near: Balance = *self.balance_block.get(&"near".to_string()).or(Some(&0u128)).unwrap();
        
        match (balance_general - balance_block_near) >= amount.0 {
            true => {
                self.balance_block.insert("NEAR".to_string(), balance_block_near + amount.0);
                true
            },
            _=> false,
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
    pub fn on_block_balance_token(&mut self,
        ft_token: String,
        amount: U128
    ) -> bool {
        require!(env::predecessor_account_id() == env::current_account_id(), "Only administrators");
        let result = promise_result_as_success();
        if result.is_none() {
            env::panic_str("Error bloquear balance token".as_ref());
        }
        let balance_block_token: u128 = *self.balance_block.get(&ft_token).or(Some(&0u128)).unwrap();
        let balance_general: U128 = near_sdk::serde_json::from_slice::<U128>(&result.unwrap()).expect("U128");
        match (balance_general.0 - balance_block_token) >= amount.0 {
            true => {
                self.balance_block.insert(ft_token, balance_block_token + amount.0); 
                true
            },
            _=> false,
        }
    }

}

fn balance_general(balance: u128) -> u128 {
    let balance_general: u128;
    match balance > CONSUMO_STORAGE_NEAR_SUBCONTRACT {
        true => balance_general = balance - CONSUMO_STORAGE_NEAR_SUBCONTRACT,
        _=> balance_general = 0,
    }
    balance_general
}