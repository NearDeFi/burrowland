mod setup;

use crate::setup::*;

fn basic_setup() -> (Env, Tokens, Users) {
    let e = Env::init();
    let tokens = Tokens::init(&e);
    e.setup_assets(&tokens);
    e.deposit_reserves(&tokens);

    let users = Users::init(&e);
    e.mint_tokens(&tokens, &users.alice);
    e.mint_tokens(&tokens, &users.bob);

    (e, tokens, users)
}

#[test]
fn test_init_env() {
    let e = Env::init();
    let _tokens = Tokens::init(&e);
    let _users = Users::init(&e);
}

#[test]
fn test_mint_tokens() {
    let e = Env::init();
    let tokens = Tokens::init(&e);
    let users = Users::init(&e);
    e.mint_tokens(&tokens, &users.alice);
}

#[test]
fn test_dev_setup() {
    let e = Env::init();
    let tokens = Tokens::init(&e);
    e.setup_assets(&tokens);
    e.deposit_reserves(&tokens);
}

#[test]
fn test_supply() {
    let (e, tokens, users) = basic_setup();
}
