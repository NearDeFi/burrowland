use common::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::ValidAccountId;
#[allow(unused_imports)]
use near_sdk::AccountId;
use near_sdk::{assert_one_yocto, env, ext_contract, near_bindgen, Gas, Promise};

near_sdk::setup_alloc!();

const GAS_FOR_PROMISE: Gas = 10 * TGAS;

#[ext_contract(ext_price_receiver)]
pub trait ExtPriceReceiver {
    fn oracle_on_call(&mut self, sender_id: AccountId, data: PriceData, msg: String);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn oracle_call(
        &mut self,
        receiver_id: ValidAccountId,
        price_data: PriceData,
        msg: String,
    ) -> Promise {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let remaining_gas = env::prepaid_gas() - env::used_gas();
        assert!(remaining_gas >= GAS_FOR_PROMISE);

        ext_price_receiver::oracle_on_call(
            sender_id,
            price_data,
            msg,
            receiver_id.as_ref(),
            NO_DEPOSIT,
            remaining_gas - GAS_FOR_PROMISE,
        )
    }
}
