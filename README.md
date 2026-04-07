# libsqlite3-hotbundle

A fork of `libsqlite3-sys` that bundles **SQLite3MultipleCiphers** - providing SQLite 3.51.3 with full encryption support.

## Features

- **🔐 Multiple Encryption Ciphers**: AES-128, AES-256, ChaCha20 (default), SQLCipher, RC4, ASCON128, AEGIS
- **📦 Hotbundle Integration**: Works seamlessly with rusqlite's non-bundled mode
- **⚡ Full SQLite**: All standard extensions (FTS3/4/5, JSON1, RTREE, GEOPOLY, etc.)
- **🔧 Zero Config**: Encryption "just works" - no OpenSSL dependencies

**Version**: 1.510300.0 (SQLite 3.51.3 + SQLite3MultipleCiphers 2.3.2)

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# Use rusqlite WITHOUT bundled SQLite (critical!)
rusqlite = { version = "0.38", default-features = false }

# Add the hotbundle - provides SQLite3MultipleCiphers
libsqlite3-hotbundle = "1.510300"
```

### Basic Usage

```rust
use rusqlite::Connection;

// Regular unencrypted database
let conn = Connection::open("app.db")?;
conn.execute("CREATE TABLE users (id INTEGER, name TEXT)", [])?;

// ✅ Works exactly like standard SQLite
```

### With Encryption

```rust
use rusqlite::Connection;

// Create encrypted database
let conn = Connection::open("secure.db")?;

// Set encryption key (MUST be done before any operations)
conn.pragma_update(None, "key", "my-secret-password")?;

// Now use database normally - it's encrypted!
conn.execute("CREATE TABLE secrets (data TEXT)", [])?;
conn.execute("INSERT INTO secrets VALUES (?)", ["sensitive"])?;
```

### Choose Cipher

```rust
// Specify cipher before setting key
conn.pragma_update(None, "cipher", "aes256cbc")?;
conn.pragma_update(None, "key", "password")?;

// Available ciphers:
// - "chacha20"   - ChaCha20 (default, fast)
// - "aes128cbc"  - AES-128
// - "aes256cbc"  - AES-256
// - "sqlcipher"  - SQLCipher compatibility
// - "ascon128"   - ASCON lightweight
// - "aegis"      - AEGIS high-performance
// - "rc4"        - RC4 (legacy)
```

## Verification

### Check It's Working

```rust
use rusqlite::Connection;

let conn = Connection::open_in_memory()?;

// 1. Check SQLite version
let version: String = conn.query_row(
    "SELECT sqlite_version()", [], |row| row.get(0)
)?;
assert_eq!(version, "3.51.3"); // ✅

// 2. Check SQLite3MultipleCiphers is active
let mc_version: String = conn.query_row(
    "SELECT sqlite3mc_version()", [], |row| row.get(0)
)?;
println!("✅ Using: {}", mc_version);
// Output: ✅ Using: SQLite3 Multiple Ciphers 2.3.2
```

### Check Binary Linking

```bash
# Build your app
cargo build

# Verify no system SQLite dependency
ldd target/debug/your-app | grep sqlite
# Should output nothing (using bundled SQLite ✅)

# Check for SQLite3MC symbols
nm target/debug/your-app | grep sqlite3mc_version
# Should show: sqlite3mc_version symbol ✅
```

## Building from Source

### Project Structure

```
libsqlite3-hotbundle/
├── sqlite3/              # SQLite3MultipleCiphers C source
│   ├── sqlite3mc.c       # Main amalgamation
│   ├── aegis/            # AEGIS cipher
│   ├── argon2/           # Argon2 key derivation
│   └── ascon/            # ASCON cipher
├── src/
│   └── lib.rs            # Rust bindings (minimal)
├── build.rs              # Compilation logic
└── upgrade.sh            # Version upgrade script
```

### Upgrade SQLite

```bash
# Upgrade to specific SQLite3MultipleCiphers version
./upgrade.sh v2.3.2

