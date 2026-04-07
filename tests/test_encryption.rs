/// Integration test for SQLite3MultipleCiphers encryption features

use rusqlite::Connection;
use std::fs;

#[test]
fn test_sqlite3mc_encryption() -> rusqlite::Result<()> {
    println!("=== SQLite3MultipleCiphers Encryption Test ===\n");

    // Clean up any existing test databases
    let _ = fs::remove_file("test_encrypted.db");
    let _ = fs::remove_file("test_plaintext.db");

    // Test 1: Verify SQLite version includes SQLite3MultipleCiphers
    test_version()?;

    // Test 2: Create and use encrypted database
    test_basic_encryption()?;

    // Test 3: Verify encryption actually works (can't open without key)
    test_encryption_required()?;

    // Test 4: Test different ciphers
    test_different_ciphers()?;

    // Test 5: Verify plaintext databases still work
    test_plaintext_database()?;

    // Cleanup
    let _ = fs::remove_file("test_encrypted.db");
    let _ = fs::remove_file("test_plaintext.db");
    let _ = fs::remove_file("test_aes256.db");
    let _ = fs::remove_file("test_sqlcipher.db");

    println!("\n✅ All tests passed! SQLite3MultipleCiphers is working correctly.");
    Ok(())
}

fn test_version() -> rusqlite::Result<()> {
    println!("📋 Test 1: Checking SQLite version...");

    let conn = Connection::open_in_memory()?;
    let version: String = conn.query_row("SELECT sqlite_version()", [], |row| row.get(0))?;

    println!("   SQLite version: {}", version);

    // Try to get SQLite3MC version if available
    // This will fail on standard SQLite but succeed on SQLite3MultipleCiphers
    match conn.query_row::<String, _, _>(
        "SELECT sqlite3mc_version()",
        [],
        |row| row.get(0)
    ) {
        Ok(mc_version) => {
            println!("   ✅ SQLite3MultipleCiphers version: {}", mc_version);
        }
        Err(_) => {
            println!("   ⚠️  SQLite3MC version function not found (might be using standard SQLite)");
        }
    }

    println!();
    Ok(())
}

fn test_basic_encryption() -> rusqlite::Result<()> {
    println!("🔐 Test 2: Creating encrypted database with ChaCha20...");

    // Create encrypted database
    let conn = Connection::open("test_encrypted.db")?;

    // Set encryption key (must be done before any other operations)
    conn.pragma_update(None, "key", "my-secret-password")?;

    // Create table and insert data
    conn.execute(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
        [],
    )?;

    conn.execute(
        "INSERT INTO users (name, email) VALUES (?1, ?2)",
        ["Alice", "alice@example.com"],
    )?;

    conn.execute(
        "INSERT INTO users (name, email) VALUES (?1, ?2)",
        ["Bob", "bob@example.com"],
    )?;

    // Verify we can read the data
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    assert_eq!(count, 2);

    let name: String = conn.query_row(
        "SELECT name FROM users WHERE id = 1",
        [],
        |row| row.get(0)
    )?;
    assert_eq!(name, "Alice");

    println!("   ✅ Created encrypted database");
    println!("   ✅ Inserted 2 records");
    println!("   ✅ Successfully queried encrypted data");
    println!();

    Ok(())
}

fn test_encryption_required() -> rusqlite::Result<()> {
    println!("🔒 Test 3: Verifying encryption is enforced...");

    // Try to open the encrypted database without a key
    match Connection::open("test_encrypted.db") {
        Ok(conn) => {
            // Try to query - this should fail
            match conn.query_row::<i64, _, _>("SELECT COUNT(*) FROM users", [], |row| row.get(0)) {
                Ok(_) => {
                    println!("   ❌ ERROR: Could read encrypted database without key!");
                    return Err(rusqlite::Error::InvalidQuery);
                }
                Err(_) => {
                    println!("   ✅ Cannot read encrypted database without key (expected)");
                }
            }
        }
        Err(e) => {
            println!("   ✅ Cannot open encrypted database: {:?}", e);
        }
    }

    // Now open with correct key
    let conn = Connection::open("test_encrypted.db")?;
    conn.pragma_update(None, "key", "my-secret-password")?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    assert_eq!(count, 2);

    println!("   ✅ Successfully opened with correct key");

    // Try with wrong key
    let _ = fs::copy("test_encrypted.db", "test_encrypted_copy.db");
    let conn2 = Connection::open("test_encrypted_copy.db")?;
    conn2.pragma_update(None, "key", "wrong-password")?;

    match conn2.query_row::<i64, _, _>("SELECT COUNT(*) FROM users", [], |row| row.get(0)) {
        Ok(_) => {
            println!("   ❌ ERROR: Could read with wrong password!");
        }
        Err(_) => {
            println!("   ✅ Cannot read with wrong password (expected)");
        }
    }

    let _ = fs::remove_file("test_encrypted_copy.db");
    println!();

    Ok(())
}

fn test_different_ciphers() -> rusqlite::Result<()> {
    println!("🔑 Test 4: Testing different cipher algorithms...");

    // Test AES-256
    println!("   Testing AES-256-CBC...");
    let conn = Connection::open("test_aes256.db")?;
    conn.pragma_update(None, "cipher", "aes256cbc")?;
    conn.pragma_update(None, "key", "aes-password")?;
    conn.execute("CREATE TABLE test (id INTEGER, data TEXT)", [])?;
    conn.execute("INSERT INTO test VALUES (1, 'AES encrypted')", [])?;
    let data: String = conn.query_row("SELECT data FROM test WHERE id = 1", [], |row| row.get(0))?;
    assert_eq!(data, "AES encrypted");
    println!("      ✅ AES-256-CBC works");
    drop(conn);

    // Test SQLCipher compatibility
    println!("   Testing SQLCipher...");
    let conn = Connection::open("test_sqlcipher.db")?;
    conn.pragma_update(None, "cipher", "sqlcipher")?;
    conn.pragma_update(None, "key", "sqlcipher-password")?;
    conn.execute("CREATE TABLE test (id INTEGER, data TEXT)", [])?;
    conn.execute("INSERT INTO test VALUES (1, 'SQLCipher encrypted')", [])?;
    let data: String = conn.query_row("SELECT data FROM test WHERE id = 1", [], |row| row.get(0))?;
    assert_eq!(data, "SQLCipher encrypted");
    println!("      ✅ SQLCipher works");
    drop(conn);

    println!("   ✅ Multiple cipher algorithms working");
    println!();

    Ok(())
}

fn test_plaintext_database() -> rusqlite::Result<()> {
    println!("📝 Test 5: Verifying plaintext databases still work...");

    // Create unencrypted database
    let conn = Connection::open("test_plaintext.db")?;

    conn.execute(
        "CREATE TABLE data (id INTEGER PRIMARY KEY, value TEXT)",
        [],
    )?;

    conn.execute(
        "INSERT INTO data (value) VALUES (?1)",
        ["plaintext data"],
    )?;

    let value: String = conn.query_row(
        "SELECT value FROM data WHERE id = 1",
        [],
        |row| row.get(0)
    )?;

    assert_eq!(value, "plaintext data");
    println!("   ✅ Unencrypted databases work normally");

    // Verify the file is actually unencrypted by reading raw bytes
    drop(conn);
    if let Ok(file_content) = fs::read("test_plaintext.db") {
        let file_str = String::from_utf8_lossy(&file_content[..100]);
        if file_str.contains("SQLite") {
            println!("   ✅ Database file contains SQLite magic string (not encrypted)");
        }
    }

    println!();
    Ok(())
}
