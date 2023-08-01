use soroban_sdk::{Address, Env, Map};

pub fn owner_of(e: &Env, token_id: u32, owners: Option<Map<u32, Address>>) -> Address {
    match owners {
        Some(v) => {
          v.get(token_id).expect("Address does not exist for given token id").clone()
        },
        None => {
            panic!("Did not find the owner");
        },
      }
}

pub fn exists(e: &Env, token_id: u32, owners: &Map<u32, Address>) -> bool {
    let address = owners.get(token_id);
    match address {
        Some(v) => {
            true
        },
        None => {
            false
        }
    }
}