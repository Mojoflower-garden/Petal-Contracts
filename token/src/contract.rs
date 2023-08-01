#![no_std]

use crate::admin::{has_administrator, read_administrator, write_administrator};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{is_authorized, write_authorization};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::event;
use crate::metadata::{read_decimal, read_name, read_symbol, write_metadata};
use crate::storage_types::INSTANCE_BUMP_AMOUNT;
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, symbol_short, Map, BytesN, Vec, log};
use crate::custom_token_metadata::{CustomTokenMetadata};
use crate::erc_functions::{owner_of, exists};

pub trait TokenTrait {
    fn initialize(e: Env, admin: Address, token_id: u32);

    fn allowance(e: Env, from: Address, spender: Address) -> i128;

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32);

    fn balance(e: Env, id: Address) -> i128;

    fn spendable_balance(e: Env, id: Address) -> i128;

    fn authorized(e: Env, id: Address) -> bool;

    fn transfer(e: Env, from: Address, to: Address, amount: i128);

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128);

    fn burn(e: Env, from: Address, amount: i128);

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128);

    fn clawback(e: Env, from: Address, amount: i128);

    fn set_authorized(e: Env, id: Address, authorize: bool);

    fn mint(e: Env, to: Address, token_id: u32);

    fn set_admin(e: Env, new_admin: Address);

    fn get_admin(e: Env) -> Address;

    fn decimals(e: Env) -> u32;

    fn name(e: Env) -> String;

    fn symbol(e: Env) -> String;

    fn test(e: Env);

    fn get_owners(e: Env) -> Map<u32, Address>;

    fn set_owners(e: Env, token_id: u32, owner: Address);

    fn set_token_uri(e: Env, token_id: u32, token_uri: String);
}

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

const OWNERS: Symbol = symbol_short!("OWNERS");
const URIS: Symbol = symbol_short!("URIS");
const APPROVALS: Symbol = symbol_short!("approvals");
const OWNED_TOKEN_COUNT: Symbol = symbol_short!("tCount");
const OPERATOR_APPROVAL: Symbol = symbol_short!("opApprov");

#[contract]
pub struct Token;

#[contractimpl]
impl TokenTrait for Token {
    fn initialize(e: Env, admin: Address, token_id: u32) {
        if has_administrator(&e) {
            panic!("already initialized")
        }

        write_administrator(&e, &admin);

        let admin = read_administrator(&e);

        log!(&e, "Admin {}", admin);

        let mut owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        owners.set(token_id, admin);
        e.storage().instance().set(&OWNERS, &owners);

        log!(&e, "Done Initializing");

        // if decimal > u8::MAX.into() {
        //     panic!("Decimal must fit in a u8");
        // }

        // write_metadata(
        //     &e,
        //     CustomTokenMetadata {
        //         decimal,
        //         name,
        //         symbol,
        //         token_uri
        //     },
        // )
    }

    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);
        read_allowance(&e, from, spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);

        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        event::approve(&e, from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);
        read_balance(&e, id)
    }

    fn spendable_balance(e: Env, id: Address) -> i128 {
        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);
        read_balance(&e, id)
    }

    fn authorized(e: Env, id: Address) -> bool {
        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);
        is_authorized(&e, id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        event::transfer(&e, from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        event::transfer(&e, from, to, amount)
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        event::burn(&e, from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        event::burn(&e, from, amount)
    }

    fn clawback(e: Env, from: Address, amount: i128) {
        check_nonnegative_amount(amount);
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        event::clawback(&e, admin, from, amount);
    }

    fn set_authorized(e: Env, id: Address, authorize: bool) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);

        write_authorization(&e, id.clone(), authorize);
        event::set_authorized(&e, admin, id, authorize);
    }

    fn test(e:Env) {
        log!(&e, "Hello 1");
    }

    fn mint(e: Env, to: Address, token_id: u32) {
        let admin = read_administrator(&e);
        admin.require_auth();

        log!(&e, "Hello 1");

        let mut owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));

        log!(&e, "Hello 2 {}", owners);

        if !exists(&e, token_id, &owners) {
            panic!("Token already minted!");
        }

        owners.set(token_id, to);
        e.storage().instance().set(&OWNERS, &owners);

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);
        // event::mint(&e, admin, to, token_id);
    }

    fn get_owners(e: Env) -> Map<u32, Address> {
        let owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        log!(&e, "Owners {}", owners);
        owners
    }

    fn set_owners(e: Env, token_id: u32, owner: Address) {
        let mut owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        owners.set(token_id, owner);
        e.storage().instance().set(&OWNERS, &owners);
    }

    fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);

        write_administrator(&e, &new_admin);
        event::set_admin(&e, admin, new_admin);
    }

    fn get_admin(e: Env) -> Address {
        let admin = read_administrator(&e);
        admin
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> String {
        read_name(&e)
    }

    fn symbol(e: Env) -> String {
        read_symbol(&e)
    }

    fn set_token_uri(e: Env, token_id: u32, token_uri: String) {
        let owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));

        if !exists(&e, token_id, &owners) {
            panic!("Token already minted!");
        }

        let mut token_uris: Map<u32, String> = e.storage().instance().get(&URIS).unwrap_or(Map::new(&e));

        token_uris.set(token_id, token_uri);
    }
}

//STEPS TO MINT: 
// 1. Initialize with admin
//

// soroban contract build --profile release-with-logs

// soroban contract deploy \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id b
// soroban contract deploy \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --source juico \
//     --network standalone

// soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
// --id CCC7BOTWWO5LDI7Z2DOLGGSRWYHBG5IBIR7FI4F36JNS6DLBBZM2DNMN \
//     -- \
//     initialize \
//     --admin GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4 \
//     --token_id 1111

// soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
// --id CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAXI7N \
//     -- \
//     mint \
//     --to GA3YIJVTHQIH3BXKQHUHAYHBZ7Z5NYPPWIXXT3OHVQO5YE3RKT5ASAFC \
//     --token_id 1111

//     soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
// --id CCB56YW6GSBHZRA2H5YNV5HY2ZC3CYYH3XTIEYVQI5HT4PAEY5VTN3DP \
//     --source juico \
//     --network standalone \
//     -- \
//     set_owners \
//     --token_id 4 \
//     --owner GA3YIJVTHQIH3BXKQHUHAYHBZ7Z5NYPPWIXXT3OHVQO5YE3RKT5ASAFC

//     soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
// --id CCC7BOTWWO5LDI7Z2DOLGGSRWYHBG5IBIR7FI4F36JNS6DLBBZM2DNMN \
//     --source juico \
//     --network standalone \
//     -- \
//     get_owners 

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id b \
//     -- \
//     test

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     balance \
//     --id GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     initialize \
//     --admin GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4 \
//     --decimal 4 \
//     --name test \
//     --symbol TST \
//     --token_uri www.test.com 

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     set_admin \
//     --new_admin GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4 

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     authorized \
//     --id GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     balance \
//     --id GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4

// soroban contract invoke \
//     --wasm target/wasm32-unknown-unknown/release-with-logs/soroban_token_contract.wasm \
//     --id 1 \
//     -- \
//     set_authorized \
//     --id GDCBSTFJSOIN5FWZ22AMOBT56AILRZ3Z2UTL6GDYG4OYRWDAWOFA5ZT4 