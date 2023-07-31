use soroban_sdk::{
    contract, 
    contractimpl, 
    contracttype, 
    symbol_short, 
    vec, 
    Env, 
    Symbol, 
    Vec, 
    Address, 
    String, 
    BytesN, 
    Bytes,
    U256,
    log,
    Val,
    Map
};

mod token_contract {
    soroban_sdk::contractimport!(
        file = "../token/target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
}

#[contract]
pub struct PetalDocuments;

#[derive(Clone, Debug)]
#[contracttype]
pub enum SignatureStatus {
    NotASigner,
    Rejected,
    Signed,
    Waiting
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct SignedMessage {
    pub deadline: U256,
    pub description: String,
    pub document_hash: BytesN<32>,
    pub document_uri: String,
    pub signer: Address,
    pub status: SignatureStatus,
    pub token_id: U256,
    pub nonce: U256,
}

const SIGNERS: Symbol = symbol_short!("addresses");
const DOCUMENT_SIGNINGS: Symbol = symbol_short!("doc_sign");
const CREACTION_FEE: Symbol = symbol_short!("crea_fee");

#[contractimpl]
impl PetalDocuments {
    pub fn init(env: Env, addresses: Vec<Val>) {
        log!(&env, "TEST: ", addresses);
        env.storage().instance().set(&SIGNERS, &addresses);

    }
    pub fn value(env: Env) -> Vec<Val> {
        let current_addresses: Option<Vec<Val>> = env.storage().instance().get(&SIGNERS);
        log!(&env, "GETTING ADDRESSES: ", current_addresses);
        current_addresses.unwrap()
        // env.storage().instance().get(&SIGNERS).unwrap()
    }

    // pub fn sign_document(env: Env, user: Address, signature: Bytes, payload: SignedMessage, ) {
    //     let client = token_contract::Client::new(&env, &contract);
    //     // if(payload.)
    // }

    // pub fn safe_mint(env: Env, to: Address, from: Address, metaUri: String, signers: Vec<Address>, documentHash: BytesN<32>, deadline: U256, tokenId: U256) {
    
    //     let creation_fee: i128 = env.storage().instance().get(&CREACTION_FEE).unwrap_or(0);
    //     let client = token_contract::Client::new(&env, &contract);
    //     let sender_balance: i128 = client.balance(&from);

    //     if sender_balance >= creation_fee {
    //         panic!("Creation fee not met: {}", sender_balance)
    //     }

    //     if signers.is_empty() {
    //         panic!("Must have some signers for each document: {}", signers.is_empty())
    //     };

    //     client.mint(&to, tokenId);
    //     client.setTokenUri(tokenId, metaUri);
    // }
}

// soroban contract install --wasm ../petal_documents/target/wasm32-unknown-unknown/release/petal_documents.wasm \
//     --source SBS6SYYI2B2POLEUTLHA63SQVK24YAGCV2XPZ5PCVGH2CQPNFGQKNUIE \
//     --rpc-url http://localhost:8000/soroban/rpc \
//     --network-passphrase 'Standalone Network ; February 2017'

// soroban contract deploy \
//     --wasm target/wasm32-unknown-unknown/release/petal_deployer_contract.wasm \
//     --source SBS6SYYI2B2POLEUTLHA63SQVK24YAGCV2XPZ5PCVGH2CQPNFGQKNUIE \
//     --rpc-url http://localhost:8000/soroban/rpc \
//     --network-passphrase 'Standalone Network ; February 2017'

// soroban contract invoke \
//     --id CCXRBYYXDTO7FER3KRBBJMB6SCHPUI47OUQR7FNW5FHMYFRWM2VZLJTH \
//     --source SBS6SYYI2B2POLEUTLHA63SQVK24YAGCV2XPZ5PCVGH2CQPNFGQKNUIE \
//     --rpc-url http://localhost:8000/soroban/rpc \
//     --network-passphrase 'Standalone Network ; February 2017' \
//     -- \
//     deploy \
//     --salt 0000000000000000000000000000000000000000000000000000000000000000 \
//     --wasm_hash 5f6a731e431449467489090148443146ece3da6ee33f26ebd67a258624a94b3c \
//     --init_fn init \
//     --init_args '[{"u32":5}]' \
//     --deployer GBLJ2KROIRNWFIITBBQZFZIZIZ6GLTTZCTI2FTJNNAJ67MNW7O6LXAFK

//     Deployer contract: CCXRBYYXDTO7FER3KRBBJMB6SCHPUI47OUQR7FNW5FHMYFRWM2VZLJTH

// soroban contract invoke \
//     --id 5f6a731e431449467489090148443146ece3da6ee33f26ebd67a258624a94b3c \
//     --source SBS6SYYI2B2POLEUTLHA63SQVK24YAGCV2XPZ5PCVGH2CQPNFGQKNUIE \
//     --rpc-url http://localhost:8000/soroban/rpc \
//     --network-passphrase 'Standalone Network ; February 2017' \
//     -- \
//     value


// LOCAL

// FIRST INSTALL IDENTITY PASSING IN SECRET KEY
// soroban config identity add test --secret-key

// soroban contract install --wasm ../petal_documents/target/wasm32-unknown-unknown/release/petal_documents.wasm

// soroban contract invoke \
//     --source-account test \
//     --wasm target/wasm32-unknown-unknown/release/petal_deployer_contract.wasm \
//     --id 0 \
//     -- \
//     deploy \
//     --salt 0000000000000000000000000000000000000000000000000000000000000003 \
//     --wasm_hash f48c446b96a2cb599860998e6489a7b8235ab20e4ea440ee4b512dc85ddfdfa5 \
//     --init_fn init \
//     --init_args '[{"vec":[{"string": "test"}, {"string": "test2"}]}]' \
//     --deployer GBCEL3EZJTX7J5JXVMPKXMXWEWUQXMUZJBT3G3ABX7AXVC652HHIWDIK

// soroban contract invoke \
//     --source-account test \
//     --id CAJLVVVSO2MOTFMSUWOU4ELRJSFCFNSOWFAI6OGLPSXT6PWJ65DON2HO \
//     -- \
//     value