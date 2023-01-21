use std::path::PathBuf;
use sui_faucet::FaucetResponse;
use sui_keys::keystore::AccountKeystore;
use sui_sdk::types::{crypto::SignatureScheme, base_types::SuiAddress};
use sui_keys::keystore::FileBasedKeystore;
use std::collections::HashMap;
use fastcrypto::encoding::{Encoding, Hex};
#[tokio::main]
async fn main() {

    let keystore_path = get_keystore_path();
    let keystore = FileBasedKeystore::new(&keystore_path).unwrap();
    let (address, phrase, _) = create_new_keypair(keystore);
    
    let _ = request_sui_coins(address).await;

    println!("{} {} ", address, phrase);

}

fn get_keystore_path() -> PathBuf {
    match dirs::home_dir() {
        Some(v) => v.join(".sui").join("sui_config").join("sui.keystore"),
        None => panic!("Cannot obtain home directory path"),
    }
}

fn create_new_keypair(mut keystore: FileBasedKeystore) -> (SuiAddress, String, SignatureScheme)  {
    keystore.generate_and_add_new_key(
        SignatureScheme::ED25519, 
        None
    ).unwrap()
}

async fn request_sui_coins(request_address: SuiAddress) -> FaucetResponse {
    let remote_url = "https://faucet.devnet.sui.io";
    let gas_url = format!("{}/gas", remote_url);
    let data = HashMap::from([("recipient", Hex::encode(request_address))]);
    let map = HashMap::from([("FixedAmountRequest", data)]);

    let response = reqwest::Client::new()
        .post(&gas_url)
        .json(&map)
        .send()
        .await
        .unwrap_or_else(|e| panic!("Failed to talk to remote faucet {:?}: {:?}", gas_url, e));
    
    let full_bytes = response.bytes().await.unwrap();
    let faucet_response: FaucetResponse = serde_json::from_slice(&full_bytes)
        .map_err(|e| anyhow::anyhow!("json deser failed with bytes {:?}: {e}", full_bytes))
        .unwrap();

    if let Some(error) = faucet_response.error {
        panic!("Failed to get gas tokens with error: {}", error)
    };

    faucet_response
}