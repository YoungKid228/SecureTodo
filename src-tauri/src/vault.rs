use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use chacha20poly1305::{aead::{Aead, KeyInit}, XChaCha20Poly1305, XNonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use zeroize::Zeroizing;

const VAULT_VERSION: u32 = 1;
/// Argon2id params for the encryption-key derivation (19 MiB, 2 passes).
const ENC_M_COST: u32 = 19 * 1024;
const ENC_T_COST: u32 = 2;
const ENC_P_COST: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub done: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VaultStatus {
    pub initialized: bool,
    pub unlocked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct VaultFile {
    version: u32,
    /// Argon2id PHC hash used ONLY for password verification (comparison of hashes).
    password_hash: String,
    /// Base64 salt for the independent encryption-key derivation.
    enc_salt: String,
    /// Base64 XChaCha20 nonce (24 bytes).
    nonce: String,
    /// Base64 ciphertext of the JSON todo list.
    ciphertext: String,
}

struct Session {
    key: Zeroizing<[u8; 32]>,
}

static SESSION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();

fn session_mutex() -> &'static Mutex<Option<Session>> {
    SESSION.get_or_init(|| Mutex::new(None))
}

fn vault_file_path() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir().ok_or_else(|| "Не найден каталог данных".to_string())?;
    Ok(base.join("secure-todo").join("vault.json"))
}

fn write_vault_file(vf: &VaultFile) -> Result<(), String> {
    let path = vault_file_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Не могу создать каталог: {e}"))?;
    }
    let json = serde_json::to_string_pretty(vf).map_err(|e| format!("Сериализация: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Запись vault: {e}"))?;
    Ok(())
}

fn read_vault_file() -> Result<VaultFile, String> {
    let path = vault_file_path()?;
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("Чтение vault: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Повреждённый vault: {e}"))
}

/// Hash password for verification (PHC string with random salt inside).
fn hash_password_for_verify(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("Хэширование пароля: {e}"))
}

fn verify_password(hash_str: &str, password: &str) -> Result<(), String> {
    let parsed = PasswordHash::new(hash_str).map_err(|_| "Повреждённый хэш пароля".to_string())?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| "Неверный пароль".to_string())
}

