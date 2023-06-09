use data_encoding::BASE64;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};
use hex;
use std::collections::HashMap;
use ureq::serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthCodeResponse {
    auth_code: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthTokenResponse {
    access_token: String,
}

fn sign_message(auth_code: &str, user_public_key: &str, user_private_key: &str) -> String {
    // get bytes of the auth code for signing
    let bytes_message = auth_code.as_bytes();

    // decode public key for signing
    let public_key_bytes = BASE64.decode(user_public_key.as_bytes()).unwrap();
    let public_key = PublicKey::from_bytes(&public_key_bytes).expect("Invalid public key");

    // decode private key for signing
    let private_key_bytes = BASE64.decode(user_private_key.as_bytes()).unwrap();
    let private_key = SecretKey::from_bytes(&private_key_bytes).expect("Invalid private key");

    // create signing key
    let signingkey = Keypair {
        secret: private_key,
        public: public_key,
    };

    // finally, sign the auth code with our private key
    let signed_message = signingkey.sign(bytes_message);

    hex::encode(&signed_message.to_bytes()[..64])
}

pub fn get_acces_token(configsettings: &HashMap<String, String>) -> Result<String, ureq::Error> {
    let private_key = configsettings.get("SXT_PRIVATE_KEY").unwrap();
    let public_key = configsettings.get("SXT_PUBLIC_KEY").unwrap();
    let user_id = configsettings.get("SXT_USER_ID").unwrap();

    let AuthCodeResponse { auth_code } = ureq::post(
        format!(
            "{}/v1/auth/code",
            configsettings.get("SXT_API_URL").unwrap()
        )
        .as_str(),
    )
    .send_json(ureq::json!({ "userId": user_id }))?
    .into_json()?;

    let signature = sign_message(&auth_code, public_key, private_key);

    let AuthTokenResponse { access_token } = ureq::post(
        format!(
            "{}/v1/auth/token",
            configsettings.get("SXT_API_URL").unwrap()
        )
        .as_str(),
    )
    .send_json(ureq::json!({
        "userId": user_id,
        "authCode": auth_code,
        "signature": signature,
        "key": public_key
    }))?
    .into_json()?;

    Ok(access_token)
}
