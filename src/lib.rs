/*
MIT license
Develop by GlobalDv @2022
*/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, ext_contract, Gas, Balance, require, PanicOnDefault};
use near_sdk::json_types::U128;

const BASE_GAS: Gas = Gas(3_000_000_000_000);
const CONSUMO_STORAGE_NEAR_SUBCONTRACT: u128 = 1131905000000000000000000;
                    
#[ext_contract(ext_tranfer_ft_token)]
trait ExtTranfer {
    fn ft_transfer(&mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>
    );
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
        }
    }

    #[payable]    
    pub fn transfer(&mut self,
        receiver_id: AccountId,
        operation_amount: U128,
        fee_deducted: U128,
        contract_ft: Option<AccountId>,
    ) {
        require!(env::attached_deposit() >= 1, "Requires attached deposit of at least 1 yoctoNEAR");
        require!(env::predecessor_account_id() == self.user_admin, "Only administrators");
        if contract_ft.is_some() {
            ext_tranfer_ft_token::ft_transfer(
                receiver_id,
                U128(operation_amount.0),
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
            
    }

    pub fn get_balance_near(self) -> Balance {
        let balance_general = balance_general(env::account_balance());
        
        balance_general
    }

    pub fn delete_contract(&mut self) {
        require!(env::predecessor_account_id() == self.user_admin, "Only administrators");
        Promise::new(AccountId::from(env::current_account_id())).delete_account(self.owner_id.clone());
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