# Diesel + libsqlite3-hotbundle Demo

This example demonstrates how to use [Diesel](https://diesel.rs/) ORM with `libsqlite3-hotbundle` to ensure a consistent, bundled SQLite version across all platforms.

## Why Use libsqlite3-hotbundle with Diesel?

- **Version Consistency**: Guarantees the same SQLite version on all platforms (development, CI, production)
- **Feature Control**: Use specific SQLite features (like SQLite3MultipleCiphers encryption) regardless of system SQLite
- **Reproducibility**: Eliminates "works on my machine" issues caused by different system SQLite versions
- **No System Dependencies**: No need to install SQLite separately

## How It Works

1. **Diesel** provides the ORM layer and uses `libsqlite3-sys` for FFI bindings
2. **libsqlite3-sys** is configured with `default-features = false` to disable its bundled SQLite
3. **libsqlite3-hotbundle** compiles and links the specific SQLite C source, providing the symbols that `libsqlite3-sys` expects
4. The `extern crate libsqlite3_hotbundle;` line ensures Cargo doesn't optimize out the hotbundle dependency

## Configuration

### Cargo.toml

```toml
[dependencies]
diesel = { version = "2.2.0", features = ["sqlite", "r2d2"] }
libsqlite3-sys = { version = "0.30.0", default-features = false }
libsqlite3-hotbundle = { version = "1.530400", default-features = false }
```

### Force Linkage in Code

```rust
// CRITICAL: This forces Cargo to include the hotbundle
extern crate libsqlite3_hotbundle;
```

## Running the Example

```bash
# Run the demo
cargo run --manifest-path examples/diesel_demo/Cargo.toml

# Run tests
cargo test --manifest-path examples/diesel_demo/Cargo.toml

# Run with encryption features (if available)
cargo run --manifest-path examples/diesel_demo/Cargo.toml --features cipher-sqlcipher
cargo test --manifest-path examples/diesel_demo/Cargo.toml --features cipher-sqlcipher
```

## Expected Output

```
=== Diesel + libsqlite3-hotbundle Demo ===

📋 Demo 1: Version Check
  SQLite Version: 3.51.3
  ✅ SQLite3 Multiple Ciphers 2.3.2

📝 Demo 2: Basic Unencrypted Database
  Users in database:
    1 - Alice (alice@example.com)

🔐 Demo 3: Encrypted Database (ChaCha20 - default cipher)
  ✅ Encryption enabled with ChaCha20
  ✅ Inserted and queried encrypted data
  Records in encrypted table: 1

🔑 Demo 4: Encrypted Database (AES-256-CBC)
  ✅ Encryption enabled with AES-256-CBC
  ✅ Retrieved encrypted data: 'top secret data'

✅ All demos completed successfully!
```

## Verification

To verify you're using the bundled SQLite (not your system's version):

1. Check the printed version - it should match the hotbundle version (3.51+)
2. Run on different systems - the version should be identical
3. Compare with system SQLite: `sqlite3 --version` vs the printed version

## Features

This example demonstrates:

- **Connection**: Establishing SQLite connection via Diesel
- **Schema**: Table creation and management
- **CRUD Operations**:
  - Create (INSERT)
  - Read (SELECT with filters)
  - Update (UPDATE)
  - Delete (DELETE)
- **Type Safety**: Using Diesel's type-safe query builder
- **Encryption**: Full database encryption with multiple ciphers
  - ChaCha20 (default, fast)
  - AES-256-CBC (hardware accelerated)
  - SQLCipher compatibility mode
- **Testing**: Comprehensive test suite with AAA pattern

## Encryption Usage

### Basic Encryption (ChaCha20 - default)

```rust
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

let mut connection = SqliteConnection::establish("secure.db")?;

// CRITICAL: Set key BEFORE any operations
diesel::sql_query("PRAGMA key = 'my-secret-password'")
    .execute(&mut connection)?;

// Now use database normally - it's encrypted!
diesel::sql_query("CREATE TABLE secrets (data TEXT)")
    .execute(&mut connection)?;
```

### Choose Different Cipher

```rust
// Set cipher BEFORE key
diesel::sql_query("PRAGMA cipher = 'aes256cbc'")
    .execute(&mut connection)?;

diesel::sql_query("PRAGMA key = 'password'")
    .execute(&mut connection)?;
```

### Available Ciphers

- **chacha20** - ChaCha20 (default, very fast)
- **aes128cbc** - AES-128-CBC
- **aes256cbc** - AES-256-CBC (hardware accelerated on modern CPUs)
- **sqlcipher** - SQLCipher compatibility mode
- **ascon128** - ASCON lightweight cipher
- **aegis** - AEGIS high-performance authenticated encryption
- **rc4** - RC4 (legacy, not recommended)

### Important Notes

⚠️ **Always set encryption key BEFORE any database operations**

```rust
// ✅ CORRECT
diesel::sql_query("PRAGMA key = 'password'").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE ...").execute(&mut conn)?;

// ❌ WRONG - Database not encrypted!
diesel::sql_query("CREATE TABLE ...").execute(&mut conn)?;
diesel::sql_query("PRAGMA key = 'password'").execute(&mut conn)?;
```

## Tests

The test suite covers:

- ✅ Version verification (confirms hotbundle is used)
- ✅ SQLite3MultipleCiphers version check
- ✅ Basic insert and query
- ✅ Multiple records
- ✅ Filtering with WHERE clauses
- ✅ Updates
- ✅ Deletes
- ✅ Encrypted database with ChaCha20
- ✅ Encrypted database with AES-256
- ✅ SQLCipher compatibility mode
- ✅ Encryption enforcement (can't read without key)

Run with: `cargo test --manifest-path examples/diesel_demo/Cargo.toml`

## Troubleshooting

### "undefined reference to sqlite3_*" errors

- Ensure `extern crate libsqlite3_hotbundle;` is present
- Verify `libsqlite3-sys` has `default-features = false`
- Run `cargo clean` and rebuild

### Wrong SQLite version printed

- Check that hotbundle appears in `cargo tree`
- Verify no other SQLite dependency is bundling its own version
- Ensure the linkage line is not commented out

## License

This example is provided as-is for demonstration purposes.
