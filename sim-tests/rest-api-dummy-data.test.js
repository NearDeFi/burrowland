/* export REACT_CONTRACT_ID=dev-1627393733545-88687685295664 */
import 'regenerator-runtime/runtime'

const contract = require('./rest-api-test-utils');
const utils = require('./utils');

const contract_id  = process.env.CONTRACT_NAME;

const near = new contract(process.env.CONTRACT_NAME);

const MIN_TOKENS = 20;

describe("Contract set", () => {
    test("Contract set: " + process.env.CONTRACT_NAME, async () => {
        expect(process.env.CONTRACT_NAME).not.toBe(undefined)
    });

    test('Accounts has enough funds', async () => {
        const contract_wallet_balance = await near.accountNearBalance(contract_id);
        expect(contract_wallet_balance).toBeGreaterThan(MIN_TOKENS);
    });
});


describe("Insert dummy data", () => {
    console.log(contract_id)
    test('Insert players', async () => {
        for (let i = 500; i <= 600; i++) {
            let account = `account_${i}.testnet`;
            const start_game = await near.call("start_game_for_account_id",
                {quiz_id: 36, account_id: account},
                {account_id: contract_id});
            expect(start_game.type).not.toBe('FunctionCallError');
        }
    });
});
