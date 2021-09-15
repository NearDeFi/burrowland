use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
enum PriceReceiverMsg {
    DecreaseCollateral {},
    Borrow {},
    Liquidate {},
}

#[near_bindgen]
impl OraclePriceReceiver for Contract {
    fn oracle_on_call(&mut self, sender_id: AccountId, data: PriceData, msg: String) {
        todo!()
    }
}
