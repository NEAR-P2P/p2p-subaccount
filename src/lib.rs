/*
MIT license
Develop by GlobalDv @2022
*/


use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use serde::Serialize;
use serde::Deserialize;
use near_sdk::{env, near_bindgen, AccountId, Promise, ext_contract, Gas, Balance};
use near_sdk::json_types::U128;


const GAS_FOR_TRANSFER: Gas = Gas(40_000_000_000_000);
const CONTRACT_USDC: &str = "usdc.fakes.testnet";
/////////////////////////////////////////////////////////////////////////////////////////////////
/// Objects Definition///////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////


#[ext_contract(ext_tranfer_usdc)]
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
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NearP2P {
    // Users
    pub user_admin: AccountId,
    pub vault: AccountId,
}

/// Initializing deafult impl
/// We are using default inizialization for the structs
impl Default for NearP2P {
    fn default() -> Self {
        Self {
                user_admin: "contract.p2p-testnet.testnet".parse().unwrap(),
                vault: "info.testnet".parse().unwrap(),
            }
    }
}

/// Implementing Struct
#[near_bindgen]
impl NearP2P {

    pub fn transfer(&mut self, ft_token: String, receiver_id: AccountId, operation_amount: u128, fee_deducted: u128) {
        if env::signer_account_id() == self.user_admin {
            if ft_token == "USDC".to_string() {
                let contract_usdc: AccountId = CONTRACT_USDC.parse().unwrap();
                // transfer usdc to owner
                ext_tranfer_usdc::ft_transfer(
                    receiver_id,
                    U128(operation_amount - fee_deducted),
                    None,
                    contract_usdc.clone(),
                    1,
                    GAS_FOR_TRANSFER,
                );
                if fee_deducted > 0 {
                    // tranfer usdc fee al vault
                    ext_tranfer_usdc::ft_transfer(
                        self.vault.clone(),
                        U128(fee_deducted),
                        None,
                        contract_usdc.clone(),
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
        } else { env::panic_str(&"Only administrators".to_string()); }
    }

    pub fn get_balance_near(self) -> Balance {
        env::account_balance()
    }

}