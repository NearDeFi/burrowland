const utils = require('./utils');
const config = require("./config");

function contract(contract_id) {
    this.contract_id = contract_id;
}

contract.prototype.deploy = async function (contractName) {
    const body = {
        contract: contractName,
        account_id: this.contract_id,
        private_key: await utils.getPrivateKey(this.contract_id),
    };

    return await utils.PostResponse("deploy", body);
};

contract.prototype.view = async function (method, params, options) {
    const body = {
        method: method,
        params: params,
        contract: this.contract_id,
        disabled_cache: true
    };

    return await utils.PostResponse("view", body, options);
};

contract.prototype.viewNearBalance = async function (method, params, options) {
    options = options || {};
    options.convertToNear = true;
    return await this.view(method, params, options);
};

contract.prototype.viewFTe18Balance = async function (method, params, options) {
    options = options || {};
    options.convertFromFTe18 = true;
    return await this.view(method, params, options);
};

contract.prototype.accountNearBalance = async function (account_id, delay) {
    delay = delay || 1000;
    await timeout(delay);

    return await utils.GetResponse("balance", account_id, {convertToNear: true})
        .catch(e => console.error("AccountNearBalance error for " + JSON.stringify(account_id) + ". Error: " + e.message));
};

contract.prototype.call = async function (method, params, options) {
    options.attached_gas = options.gas || config.GAS;
    options.attached_tokens = options.tokens || 0;
    options.private_key = options.private_key || await utils.getPrivateKey(options.account_id);
    options.log_errors = options.log_errors || false;
    options.return_value = options.return_value || false;

    const body = {
        ...options,
        method: method,
        params: params,
        contract: this.contract_id,
    };

    return await utils.PostResponse("call", body, options)
        .catch(e => {
            if(e.message.includes("Unexpected token < in JSON at position 0"))
                console.error("RPC/JSON Error: " + e.message);
            else
                console.error("Call error for " + JSON.stringify(body) + ". Error: " + e.message)
        });
};

function timeout(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

module.exports = contract;
