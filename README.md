# Secure Todo — список дел на Tauri + React

Приложение-список дел, которое открывается по паролю:
- пароль **нигде не хранится**, для проверки хранится только его **Argon2id-хэш** (валидация = сравнение хэшей);
- задачи хранятся **только в зашифрованном виде** (XChaCha20-Poly1305), расшифровка — только после ввода пароля;
- ключ шифрования выводится из пароля отдельным Argon2id-KDF и живёт только в памяти (обнуляется при блокировке).

## Файлы хранилища

`%LOCALAPPDATA%\secure-todo\vault.json`:
```json
{
  "version": 1,
  "password_hash": "<argon2id PHC для проверки>",
  "enc_salt": "<base64 соль для ключа шифрования>",
  "nonce": "<base64 XChaCha nonce>",
  "ciphertext": "<base64 шифротекст JSON списка задач>"
}
```

## Команды бэкенда (Rust, `src-tauri/src/vault.rs`)

- `vault_status` — есть ли хранилище + разблокировано ли
- `vault_init(password)` — первый запуск: хэш для проверки + шифрование пустого списка
- `vault_unlock(password)` — сначала сравнение хэша, только потом вывод ключа и расшифровка
- `vault_lock` — забыть ключ из памяти
- `todos_list` / `todos_save` — расшифровать / зашифровать
- `vault_change_password(old, new)` — проверка старого, перешифровка под новым

## Запуск

Для сборки Rust нужен MSVC (Build Tools) — уже установлен. Запуск через helper,
который подхватывает окружение MSVC:

```bat
tauri-dev.bat dev     :: режим разработки
tauri-dev.bat build   :: production-сборка (msi + nsis setup в src-tauri\target\release\bundle)
```

Или вручную из "x64 Native Tools Command Prompt":

```bat
npm install
npm run tauri dev
npm run tauri build
```
