/// Integration test to verify rusqlite uses our SQLite3MultipleCiphers bundle
/// This test is in the tests/ directory to ensure proper linking

use rusqlite::{Connection, Result};

#[test]
fn test_sqlite_version() {
    let conn = Connection::open_in_memory().unwrap();
    let version: String = conn
        .query_row("SELECT sqlite_version()", [], |row| row.get(0))
        .unwrap();

    println!("SQLite version: {}", version);

    // Our bundle is SQLite 3.51.3
    assert!(
        version.starts_with("3.51"),
        "Expected SQLite 3.51.x from our bundle, got {}",
        version
    );
}

#[test]
fn test_sqlite3mc_version_function() {
    let conn = Connection::open_in_memory().unwrap();

    // Try to call sqlite3mc_version() - this only exists in SQLite3MultipleCiphers
    match conn.query_row::<String, _, _>("SELECT sqlite3mc_version()", [], |row| row.get(0)) {
        Ok(mc_version) => {
            println!("SQLite3MultipleCiphers version: {}", mc_version);
            assert!(
                mc_version.contains("SQLite3 Multiple Ciphers"),
                "Expected SQLite3MultipleCiphers version string"
            );
        }
        Err(e) => {
            panic!(
                "sqlite3mc_version() function not found - rusqlite is NOT using our bundle! Error: {:?}",
                e
            );
        }
    }
}

#[test]
fn test_basic_encryption() {
    use std::fs;

    let _ = fs::remove_file("test_integration.db");

    let conn = Connection::open("test_integration.db").unwrap();

    // Set encryption key
    conn.pragma_update(None, "key", "test-password").unwrap();

    // Create table
    conn.execute(
        "CREATE TABLE test (id INTEGER PRIMARY KEY, data TEXT)",
        [],
    )
    .unwrap();

    // Insert data
    conn.execute("INSERT INTO test (data) VALUES (?1)", ["encrypted data"])
        .unwrap();

    // Read it back
    let data: String = conn
        .query_row("SELECT data FROM test WHERE id = 1", [], |row| row.get(0))
        .unwrap();

    assert_eq!(data, "encrypted data");

    drop(conn);

    // Now try to open without key - should fail
    let conn2 = Connection::open("test_integration.db").unwrap();

    let result = conn2.query_row::<i64, _, _>("SELECT COUNT(*) FROM test", [], |row| row.get(0));

    assert!(
        result.is_err(),
        "Should not be able to read encrypted database without key"
    );

    // Clean up
    let _ = fs::remove_file("test_integration.db");
}

#[test]
fn test_cipher_selection() {
    use std::fs;

    let _ = fs::remove_file("test_aes.db");

    let conn = Connection::open("test_aes.db").unwrap();

    // Set cipher to AES256
    conn.pragma_update(None, "cipher", "aes256cbc").unwrap();
    conn.pragma_update(None, "key", "aes-password").unwrap();

    conn.execute("CREATE TABLE test (value TEXT)", []).unwrap();
    conn.execute("INSERT INTO test VALUES (?1)", ["AES test"])
        .unwrap();

    let value: String = conn
        .query_row("SELECT value FROM test", [], |row| row.get(0))
        .unwrap();

    assert_eq!(value, "AES test");

    let _ = fs::remove_file("test_aes.db");
}
