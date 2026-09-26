// Secure Todo vault backend.
//
// Security model:
// - Password is NEVER stored. Only an Argon2id PHC hash is stored for verification.
// - Todo data is stored only as XChaCha20-Poly1305 ciphertext.
// - Auth key and encryption key are derived independently:
//     * auth: Argon2id(password, random auth salt embedded in PHC string)
//     * encryption: Argon2id(password, enc_salt) -> 32-byte key
// - Unlock flow: verify password against stored hash FIRST, only then derive
//   the encryption key and decrypt.
// - The encryption key lives only in process memory (zeroized on lock).

mod vault;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            vault::vault_status,
            vault::vault_init,
            vault::vault_unlock,
            vault::vault_lock,
            vault::vault_is_unlocked,
            vault::todos_list,
            vault::todos_save,
            vault::vault_change_password,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
