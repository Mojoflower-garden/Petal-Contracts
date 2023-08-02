#![no_std]

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
    Map,
};

mod erc721 {
    soroban_sdk::contractimport!(
        file = "../token/target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
}

#[contract]
pub struct PetalDocuments;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub deadline: u64,
    pub description: String,
    pub document_hash: BytesN<32>,
    pub document_uri: String,
    pub signer: Address,
    pub status: SignatureStatus,
    pub token_id: u32,
    pub nonce: u32,
}

const NONCES: Symbol = symbol_short!("NONCES");
const T2DHASH: Symbol = symbol_short!("T2DHASH");
const DEADLINES: Symbol = symbol_short!("DEADLINES");
const DOCSIGN: Symbol = symbol_short!("DOCSIGN");
const CREACTION_FEE: Symbol = symbol_short!("crea_fee");


#[contractimpl]
impl PetalDocuments {
    // pub fn init(env: Env, addresses: Vec<Val>) {

    //     log!(&env, "TEST: ", addresses);
    //     env.storage().instance().set(&SIGNERS, &addresses);

    // }
    // pub fn value(env: Env) -> Vec<Val> {
    //     let current_addresses: Option<Vec<Val>> = env.storage().instance().get(&SIGNERS);

    //     log!(&env, "GETTING ADDRESSES: ", current_addresses);
    //     // current_addresses.unwrap()
    //     env.storage().instance().get(&SIGNERS).unwrap_or_else(|| panic!("Admin not found"));
    // }

    // pub fn test(env: Env, to: Vec<Address>) {

    //     // env.storage().instance().set(&SIGNERS, hehu);
    // }
    // soroban contract install --wasm ../token/target/wasm32-unknown-unknown/release/soroban_token_contract.wasm

    pub fn sign_document(e: Env, erc721_address: Address, user: Address, signature: Bytes, payload: SignedMessage) {
        let client = erc721::Client::new(&e, &erc721_address);
        let is_token_minted: bool = client.require_minted(&payload.token_id);
        if is_token_minted == false {
            panic!("ERC721: invalid token ID")
        }
        let mut doc_signings: Map<u32, Map<Address, SignatureStatus>> = e.storage().instance().get(&DOCSIGN).unwrap_or(Map::new(&e));
        if doc_signings.is_empty() {
            panic!("Document signings is empty")
        }
        let clone_signer = payload.signer.clone();
        let all_signings = doc_signings.get(payload.token_id);
        let signer_status = match all_signings {
            Some(signing) => {
                let is_signer = signing.get(payload.signer);
                match is_signer {
                    Some(signer) => {
                        if signer == SignatureStatus::NotASigner {
                            panic!("Not a signer - 401");
                        } else if signer == SignatureStatus::Signed {
                            panic!("Signer has already signed - 401");
                        }
                        signer
                    },
                    None => {
                        panic!("Signer does not exist");
                    }
                }
            },
            None => {
                panic!("Document signings is empty");
            }
        };

        let token_to_doc_hashes: Map<u32, BytesN<32>> = e.storage().instance().get(&T2DHASH).unwrap_or(Map::new(&e));
        if token_to_doc_hashes.is_empty() {
            panic!("Document hashes is empty")
        }

        let doc_hash = token_to_doc_hashes.get(payload.token_id);
        let matched_hash = match doc_hash {
            Some(hash) => {
                if (hash != payload.document_hash) {
                    panic!("The document hash must match the hash of the token document.")
                }
                hash
            },
            None => {
                panic!("Hash not found")
            }
        };

        let doc_signing_deadlines: Map<u32, u64> = e.storage().instance().get(&DEADLINES).unwrap_or(Map::new(&e));
        if doc_signing_deadlines.is_empty() {
            panic!("Document deadlines is empty")
        }
        let deadlines = doc_signing_deadlines.get(payload.token_id);
        let deadline = match deadlines {
            Some(v) => {
                if e.ledger().timestamp() > v {
                    panic!("Document's deadline passed - 410")
                }
                v
            },
            None => {
                panic!("Deadline not found")
            }
        };

        // EQUIVALENT TO SOLIDITY
        // bytes32 digest = _hashTypedDataV4(
		// 	keccak256(
		// 		abi.encode(
		// 			keccak256(
		// 				'signedMessage(uint256 deadline,string description,bytes32 documentHash,string documentUri,address signer,uint8 status,uint256 tokenId,uint256 nonce)'
		// 			),
		// 			payload.deadline,
		// 			keccak256(abi.encodePacked(payload.description)), // https://ethereum.stackexchange.com/questions/131282/ethers-eip712-wont-work-with-strings
		// 			payload.documentHash,
		// 			keccak256(abi.encodePacked(payload.documentUri)),
		// 			payload.signer,
		// 			payload.status,
		// 			payload.tokenId,
		// 			signatureNonces[payload.signer]
		// 		)
		// 	)
		// );
		// address signer = ECDSA.recover(digest, signature);
        // require(signer == payload.signer, 'Invalid signature - 401');
		// require(signer != address(0), 'Invalid signature - 401');

        if e.ledger().timestamp() > payload.deadline {
            panic!("Signature expired")
        };
        let clone_singer_2 = clone_signer.clone();
        let clone_singer_3 = clone_singer_2.clone();
        let mut signature_nonces: Map<Address, u32> = e.storage().instance().get(&NONCES).unwrap_or(Map::new(&e));
        let last_nonce = signature_nonces.get(clone_signer).unwrap_or(0);
        if signature_nonces.is_empty() {
            signature_nonces.set(clone_singer_2, last_nonce);
        } else {
            signature_nonces.set(clone_singer_2, last_nonce + 1);
        }

        doc_signings.get(payload.token_id).unwrap().set(clone_singer_3, payload.status);
        e.storage().instance().set(&DOCSIGN, &doc_signings);
    }

