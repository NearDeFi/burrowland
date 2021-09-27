use crate::*;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::{is_promise_success, serde_json, PromiseOrValue};

const GAS_FOR_FT_TRANSFER: Gas = 10 * TGAS;
const GAS_FOR_AFTER_FT_TRANSFER: Gas = 20 * TGAS;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TokenReceiverMsg {
    Execute { actions: Vec<Action> },
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // TODO: We need to be careful that only whitelisted tokens can call this method with a
        //     given set of actions.
        let token_account_id = env::predecessor_account_id();

        let actions: Vec<Action> = if msg.is_empty() {
            vec![]
        } else {
            match serde_json::from_str(&msg).expect("Can't parse TokenReceiverMsg") {
                TokenReceiverMsg::Execute { actions } => actions,
            }
        };

        let (mut account, mut storage) =
            self.internal_unwrap_account_with_storage(sender_id.as_ref());
        self.internal_deposit(&mut account, &token_account_id, amount.0);
        self.internal_execute(
            sender_id.as_ref(),
            &mut account,
            &mut storage,
            actions,
            Prices::new(),
        );
        self.internal_set_account(sender_id.as_ref(), account, storage);

        PromiseOrValue::Value(U128(0))
    }
}

impl Contract {
    pub fn internal_ft_transfer(
        &mut self,
        account_id: &AccountId,
        token_account_id: &TokenAccountId,
        amount: Balance,
    ) -> Promise {
        ext_fungible_token::ft_transfer(
            account_id.clone(),
            amount.into(),
            None,
            token_account_id,
            ONE_YOCTO,
            GAS_FOR_FT_TRANSFER,
        )
        .then(ext_self::after_ft_transfer(
            account_id.clone(),
            token_account_id.clone(),
            amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_AFTER_FT_TRANSFER,
        ))
    }
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn after_ft_transfer(
        &mut self,
        account_id: AccountId,
        token_account_id: TokenAccountId,
        amount: U128,
    ) -> bool;
}

trait ExtSelf {
    fn after_ft_transfer(
        &mut self,
        account_id: AccountId,
        token_account_id: TokenAccountId,
        amount: U128,
    ) -> bool;
}

#[near_bindgen]
impl ExtSelf for Contract {
    #[private]
    fn after_ft_transfer(
        &mut self,
        account_id: AccountId,
        token_account_id: TokenAccountId,
        amount: U128,
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success {
            let (mut account, storage) = self.internal_unwrap_account_with_storage(&account_id);
            self.internal_deposit(&mut account, &token_account_id, amount.0);
            self.internal_set_account(&account_id, account, storage);
        }
        promise_success
    }
}