/// Derive the 32-byte encryption key: Argon2id(password, enc_salt).
fn derive_enc_key(password: &str, enc_salt_b64: &str) -> Result<Zeroizing<[u8; 32]>, String> {
    let salt = B64
        .decode(enc_salt_b64)
        .map_err(|_| "Повреждённая соль шифрования".to_string())?;
    let params = Params::new(ENC_M_COST, ENC_T_COST, ENC_P_COST, Some(32))
        .map_err(|e| format!("Параметры Argon2: {e}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(password.as_bytes(), &salt, &mut key[..])
        .map_err(|e| format!("Вывод ключа: {e}"))?;
    Ok(key)
}

fn new_enc_salt_b64() -> String {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    B64.encode(salt)
}

fn encrypt_todos(key: &[u8; 32], todos: &[TodoItem]) -> Result<(String, String), String> {
    let plaintext =
        serde_json::to_vec(todos).map_err(|e| format!("Сериализация задач: {e}"))?;
    let cipher = XChaCha20Poly1305::new(key.into());
    let mut nonce_bytes = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Шифрование: {e}"))?;
    Ok((B64.encode(nonce_bytes), B64.encode(ct)))
}

fn decrypt_todos(key: &[u8; 32], nonce_b64: &str, ct_b64: &str) -> Result<Vec<TodoItem>, String> {
    let nonce_bytes = B64
        .decode(nonce_b64)
        .map_err(|_| "Повреждённый nonce".to_string())?;
    let ct = B64
        .decode(ct_b64)
        .map_err(|_| "Повреждённый шифротекст".to_string())?;
    if nonce_bytes.len() != 24 {
        return Err("Повреждённый nonce".to_string());
    }
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XNonce::from_slice(&nonce_bytes);
    let pt = cipher
        .decrypt(nonce, ct.as_ref())
        .map_err(|_| "Не удалось расшифровать (неверный ключ?)".to_string())?;
    serde_json::from_slice(&pt).map_err(|_| "Расшифрованные данные повреждены".to_string())
}

fn lock_session() {
    if let Ok(mut guard) = session_mutex().lock() {
        *guard = None; // Zeroizing key wipes on drop
    }
}

fn store_session(key: Zeroizing<[u8; 32]>) {
    if let Ok(mut guard) = session_mutex().lock() {
        *guard = Some(Session { key });
    }
}

fn session_key() -> Result<Zeroizing<[u8; 32]>, String> {
    session_mutex()
        .lock()
        .map_err(|_| "Внутренняя ошибка блокировки".to_string())?
        .as_ref()
        .map(|s| s.key.clone())
        .ok_or_else(|| "Хранилище заблокировано".to_string())
}

fn check_password_strength(password: &str) -> Result<(), String> {
    if password.chars().count() < 4 {
        return Err("Пароль должен содержать минимум 4 символа".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn vault_status() -> Result<VaultStatus, String> {
    let initialized = vault_file_path().map(|p| p.exists()).unwrap_or(false);
    let unlocked = session_mutex()
        .lock()
        .map(|g| g.is_some())
        .unwrap_or(false);
    Ok(VaultStatus { initialized, unlocked })
}

#[tauri::command]
pub fn vault_is_unlocked() -> bool {
    session_mutex().lock().map(|g| g.is_some()).unwrap_or(false)
}

#[tauri::command]
pub fn vault_init(password: String) -> Result<VaultStatus, String> {
    check_password_strength(&password)?;
    if vault_file_path().map(|p| p.exists()).unwrap_or(false) {
        return Err("Хранилище уже создано".to_string());
    }
    // 1. Hash for verification (hash comparison on unlock).
    let password_hash = hash_password_for_verify(&password)?;
    // 2. Independent salt + key for encryption.
    let enc_salt = new_enc_salt_b64();
    let key = derive_enc_key(&password, &enc_salt)?;
    // 3. Encrypt empty list — until now data exists only as ciphertext.
    let empty: Vec<TodoItem> = vec![];
    let (nonce, ciphertext) = encrypt_todos(&key, &empty)?;
    write_vault_file(&VaultFile {
        version: VAULT_VERSION,
        password_hash,
        enc_salt,
        nonce,
        ciphertext,
    })?;
    store_session(key);
    Ok(VaultStatus { initialized: true, unlocked: true })
}

#[tauri::command]
pub fn vault_unlock(password: String) -> Result<VaultStatus, String> {
    let vf = read_vault_file()?;
    if vf.version != VAULT_VERSION {
        return Err("Неподдерживаемая версия хранилища".to_string());
    }
    // 1. Validate password by hash comparison FIRST.
    verify_password(&vf.password_hash, &password)?;
    // 2. Only after successful verification derive key and decrypt.
    let key = derive_enc_key(&password, &vf.enc_salt)?;
    let _todos = decrypt_todos(&key, &vf.nonce, &vf.ciphertext)?;
    store_session(key);
    Ok(VaultStatus { initialized: true, unlocked: true })
}

#[tauri::command]
pub fn vault_lock() -> Result<VaultStatus, String> {
    lock_session();
    let initialized = vault_file_path().map(|p| p.exists()).unwrap_or(false);
    Ok(VaultStatus { initialized, unlocked: false })
}

#[tauri::command]
pub fn todos_list() -> Result<Vec<TodoItem>, String> {
    let key = session_key()?;
    let vf = read_vault_file()?;
    decrypt_todos(&key, &vf.nonce, &vf.ciphertext)
}

#[tauri::command]
pub fn todos_save(todos: Vec<TodoItem>) -> Result<(), String> {
    let key = session_key()?;
    let mut vf = read_vault_file()?;
    let (nonce, ciphertext) = encrypt_todos(&key, &todos)?;
    vf.nonce = nonce;
    vf.ciphertext = ciphertext;
    write_vault_file(&vf)
}

#[tauri::command]
pub fn vault_change_password(old_password: String, new_password: String) -> Result<VaultStatus, String> {
    check_password_strength(&new_password)?;
    let mut vf = read_vault_file()?;
    // Verify old password by hash comparison.
    verify_password(&vf.password_hash, &old_password)?;
    let old_key = derive_enc_key(&old_password, &vf.enc_salt)?;
    let todos = decrypt_todos(&old_key, &vf.nonce, &vf.ciphertext)?;
    // Re-encrypt everything under the new password.
    let new_hash = hash_password_for_verify(&new_password)?;
    let new_salt = new_enc_salt_b64();
    let new_key = derive_enc_key(&new_password, &new_salt)?;
    let (nonce, ciphertext) = encrypt_todos(&new_key, &todos)?;
    vf.password_hash = new_hash;
    vf.enc_salt = new_salt;
    vf.nonce = nonce;
    vf.ciphertext = ciphertext;
    write_vault_file(&vf)?;
    store_session(new_key);
    Ok(VaultStatus { initialized: true, unlocked: true })
}
