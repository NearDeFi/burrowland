use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::{serde_json, PromiseOrValue};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TokenReceiverMsg {
    Actions {},
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_account_id = env::predecessor_account_id();
        // self.assert_deposit_enabled(&token_account_id);

        let msg: TokenReceiverMsg =
            serde_json::from_str(&msg).expect("ERR: Parsing TokenReceiverMsg");

        let mut account = self.internal_unwrap_account(sender_id.as_ref());
        self.internal_deposit(&mut account, &token_account_id, amount.0);
        self.internal_set_account(sender_id.as_ref(), account);

        PromiseOrValue::Value(U128(0))
    }
}
