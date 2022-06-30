/*
MIT license
Develop by GlobalDv @2022
*/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, ext_contract, Gas, Balance, require, PanicOnDefault};
use near_sdk::json_types::U128;

const GAS_FOR_TRANSFER: Gas = Gas(40_000_000_000_000);

#[ext_contract(ext_tranfer_ft_token)]
trait ExtTranferUsdc {
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
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Clone)]
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

    pub fn transfer(&mut self, receiver_id: AccountId, operation_amount: u128, fee_deducted: u128, contract_ft: Option<AccountId>) {
        require!(env::predecessor_account_id() != self.user_admin, "Only administrators");
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

    pub fn get_balance_near(self) -> Balance {
        env::account_balance()
    }

    pub fn delete_contract(&mut self) {
        require!(env::predecessor_account_id() != self.user_admin, "Only administrators");
        Promise::new(AccountId::from(env::current_account_id())).delete_account(self.owner_id.clone());
    }
}