use crate::*;
use near_sdk::serde_json;

#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum PriceReceiverMsg {
    Execute { actions: Vec<Action> },
}

#[near_bindgen]
impl OraclePriceReceiver for Contract {
    /// The method will execute a given list of actions in the msg using the prices from the `data`
    /// provided by the oracle on behalf of the sender_id.
    /// - Requires to be called by the oracle account ID.
    fn oracle_on_call(&mut self, sender_id: ValidAccountId, data: PriceData, msg: String) {
        assert_eq!(env::predecessor_account_id(), self.get_oracle_account_id());

        let actions = match serde_json::from_str(&msg).expect("Can't parse PriceReceiverMsg") {
            PriceReceiverMsg::Execute { actions } => actions,
        };

        let mut account = self.internal_unwrap_account(sender_id.as_ref());
        self.internal_execute(sender_id.as_ref(), &mut account, actions, data.into());
        self.internal_set_account(sender_id.as_ref(), account);
    }
}
