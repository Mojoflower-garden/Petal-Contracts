#![no_std]

mod storage_types;
use crate::storage_types::INSTANCE_BUMP_AMOUNT;

mod erc_functions;
use crate::erc_functions::{exists};

mod event;

mod admin;
use crate::admin::{has_administrator, read_administrator, write_administrator};

use soroban_sdk::{
    contract, 
    contractimpl, 
    contracttype, 
    symbol_short, 
    Env, 
    Symbol, 
    Vec, 
    Address, 
    String, 
    BytesN, 
    Bytes,
    log,
    Val,
    Map,
};

// mod erc721 {
//     soroban_sdk::contractimport!(
//         file = "../token/target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
//     );
// }

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
    pub document_hash: String,
    pub document_uri: String,
    pub signer: Address,
    pub status: SignatureStatus,
    pub token_id: u32,
    pub nonce: u32,
}

const OWNERS: Symbol = symbol_short!("OWNERS");
const URIS: Symbol = symbol_short!("URIS");

const NONCES: Symbol = symbol_short!("NONCES");
const T2DHASH: Symbol = symbol_short!("T2DHASH");
const DEADLINES: Symbol = symbol_short!("DEADLINES");
const DOCSIGN: Symbol = symbol_short!("DOCSIGN");
const CREACTION_FEE: Symbol = symbol_short!("crea_fee");


#[contractimpl]
impl PetalDocuments {
    pub fn init(e: Env, admin: Address, token_id: u32) {
        if has_administrator(&e) {
            panic!("already initialized")
        }

        write_administrator(&e, &admin);
    }

    pub fn sign_document(e: Env, erc721_address: Address, user: Address, signature: BytesN<64>, message: Bytes, payload: SignedMessage) {
        // let client = erc721::Client::new(&e, &erc721_address);
        // let is_token_minted: bool = client.require_minted(&payload.token_id);
        let is_token_minted: bool = Self::require_minted(&e, payload.token_id);
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

        let token_to_doc_hashes: Map<u32, String> = e.storage().instance().get(&T2DHASH).unwrap_or(Map::new(&e));
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
        
        

        // e.crypto().ed25519_verify(&user, &message, &signature);

        // EQUIVALENT TO SOLIDITY
        // THIS MEANS THAT USE HAVE SEEN THE CONTRACT AND DATA AND SIGNS IT TO AGREE -> THERE IS NO STELLAR SOROBAN EQUIVALENT TO THIS STANDARD
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
        let clone_signer_2 = clone_signer.clone();
        let clone_signer_3 = clone_signer_2.clone();
        let mut signature_nonces: Map<Address, u32> = e.storage().instance().get(&NONCES).unwrap_or(Map::new(&e));
        let last_nonce = signature_nonces.get(clone_signer).unwrap_or(0);
        if signature_nonces.is_empty() {
            signature_nonces.set(clone_signer_2, last_nonce);
        } else {
            signature_nonces.set(clone_signer_2, last_nonce + 1);
        }

        doc_signings.get(payload.token_id).unwrap().set(clone_signer_3, payload.status);
        e.storage().instance().set(&DOCSIGN, &doc_signings);
        e.storage().instance().bump(34560);
    }

    pub fn testVerifySignature(e: Env, pubKey: BytesN<32>, message: Bytes, signature: BytesN<64> ) -> u32 {
        e.crypto().ed25519_verify(&pubKey, &message, &signature);
        log!(&e, "Passed this point. Message Verified!");
        3
    }

