import 'regenerator-runtime/runtime'

/*
export NEAR_ENV=testnet
export OWNER_ID=dev-1634805788173-90290670203248
export ORACLE_ID=priceoracle.testnet
export CONTRACT_ID=dev-1634805794596-68676200580325
export BOOSTER_TOKEN_ID=ref.fakes.testnet
export WETH_TOKEN_ID=weth.fakes.testnet
export DAI_TOKEN_ID=dai.fakes.testnet
export USDT_TOKEN_ID=usdt.fakes.testnet
export WNEAR_TOKEN_ID=wrap.testnet
export ONE_YOCTO=0.000000000000000000000001
export GAS=200000000000000
export ACCOUNT_ID=dev-1634809261182-39559532470451

 */
const contract = require('./rest-api-test-utils');
const utils = require('./utils');

const alice = process.env.ACCOUNT_ID;
const contract_id = process.env.CONTRACT_ID;
const usdt_contract_id = process.env.USDT_TOKEN_ID;
const dai_contract_id = process.env.DAI_TOKEN_ID;
const oracle_contract_id = process.env.ORACLE_ID;

const bob = "place.testnet";

const burrow = new contract(contract_id);
const usdt = new contract(usdt_contract_id);
const dai = new contract(dai_contract_id);
const oracle = new contract(oracle_contract_id);

describe("Contract set", () => {
    test("Contract is not null " + contract_id, async () => {
        expect(contract_id).not.toBe(undefined)
    });

    test("USDT Contract is not null " + usdt_contract_id, async () => {
        expect(usdt_contract_id).not.toBe(undefined)
    });

    test("DAI Contract is not null " + usdt_contract_id, async () => {
        expect(dai_contract_id).not.toBe(undefined)
    });

    test("Oracle Contract is not null " + alice, async () => {
        expect(oracle_contract_id).not.toBe(undefined)
    });

    test("Alice Account is not null " + alice, async () => {
        expect(alice).not.toBe(undefined)
    });

    test('Alice has enough funds', async () => {
        const alice_wallet_balance = await burrow.accountNearBalance(alice, 0);
        expect(alice_wallet_balance).toBeGreaterThan(20);
    });
});

describe("Accounts", () => {
    test('Register account by paying for storage, deposit tokens', async () => {
        const storage_deposit = await burrow.call("storage_deposit",
            {}, {
                account_id: alice,
                tokens: utils.ConvertToNear(0.1),
                log_errors: true
            });
        expect(storage_deposit.is_error).toBeFalsy();

        const account_initial = await burrow.view("get_account",
            {account_id: alice}, {});

        // make ft deposit and check balance/shares
        const deposit_1 = 5;
        const asset_1 = await burrow.view("get_asset",
            {token_id: usdt_contract_id}, {});

        const ft_transfer_1 = await usdt.call("ft_transfer_call", {
            receiver_id: contract_id,
            amount: deposit_1.toString() + "000000",
            msg: ""
        }, {account_id: alice, tokens: 1})
        expect(ft_transfer_1.is_error).toBeFalsy();

        const account_1 = await burrow.view("get_account",
            {account_id: alice}, {});
        expect(account_1.account_id).toBe(alice);
        expect(account_1.supplied.length).toBeGreaterThan(0);

        const usdt_supplied_1 = account_1.supplied.filter(token => token.token_id === 'usdt.fakes.testnet');
        expect(usdt_supplied_1.length).toBe(1);

        const usdt_supplied_initial = account_initial.supplied.filter(token => token.token_id === 'usdt.fakes.testnet');

        expect(utils.ConvertFromFTe18(usdt_supplied_1[0]?.balance))
            .toBe(utils.ConvertFromFTe18(usdt_supplied_initial[0]?.balance) + deposit_1);
        expect(utils.ConvertFromFTe18(usdt_supplied_1[0]?.shares))
            .toBe(utils.ConvertFromFTe18(usdt_supplied_initial[0]?.shares) + deposit_1);

        const asset_2 = await burrow.view("get_asset",
            {token_id: usdt_contract_id}, {});
        expect(utils.ConvertFromFTe18(asset_2.supplied.shares)
            - utils.ConvertFromFTe18(asset_1.supplied.shares)).toBe(5)

        const deposit_2 = 3;
        const ft_transfer_2 = await usdt.call("ft_transfer_call", {
            receiver_id: contract_id,
            amount: deposit_2.toString() + "000000",
            msg: ""
        }, {account_id: alice, tokens: 1})
        expect(ft_transfer_2.is_error).toBeFalsy();

        const account_2 = await burrow.view("get_account",
            {account_id: alice}, {});

        const usdt_supplied_2 = account_2.supplied.filter(token => token.token_id === 'usdt.fakes.testnet');
        expect(usdt_supplied_2.length).toBe(1);

        expect(utils.ConvertFromFTe18(usdt_supplied_2[0]?.balance))
            .toBe(utils.ConvertFromFTe18(usdt_supplied_1[0]?.balance) + deposit_2);
        expect(utils.ConvertFromFTe18(usdt_supplied_2[0]?.shares))
            .toBe(utils.ConvertFromFTe18(usdt_supplied_1[0]?.shares) + deposit_2);

        const deposit_3 = 0;
        const ft_transfer_3 = await usdt.call("ft_transfer_call", {
            receiver_id: contract_id,
            amount: deposit_3.toString() + "000000",
            msg: ""
        }, {account_id: alice, tokens: 1})
        expect(ft_transfer_3.is_error).toBeTruthy();

        const account_3 = await burrow.view("get_account",
            {account_id: alice}, {});

        const usdt_supplied_3 = account_2.supplied.filter(token => token.token_id === 'usdt.fakes.testnet');
        expect(usdt_supplied_3.length).toBe(1);

        expect(utils.ConvertFromFTe18(usdt_supplied_3[0]?.balance))
            .toBe(utils.ConvertFromFTe18(usdt_supplied_2[0]?.balance));
        expect(utils.ConvertFromFTe18(usdt_supplied_3[0]?.shares))
            .toBe(utils.ConvertFromFTe18(usdt_supplied_2[0]?.shares));
    });
});

