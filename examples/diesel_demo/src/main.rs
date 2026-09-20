use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

// CRITICAL: This forces Cargo to include the hotbundle object files during the link phase
extern crate libsqlite3_hotbundle;

// Helper structs for raw SQL queries
#[derive(QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

#[derive(QueryableByName)]
struct SecretResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    data: String,
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
    }
}

#[derive(Queryable, Selectable, Debug, PartialEq)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
}

fn main() {
    println!("=== Diesel + libsqlite3-hotbundle Demo ===\n");

    // Demo 1: Verify SQLite version
    demo_version_check();

    // Demo 2: Basic unencrypted database
    demo_basic_usage();

    // Demo 3: Encrypted database with ChaCha20 (default)
    demo_encryption_chacha20();

    // Demo 4: Encrypted database with AES-256
    demo_encryption_aes256();
}

fn demo_version_check() {
    println!("📋 Demo 1: Version Check");
    let mut connection = SqliteConnection::establish(":memory:")
        .expect("Failed to create in-memory database");

    let version: String = diesel::select(diesel::dsl::sql::<diesel::sql_types::Text>(
        "sqlite_version()",
    ))
    .get_result(&mut connection)
    .expect("Failed to fetch version");

    println!("  SQLite Version: {}", version);

    // Check SQLite3MultipleCiphers
    let mc_version: Result<String, _> = diesel::select(diesel::dsl::sql::<
        diesel::sql_types::Text,
    >("sqlite3mc_version()"))
    .get_result(&mut connection);

    if let Ok(version) = mc_version {
        println!("  ✅ {}", version);
    }
    println!();
}

fn demo_basic_usage() {
    println!("📝 Demo 2: Basic Unencrypted Database");

    let mut connection = SqliteConnection::establish(":memory:")
        .expect("Failed to create in-memory database");

    // Create the users table
    diesel::sql_query(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        )",
    )
    .execute(&mut connection)
    .expect("Failed to create table");

    // Insert a new user
    let new_user = NewUser {
        name: "Alice",
        email: "alice@example.com",
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut connection)
        .expect("Failed to insert user");

    // Query all users
    let results = users::table
        .select(User::as_select())
        .load(&mut connection)
        .expect("Failed to load users");

    println!("  Users in database:");
    for user in results {
        println!("    {} - {} ({})", user.id, user.name, user.email);
    }
    println!();
}

fn demo_encryption_chacha20() {
    use std::fs;

    println!("🔐 Demo 3: Encrypted Database (ChaCha20 - default cipher)");

    // Clean up if exists
    let _ = fs::remove_file("demo_encrypted.db");

    let mut connection = SqliteConnection::establish("demo_encrypted.db")
        .expect("Failed to create database");

    // CRITICAL: Set encryption key BEFORE any operations
    diesel::sql_query("PRAGMA key = 'my-secret-password'")
        .execute(&mut connection)
        .expect("Failed to set encryption key");

    println!("  ✅ Encryption enabled with ChaCha20");

    // Now use database normally - it's encrypted!
    diesel::sql_query(
        "CREATE TABLE secrets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            data TEXT NOT NULL
        )",
    )
    .execute(&mut connection)
    .expect("Failed to create table");

    diesel::sql_query("INSERT INTO secrets (data) VALUES ('sensitive information')")
        .execute(&mut connection)
        .expect("Failed to insert");

    // Query using raw SQL
    let result: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM secrets")
        .get_result(&mut connection)
        .expect("Failed to query");

    println!("  ✅ Inserted and queried encrypted data");
    println!("  Records in encrypted table: {}", result.count);

    // Clean up
    drop(connection);
    let _ = fs::remove_file("demo_encrypted.db");
    println!();
}

fn demo_encryption_aes256() {
    use std::fs;

    println!("🔑 Demo 4: Encrypted Database (AES-256-CBC)");

    // Clean up if exists
    let _ = fs::remove_file("demo_aes256.db");

    let mut connection = SqliteConnection::establish("demo_aes256.db")
        .expect("Failed to create database");

    // Set cipher BEFORE key
    diesel::sql_query("PRAGMA cipher = 'aes256cbc'")
        .execute(&mut connection)
        .expect("Failed to set cipher");

    diesel::sql_query("PRAGMA key = 'aes-password'")
        .execute(&mut connection)
        .expect("Failed to set encryption key");

    println!("  ✅ Encryption enabled with AES-256-CBC");

    diesel::sql_query("CREATE TABLE vault (id INTEGER, secret TEXT)")
        .execute(&mut connection)
        .expect("Failed to create table");

    diesel::sql_query("INSERT INTO vault VALUES (1, 'top secret data')")
        .execute(&mut connection)
        .expect("Failed to insert");

    // Query using raw SQL
    let result: SecretResult = diesel::sql_query("SELECT secret as data FROM vault WHERE id = 1")
        .get_result(&mut connection)
        .expect("Failed to query");

    println!("  ✅ Retrieved encrypted data: '{}'", result.data);

    // Clean up
    drop(connection);
    let _ = fs::remove_file("demo_aes256.db");
    println!();

    println!("✅ All demos completed successfully!");
}
