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
    DepositToReserve,
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Receives the transfer from the fungible token and executes a list of actions given in the
    /// message on behalf of the sender. The actions that can be executed should be limited to a set
    /// that doesn't require pricing.
    /// - Requires to be called by the fungible token account.
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_id = env::predecessor_account_id();
        let mut asset = self.internal_unwrap_asset(&token_id);
        assert!(
            asset.config.can_deposit,
            "Deposits for this asset are not enabled"
        );

        let amount = amount.0 * 10u128.pow(asset.config.extra_decimals as u32);

        // TODO: We need to be careful that only whitelisted tokens can call this method with a
        //     given set of actions. Or verify which actions are possible to do.
        let actions: Vec<Action> = if msg.is_empty() {
            vec![]
        } else {
            let token_receiver_msg: TokenReceiverMsg =
                serde_json::from_str(&msg).expect("Can't parse TokenReceiverMsg");
            match token_receiver_msg {
                TokenReceiverMsg::Execute { actions } => actions,
                TokenReceiverMsg::DepositToReserve => {
                    asset.reserved += amount;
                    self.internal_set_asset(&token_id, asset);
                    log!(
                        "Account {} deposits to reserve {} of {}",
                        sender_id.as_ref(),
                        amount,
                        token_id
                    );
                    return PromiseOrValue::Value(U128(0));
                }
            }
        };

        let (mut account, mut storage) =
            self.internal_unwrap_account_with_storage(sender_id.as_ref());
        account.add_affected_farm(FarmId::Supplied(token_id.clone()));
        self.internal_deposit(&mut account, &token_id, amount);
        log!("Account {} deposits {} of {}", sender_id, amount, token_id);
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
        token_id: &TokenId,
        amount: Balance,
    ) -> Promise {
        let asset = self.internal_unwrap_asset(token_id);
        let ft_amount = amount / 10u128.pow(asset.config.extra_decimals as u32);
        ext_fungible_token::ft_transfer(
            account_id.clone(),
            ft_amount.into(),
            None,
            token_id,
            ONE_YOCTO,
            GAS_FOR_FT_TRANSFER,
        )
        .then(ext_self::after_ft_transfer(
            account_id.clone(),
            token_id.clone(),
            amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_AFTER_FT_TRANSFER,
        ))
    }
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn after_ft_transfer(&mut self, account_id: AccountId, token_id: TokenId, amount: U128)
        -> bool;
}

trait ExtSelf {
    fn after_ft_transfer(&mut self, account_id: AccountId, token_id: TokenId, amount: U128)
        -> bool;
}

#[near_bindgen]
impl ExtSelf for Contract {
    #[private]
    fn after_ft_transfer(
        &mut self,
        account_id: AccountId,
        token_id: TokenId,
        amount: U128,
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success {
            let (mut account, storage) = self.internal_unwrap_account_with_storage(&account_id);
            account.add_affected_farm(FarmId::Supplied(token_id.clone()));
            self.internal_deposit(&mut account, &token_id, amount.0);
            log!(
                "Withdrawal has failed: Account {} deposits {} of {}",
                account_id,
                amount.0,
                token_id
            );
            self.internal_set_account(&account_id, account, storage);
        }
        promise_success
    }
}