describe("Collateral", () => {
    test('Provide token as a collateral', async () => {
        const account_1 = await burrow.view("get_account",
            {account_id: alice}, {});
        const usdt_collateral_1 = account_1.collateral.filter(token => token.token_id === 'usdt.fakes.testnet');
        const usdt_supplied_1 = account_1.supplied.filter(token => token.token_id === 'usdt.fakes.testnet');

        const execute = await burrow.call("execute",
            {
                actions: [{
                    IncreaseCollateral: {token_id: usdt_contract_id}
                }]
            },
            {
                account_id: alice,
                tokens: 1,
                log_errors: true
            })
        expect(execute.is_error).toBeFalsy();

        const account_2 = await burrow.view("get_account",
            {account_id: alice}, {});

        const usdt_collateral_2 = account_2.collateral.filter(token => token.token_id === 'usdt.fakes.testnet');
        expect(usdt_collateral_2.length).toBe(1);

        expect(utils.ConvertFromFTe18(usdt_collateral_2[0].balance)
            - utils.ConvertFromFTe18(usdt_collateral_1[0]?.balance))
            .toBe(utils.ConvertFromFTe18(usdt_supplied_1[0].balance));
        expect(utils.ConvertFromFTe18(usdt_collateral_2[0].shares)
            - utils.ConvertFromFTe18(usdt_collateral_1[0]?.shares))
            .toBe(utils.ConvertFromFTe18(usdt_supplied_1[0].shares));

        const execute_wrong_account = await burrow.call("execute",
            {
                actions: [{
                    IncreaseCollateral: {"token_id": usdt_contract_id}
                }]
            },
            {account_id: bob, tokens: 1})
        expect(execute_wrong_account.is_error).toBeTruthy();

        const execute_without_supply = await burrow.call("execute",
            {
                actions: [{
                    IncreaseCollateral: {
                        token_id: usdt_contract_id
                    }
                }]
            },
            {account_id: alice, tokens: 1})
        expect(execute_without_supply.is_error).toBeTruthy();
    });
});


