use cosmwasm_std::{Addr, Env};
use k256::{
    ecdsa::{RecoveryId, Signature, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};
use sha2::{Digest, Sha256};

use crate::error::ContractError;
use crate::msg::Permit;

pub fn verify_permit_signature(
    env: &Env,
    permit: &Permit,
    owner: &Addr,
) -> Result<(), ContractError> {
    // Create the message hash that should have been signed
    let message = create_permit_message(env, permit, owner)?;
    let message_hash = Sha256::digest(message.as_bytes());
    
    // Parse the signature
    let signature_bytes = permit.signature.signature.as_slice();
    if signature_bytes.len() != 65 {
        return Err(ContractError::InvalidSignatureLength {
            length: signature_bytes.len(),
        });
    }
    
    // Extract r, s, and recovery_id
    let r_bytes: [u8; 32] = signature_bytes[0..32]
        .try_into()
        .map_err(|_| ContractError::InvalidSignature {})?;
    let s_bytes: [u8; 32] = signature_bytes[32..64]
        .try_into()
        .map_err(|_| ContractError::InvalidSignature {})?;
    let recovery_id = signature_bytes[64];
    
    // Create signature object
    let signature = Signature::from_scalars(r_bytes, s_bytes)
        .map_err(|_| ContractError::InvalidSignature {})?;
    
    let recovery_id = RecoveryId::try_from(recovery_id)
        .map_err(|_| ContractError::InvalidSignature {})?;
    
    // Recover the public key
    let recovered_key = VerifyingKey::recover_from_prehash(&message_hash, &signature, recovery_id)
        .map_err(|_| ContractError::InvalidSignature {})?;
    
    // Convert recovered public key to address format
    let recovered_pubkey_bytes = recovered_key.to_encoded_point(false).as_bytes().to_vec();
    let recovered_addr = pubkey_to_address(&recovered_pubkey_bytes)?;
    
    // Verify the recovered address matches the owner
    if recovered_addr != *owner {
        return Err(ContractError::InvalidSignature {});
    }
    
    Ok(())
}

fn create_permit_message(env: &Env, permit: &Permit, owner: &Addr) -> Result<String, ContractError> {
    // Create a deterministic message to sign
    // This should match the format used on the client side
    let message = format!(
        "{}:{}:{}:{}:{}:{}:{}",
        env.contract.address,
        owner,
        permit.params.permit_name,
        permit.params.nonce,
        permit.params.allowed_tokens.join(","),
        permit.params.permissions.join(","),
        permit.params.expiration.unwrap_or(0)
    );
    Ok(message)
}

fn pubkey_to_address(pubkey: &[u8]) -> Result<Addr, ContractError> {
    // This is a simplified version - you'll need to implement
    // the proper address derivation for your specific chain
    // For Cosmos chains, this typically involves bech32 encoding
    let hash = Sha256::digest(pubkey);
    let addr_bytes = &hash[12..32]; // Take last 20 bytes like Ethereum
    let addr_str = hex::encode(addr_bytes);
    
    // In practice, you'd use proper bech32 encoding here
    // This is just a placeholder implementation
    Ok(Addr::unchecked(format!("cosmos{}", &addr_str[0..10])))
}