    pub fn safe_mint(e: Env, erc721_address: Address, to: Address, token_id: u32, meta_uri: String, signers: Vec<Address>, document_hash: BytesN<32>, deadline: u64) -> u32 {
        if signers.is_empty() {
            panic!("Must have some signers for each document");
        }
        log!(&e, "Hello");
        let client = erc721::Client::new(&e, &erc721_address);
        client.mint(&token_id, &to);
        client.set_token_uri(&token_id, &meta_uri);

        let mut token_to_doc_hashes: Map<u32, BytesN<32>> = e.storage().instance().get(&T2DHASH).unwrap_or(Map::new(&e));
        token_to_doc_hashes.set(token_id, document_hash);

        let mut doc_signing_deadlines: Map<u32, u64> = e.storage().instance().get(&DEADLINES).unwrap_or(Map::new(&e));
        doc_signing_deadlines.set(token_id, deadline);
        
        let mut doc_signings: Map<u32, Map<Address, SignatureStatus>> = e.storage().instance().get(&DOCSIGN).unwrap_or(Map::new(&e));
        let mut inner_doc_signings: Map<Address, SignatureStatus> = doc_signings.get(token_id).unwrap_or(Map::new(&e));

        for signer in signers.iter() {
            inner_doc_signings.set(signer, SignatureStatus::Waiting);
            let cloned_inner_doc_signings = inner_doc_signings.clone();
            doc_signings.set(token_id, cloned_inner_doc_signings);
        };

        token_id
    }

    pub fn get_td_hashes(e: Env) -> Map<u32, BytesN<32>> {
        let token_to_doc_hashes: Map<u32, BytesN<32>> = e.storage().instance().get(&T2DHASH).unwrap_or(Map::new(&e));
        token_to_doc_hashes
    }

    pub fn get_deadlines(e: Env) -> Map<u32, u64> {
        let deadlines: Map<u32, u64> = e.storage().instance().get(&DEADLINES).unwrap_or(Map::new(&e));
        deadlines
    }

    pub fn get_signatures(e: Env) -> Map<u32, Map<Address, SignatureStatus>> {
        let doc_signings: Map<u32, Map<Address, SignatureStatus>> = e.storage().instance().get(&DOCSIGN).unwrap_or(Map::new(&e));
        doc_signings
    }
}

// ------------> CURRENT CONTRACT ID = CDZFLWTJUMT6MK57XHWIGSJHOKR4W55MBYXKKSYLS3W2SCAB2IZQTTVO --------------------

//     soroban contract deploy \
//     --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
//     --source juico \
//     --network standalone

//     soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
// --id CDZFLWTJUMT6MK57XHWIGSJHOKR4W55MBYXKKSYLS3W2SCAB2IZQTTVO \
//     --source juico \
//     --network standalone \
//     -- \
//     getSignatures 

// soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
// --id CDZFLWTJUMT6MK57XHWIGSJHOKR4W55MBYXKKSYLS3W2SCAB2IZQTTVO \
//     --source juico \
//     --network standalone \
//     -- \
//     safeMint \
//     --erc721_address CB7OSKDWKBCFNW2CEQI67RFSGWLOIS3EKAXIQDJSIWMPZLDMJ7SHSPRK \
//     --to GDOB4GMX45VENP4YMUQMH4ZJ6KJZTERQVOASFTXC7OMOZM5EFKPFU4X5 \
//     --token_id 1234 \
//     --meta_uri "www.help.com" \
//     --signers '[{"Address": GBRVKHUULGOAU2ADSZZKFH2DZBZF2S4PXVEMSE23PPTFZDST464RDHIM}]' \
//     --deadline 1679905807890 \
//     --document_hash '[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32]'

// soroban contract install --wasm ../token/target/wasm32-unknown-unknown/release/petal_documents.wasm \
//     --source SBS6SYYI2B2POLEUTLHA63SQVK24YAGCV2XPZ5PCVGH2CQPNFGQKNUIE \
//     --rpc-url http://localhost:8000/soroban/rpc \
//     --network-passphrase 'Standalone Network ; February 2017'



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