describe("Borrow", () => {
    test('Borrow a token', async () => {
        const account_1 = await burrow.view("get_account",
            {account_id: alice}, {});

        const borrow_amount_1 = 1;
        const execute = await oracle.call("oracle_call",
            {
                receiver_id: contract_id,
                asset_ids: [
                    'usdt.fakes.testnet',
                    'dai.fakes.testnet'
                ],
                msg: JSON.stringify({
                    Execute: {
                        actions:
                            [{
                                Borrow: {
                                    token_id: 'dai.fakes.testnet',
                                    amount: borrow_amount_1.toString() + "000000000000000000"
                                }
                            }]
                    }
                })
            },
            {
                account_id: alice,
                tokens: 1
            })
        expect(execute.is_error).toBeFalsy();

        const account_2 = await burrow.view("get_account",
            {account_id: alice}, {});

        expect(account_2.borrowed.length).toBe(1);
        expect(account_2.borrowed[0].token_id).toBe('dai.fakes.testnet');
        // check borrowed funds
        expect(utils.ConvertFromFTe18(account_2.borrowed[0].balance)
            - utils.ConvertFromFTe18(account_1?.borrowed[0]?.balance))
            .toBeCloseTo(borrow_amount_1);
        expect(utils.ConvertFromFTe18(account_2.borrowed[0].shares)
            - utils.ConvertFromFTe18(account_1?.borrowed[0]?.shares))
            .toBe(borrow_amount_1);

        // check supplied funds
        expect(utils.ConvertFromFTe18(account_2.supplied[0].balance)
            - utils.ConvertFromFTe18(account_1?.supplied[0]?.balance))
            .toBeCloseTo(borrow_amount_1);
        expect(utils.ConvertFromFTe18(account_2.supplied[0].shares)
            - utils.ConvertFromFTe18(account_1?.supplied[0]?.shares))
            .toBe(borrow_amount_1);
    });
});


describe("Withdraw", () => {
    test('Withdrawing the asset', async () => {
        const account_1 = await burrow.view("get_account",
            {account_id: alice}, {});

        const ft_balance_1 = await usdt.view("ft_balance_of",
            {account_id: alice},
            {convertFromFTe18: true})

        const withdraw = await burrow.call("execute",
            {
                actions: [{
                    Withdraw: {
                        token_id: 'dai.fakes.testnet'
                    }
                }]
            },
            {
                account_id: alice,
                tokens: 1,
                log_errors: true
            })
        expect(withdraw.is_error).toBeFalsy();

        const ft_balance_2 = await usdt.view("ft_balance_of",
            {account_id: alice},
            {convertFromFTe18: true})

        expect(ft_balance_2 - ft_balance_1)
            .toBeCloseTo(utils.ConvertFromFTe18(account_1.supplied.balance));

        const account_2 = await burrow.view("get_account",
            {account_id: alice}, {});

        expect(account_2.supplied.length).toBe(0);
    });
});


describe("Repay", () => {
    test('Deposit asset and repay it in one call', async () => {
        const account_1 = await burrow.view("get_account",
            {account_id: alice}, {});

        const repay_amount_1 = 5;

        const ft_transfer_1 = await dai.call("ft_transfer_call", {
            receiver_id: contract_id,
            amount: repay_amount_1.toString() + "000000000000000000",
            msg: JSON.stringify({
                Execute: {
                    actions: [
                        {
                            Repay: {
                                token_id: 'dai.fakes.testnet'
                            }
                        }
                    ]
                }
            })
        }, {account_id: alice, tokens: 1, log_errors: true})
        expect(ft_transfer_1.is_error).toBeFalsy();

        const account_2 = await burrow.view("get_account",
            {account_id: alice}, {});

        expect(account_2.borrowed.length).toBe(0);

        expect(utils.ConvertFromFTe18(account_1.borrowed[0].balance) +
            utils.ConvertFromFTe18(account_2.supplied[0].balance)).toBeCloseTo(repay_amount_1);
    });
});