# Or upgrade to latest
./upgrade.sh latest
```

This will:
1. Clone SQLite3MultipleCiphers repository
2. Checkout specified tag
3. Copy source files to `sqlite3/`
4. Update `Cargo.toml` version
5. Create git commit and tag

## Testing

```bash
# Run all tests
cargo test

# Run with example
cargo run --example test_encryption

# Test specific features
cargo test --features cipher-aes256
```

**All tests should pass:**
```
test test_bundled_version ... ok
test test_sqlite3mc_version_function ... ok
test test_sqlite_version ... ok
test test_basic_encryption ... ok
test test_cipher_selection ... ok
```

## How It Works

The "hotbundle" compiles SQLite3MultipleCiphers as `libsqlite3.a` (static library). When rusqlite is in non-bundled mode, the linker:

1. Looks for SQLite library to link
2. Finds our `libsqlite3.a` static library
3. Prefers static over dynamic (system's `libsqlite3.so`)
4. ✅ Links our encrypted SQLite instead!

**Critical:** Library must be named `sqlite3` (not `sqlite3mc`) for this to work.

## Advanced Configuration

### Default Cipher Selection

Choose default cipher at compile time:

```toml
[dependencies]
libsqlite3-hotbundle = { version = "1.510300", features = ["cipher-aes256"] }
```

**Available features:**
- `cipher-aes128` - AES-128 as default
- `cipher-aes256` - AES-256 as default  
- `cipher-sqlcipher` - SQLCipher as default
- `cipher-ascon` - ASCON128 as default
- `cipher-aegis` - AEGIS as default
- (no feature = ChaCha20 default)

### Additional Features

```toml
[dependencies]
libsqlite3-hotbundle = { 
    version = "1.510300", 
    features = [
        "unlock_notify",    # Enable unlock notify
        "preupdate_hook",   # Enable preupdate hooks
        "session",          # Enable session extension
    ]
}
```


## Security Notes

### Encryption Strength

- All ciphers use cryptographically secure key derivation (Argon2)
- Random numbers from OS-provided sources (`/dev/urandom`)
- Same security as SQLCipher and similar encryption solutions

### Best Practices

```rust
// ✅ Set key BEFORE any operations
conn.pragma_update(None, "key", "password")?;
conn.execute("CREATE TABLE ...", [])?;

// ❌ Don't do operations before setting key
conn.execute("CREATE TABLE ...", [])?;  // Database not encrypted!
conn.pragma_update(None, "key", "password")?;  // Too late
```

## Troubleshooting

### Still using system SQLite?

```toml
# Make sure default-features = false
rusqlite = { version = "0.38", default-features = false }
```

### sqlite3mc_version() not found?

1. Check `libsqlite3-hotbundle` is in dependencies
2. Run `cargo clean` and rebuild
3. Verify with `nm target/debug/your-app | grep sqlite3mc`

### Encryption fails?

1. Verify SQLite3MC is active: `SELECT sqlite3mc_version()`
2. Set key BEFORE any database operations
3. Check cipher name spelling: `conn.pragma_update(None, "cipher", "aes256cbc")`

## Performance

### Native Builds
- **ChaCha20**: Very fast on all platforms
- **AES-256**: Extremely fast with hardware AES (Intel/ARM)
- **AEGIS**: Highest performance for authenticated encryption
- **Overhead**: < 5% when encryption disabled

## License

This project maintains dual MIT/Apache-2.0 licensing. SQLite3MultipleCiphers is licensed under the MIT license.

## Credits

- **SQLite** - Public domain database engine
- **SQLite3MultipleCiphers** by Ulrich Telle - MIT license
- **libsqlite3-hotbundle** by Kent Ross - Original hotbundle concept
- **rusqlite** - Rust SQLite bindings

## Links

- [SQLite3MultipleCiphers](https://github.com/utelle/SQLite3MultipleCiphers)
- [rusqlite](https://github.com/rusqlite/rusqlite)
- [Original hotbundle](https://github.com/mumbleskates/libsqlite3-hotbundle)

---

**Status**: Production Ready ✅ | **Platform**: Linux, macOS, Windows | **Encryption**: Fully Working ✅