    pub fn safe_mint(e: Env, erc721_address: Address, to: Address, token_id: u32, meta_uri: String, signers: Vec<Address>, document_hash: String, deadline: u64) -> u32 {
        // IMPLEMENT THIS LIKE IN SOLIDITY PETAL DOCUMENTS CONTRACT
        //		require(
		// 	msg.value >= creationFee || owner() == msg.sender,
		// 	'Creation fee not met'
		// );

        if signers.is_empty() {
            panic!("Must have some signers for each document");
        }
        log!(&e, "Hello");
        // let client = erc721::Client::new(&e, &erc721_address);
        // client.mint(&token_id, &to);
        // client.set_token_uri(&token_id, &meta_uri);

        Self::mint(&e, token_id, to);
        Self::set_token_uri(&e, token_id, meta_uri);

        let mut token_to_doc_hashes: Map<u32, String> = e.storage().instance().get(&T2DHASH).unwrap_or(Map::new(&e));
        token_to_doc_hashes.set(token_id, document_hash);

        let mut doc_signing_deadlines: Map<u32, u64> = e.storage().instance().get(&DEADLINES).unwrap_or(Map::new(&e));
        doc_signing_deadlines.set(token_id, deadline);
        
        let mut doc_signings: Map<u32, Map<Address, SignatureStatus>> = e.storage().instance().get(&DOCSIGN).unwrap_or(Map::new(&e));
        let mut inner_doc_signings: Map<Address, SignatureStatus> = Map::new(&e);

        for signer in signers.iter() {
            inner_doc_signings.set(signer, SignatureStatus::Waiting);
        };
        doc_signings.set(token_id, inner_doc_signings);

        e.storage().instance().set(&T2DHASH, &token_to_doc_hashes);
        e.storage().instance().set(&DEADLINES, &doc_signing_deadlines);
        e.storage().instance().set(&DOCSIGN, &doc_signings);

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);
        token_id
    }

    fn mint(e: &Env, token_id: u32, to: Address) {
        // New Token id should be incremented by 1 and not injected as param.

        let mut owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        log!(&e, "Owners {}", owners);

        if exists(&e, token_id, &owners) == true {
            panic!("Token already minted!");
        }
        log!(&e, "Token does not exists {}", token_id);

        let cloned_to = to.clone();

        owners.set(token_id, to);
        log!(&e, "Owners set locally {}", owners);

        e.storage().instance().set(&OWNERS, &owners);
        log!(&e, "Owners set instance {}", owners);

        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);
        event::mint(&e, &cloned_to, token_id);
    }

    fn set_token_uri(e: &Env, token_id: u32, token_uri: String) {
        let owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));

        if exists(&e, token_id, &owners) == false {
            panic!("ERC721URIStorage: URI set of nonexistent token");
        }

        let mut token_uris: Map<u32, String> = e.storage().instance().get(&URIS).unwrap_or(Map::new(&e));
        token_uris.set(token_id, token_uri);

        e.storage().instance().set(&URIS, &token_uris);
        e.storage().instance().bump(INSTANCE_BUMP_AMOUNT);
    }


    fn require_minted(e: &Env, token_id: u32) -> bool {
        let owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        if exists(&e, token_id, &owners) == true {
            return true
        } 
        return false
    }

    pub fn get_admin(e: Env) -> Address {
        let admin = read_administrator(&e);
        admin
    }

    pub fn get_nonces(e: Env) -> Map<Address, u32> {
        let nonces: Map<Address, u32> = e.storage().instance().get(&NONCES).unwrap_or(Map::new(&e));
        nonces
    }

    pub fn get_owners(e: Env) -> Map<u32, Address> {
        let owners: Map<u32, Address> = e.storage().instance().get(&OWNERS).unwrap_or(Map::new(&e));
        owners
    }

    pub fn get_token_uris(e: Env) -> Map<u32, String> {
        let token_uris: Map<u32, String> = e.storage().instance().get(&URIS).unwrap_or(Map::new(&e));
        token_uris
    }

    pub fn get_td_hashes(e: Env) -> Map<u32, String> {
        let token_to_doc_hashes: Map<u32, String> = e.storage().instance().get(&T2DHASH).unwrap_or(Map::new(&e));
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

// ------------> CURRENT CONTRACT ID = CDOLH5KFTEZ5ZYR3LZSPYBCSW23BJNBYMSLKF6EFP5SEDTSAPLNFYRTD --------------------

// pubKey = [121, 149, 162, 22, 5, 154, 125, 213, 68, 40, 244, 187, 139, 11, 229, 176, 124, 56, 209, 54, 247, 190, 119, 249, 142, 36, 6, 202, 12, 216, 10, 241]
// message = [ 0, 0, 0, 0, 121, 149, 162, 22, 5, 154, 125, 213, 68, 40, 244, 187, 139, 11, 229, 176, 124, 56, 209, 54, 247, 190, 119, 249, 142, 36, 6, 202, 12, 216, 10, 241, 0, 0, 1, 44, 0, 8, 174, 96, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100, 209, 236, 50, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 6, 115, 105, 103, 110, 101, 114, 0, 0, 0, 0, 1, 0, 0, 0, 4, 116, 101, 115, 116, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 6, 115, 116, 97, 116, 117, 115, 0, 0, 0, 0, 0, 1, 0, 0, 0, 10, 78, 79, 84, 95, 83, 73, 71, 78, 69, 68, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 7, 116, 111, 107, 101, 110, 73, 100, 0, 0, 0, 0, 1, 0, 0, 0, 13, 116, 101, 115, 116, 95, 116, 111, 107, 101, 110, 95, 105, 100, 0, 0, 0, 0, 0, 0 ]
// signature = [90, 90, 116, 207, 13, 25, 179, 44, 67, 30, 59, 254, 214, 220, 29, 226, 185, 166, 56, 135, 62, 172, 147, 223, 95, 185, 51, 27, 130, 204, 190, 197, 247, 241, 241, 194, 180, 84, 245, 109, 21, 33, 202, 178, 238, 82, 186, 192, 93, 245, 39, 30, 84, 139, 191, 111, 184, 101, 84, 85, 85, 106, 120, 3]



    // soroban contract deploy \
    // --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
    // --source juico \
    // --network standalone

    // // // testVerifySignature(e: Env, pubKey: BytesN<32>, message: Bytes, signature: BytesN<64> )
    // soroban contract invoke \
    // --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
    // --id CB44HZ226IGVVFYOQ5R7S2THMO6ILL6MRIMZ672WOBC4MJSQAF34ZUR4 \
    //     --source juico \
    //     --network standalone \
    //     -- \
    //     testVerifySignature \
    //     --pubKey '[121, 149, 162, 22, 5, 154, 125, 213, 68, 40, 244, 187, 139, 11, 229, 176, 124, 56, 209, 54, 247, 190, 119, 249, 142, 36, 6, 202, 12, 216, 10, 241]' \
    //     --message '[225, 83, 85, 106, 167, 56, 28, 124, 125, 166, 175, 228, 172, 185, 61, 39, 46, 28, 72, 250, 76, 97, 114, 23, 156, 255, 187, 236, 168, 107, 40, 48]' \
    //     --signature '[147, 104, 161, 187, 15, 179, 242, 150, 34, 137, 132, 69, 234, 168, 223, 66, 77, 52, 0, 142, 229, 144, 130, 28, 199, 114, 196, 191, 234, 224, 152, 142, 200, 133, 194, 45, 221, 164, 247, 57, 108, 73, 210, 7, 128, 92, 228, 155, 22, 64, 211, 101, 28, 199, 174, 194, 224, 106, 250, 144, 26, 160, 58, 4]'

//     soroban contract deploy \
//     --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
//     --source juico \
//     --network standalone

//     soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
// --id CDCUDM6YVA5MJFUTWVZLRXAL5CR35RYGUYBXKCAWWILV3B2B3P6AZYLR \
//     --source juico \
//     --network standalone \
//     -- \
//     get_token_uris 


// soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
// --id CDCUDM6YVA5MJFUTWVZLRXAL5CR35RYGUYBXKCAWWILV3B2B3P6AZYLR \
//     --source juico \
//     --network standalone \
//     -- \
//     safe_mint \
//     --erc721_address CCO42P3BKGQTZGVHWTK5EUS4R7OR7RHUUPH3PHQJKXLGJZP34INQJMIT \
//     --to GDOB4GMX45VENP4YMUQMH4ZJ6KJZTERQVOASFTXC7OMOZM5EFKPFU4X5 \
//     --token_id 1235 \
//     --meta_uri "www.help.com" \
//     --signers '["GBRVKHUULGOAU2ADSZZKFH2DZBZF2S4PXVEMSE23PPTFZDST464RDHIM"]' \
//     --deadline 1679905807890\
//     --document_hash "testinghash"

// soroban contract invoke \
// --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
// --id CDCUDM6YVA5MJFUTWVZLRXAL5CR35RYGUYBXKCAWWILV3B2B3P6AZYLR \
//     --source juico \
//     --network standalone \
//     -- \
//     sign_document \
//     --erc721_address CCO42P3BKGQTZGVHWTK5EUS4R7OR7RHUUPH3PHQJKXLGJZP34INQJMIT \
//     --user GDOB4GMX45VENP4YMUQMH4ZJ6KJZTERQVOASFTXC7OMOZM5EFKPFU4X5 \
//     --signature 1235 \
//     --payload \
//     -- \
//         --deadline: 1679905807890 \
//         --description: "description yo" \
//         --document_hash: "testinghash" \
//         --document_uri: "www.help.com" \
//         --signer: GBRVKHUULGOAU2ADSZZKFH2DZBZF2S4PXVEMSE23PPTFZDST464RDHIM \
//         --status: 'Signed' \
//         --token_id: 1235 \
//         --nonce: 1,

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
//     --wasm target/wasm32-unknown-unknown/release/petal_documents.wasm \
//     --id 0 \
//     -- \
//     safe_mint \
// --erc721_address CCO42P3BKGQTZGVHWTK5EUS4R7OR7RHUUPH3PHQJKXLGJZP34INQJMIT \
// --to GDOB4GMX45VENP4YMUQMH4ZJ6KJZTERQVOASFTXC7OMOZM5EFKPFU4X5 \
// --token_id 1234 \
// --meta_uri "www.help.com" \
// --signers '["GBRVKHUULGOAU2ADSZZKFH2DZBZF2S4PXVEMSE23PPTFZDST464RDHIM"]' \
// --deadline 1679905807890\
// --document_hash "testinghash"

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