describe("Decrease collateral", () => {
    test('Decreasing collateral without providing prices', async () => {
        const account_1 = await burrow.view("get_account",
            {account_id: alice}, {});
        const usdt_supplied_1 = account_1.supplied.filter(token => token.token_id === 'usdt.fakes.testnet');
        const usdt_collateral_1 = account_1.collateral.filter(token => token.token_id === 'usdt.fakes.testnet');

        const decrease_collateral_wrong_asset = await burrow.call("execute",
            {
                actions: [{
                    DecreaseCollateral: {
                        token_id: 'dai.fakes.testnet'
                    }
                }]
            },
            {
                account_id: alice,
                tokens: 1
            })
        expect(decrease_collateral_wrong_asset.is_error).toBeTruthy();

        const decrease_collateral = await burrow.call("execute",
            {
                actions: [{
                    DecreaseCollateral: {
                        token_id: 'usdt.fakes.testnet'
                    }
                }]
            },
            {
                account_id: alice,
                tokens: 1,
                log_errors: true
            })
        expect(decrease_collateral.is_error).toBeFalsy();

        const account_2 = await burrow.view("get_account", {account_id: alice}, {});
        const usdt_supplied_2 = account_2.supplied.filter(token => token.token_id === 'usdt.fakes.testnet');
        const usdt_collateral_2 = account_2.collateral.filter(token => token.token_id === 'usdt.fakes.testnet');

        expect(usdt_collateral_2.length).toBe(0);
        expect(usdt_supplied_2.length).toBeGreaterThan(0);
        expect(utils.ConvertFromFTe18(usdt_collateral_1[0]?.balance) +
            utils.ConvertFromFTe18(usdt_supplied_1[0]?.balance))
            .toBeCloseTo(utils.ConvertFromFTe18(usdt_supplied_2[0]?.balance));
        expect(utils.ConvertFromFTe18(usdt_collateral_1[0]?.shares) +
            utils.ConvertFromFTe18(usdt_supplied_1[0]?.shares))
            .toBeCloseTo(utils.ConvertFromFTe18(usdt_supplied_2[0]?.shares));
    });
});


describe("Liquidate", () => {
    test('Liquidate', async () => {
        const account_1 = await burrow.view("get_account",
            {account_id: alice}, {});

        console.log(account_1);

        const increaseCollateral = await burrow.call("execute",
            {
                actions: [{
                    IncreaseCollateral: {token_id: dai_contract_id}
                }]
            },
            {
                account_id: alice,
                tokens: 1,
                log_errors: true
            })
        expect(increaseCollateral.is_error).toBeFalsy();

        const borrow_amount_1 = 1;
        const execute = await oracle.call("oracle_call",
            {
                receiver_id: contract_id,
                asset_ids: [
                    'usdt.fakes.testnet',
                    'dai.fakes.testnet'
                ],
                msg: JSON.stringify({
                    Execute: {
                        actions:
                            [{
                                Borrow: {
                                    token_id: dai_contract_id,
                                    amount: borrow_amount_1.toString() + "000000000000000000"
                                }
                            }]
                    }
                })
            },
            {
                account_id: alice,
                tokens: 1
            })
        expect(execute.is_error).toBeFalsy();

        const liquidate = await burrow.call("execute",
            {
                actions: [{
                    Liquidate: {
                        token_id: 'usdt.fakes.testnet',
                        in_assets: {
                            token_id: 'dai.fakes.testnet',
                            max_amount: borrow_amount_1.toString() + "000000000000000000"
                        }
                    }
                }]
            },
            {
                account_id: bob,
                tokens: 1,
                log_errors: true
            })
        expect(liquidate.is_error).toBeFalsy();

        const account_2 = await burrow.view("get_account",
            {account_id: alice}, {});

        expect(account_2.borrowed.length).toBe(1);
        expect(account_2.borrowed[0].token_id).toBe('dai.fakes.testnet');
        // check borrowed funds
        expect(utils.ConvertFromFTe18(account_2.borrowed[0].balance)
            - utils.ConvertFromFTe18(account_1?.borrowed[0]?.balance))
            .toBeCloseTo(borrow_amount_1);
        expect(utils.ConvertFromFTe18(account_2.borrowed[0].shares)
            - utils.ConvertFromFTe18(account_1?.borrowed[0]?.shares))
            .toBe(borrow_amount_1);

        // check supplied funds
        expect(utils.ConvertFromFTe18(account_2.supplied[0].balance)
            - utils.ConvertFromFTe18(account_1?.supplied[0]?.balance))
            .toBeCloseTo(borrow_amount_1);
        expect(utils.ConvertFromFTe18(account_2.supplied[0].shares)
            - utils.ConvertFromFTe18(account_1?.supplied[0]?.shares))
            .toBe(borrow_amount_1);
    });
});

