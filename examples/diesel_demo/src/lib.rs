use diesel::prelude::*;

extern crate libsqlite3_hotbundle;

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

#[derive(QueryableByName, Debug, PartialEq)]
pub struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> SqliteConnection {
        let mut connection =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        diesel::sql_query(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL
            )",
        )
        .execute(&mut connection)
        .expect("Failed to create table");

        connection
    }

    #[test]
    fn test_sqlite_version_from_hotbundle() {
        // Arrange
        let mut connection =
            SqliteConnection::establish(":memory:").expect("Failed to create connection");

        // Act
        let version: String = diesel::select(diesel::dsl::sql::<diesel::sql_types::Text>(
            "sqlite_version()",
        ))
        .get_result(&mut connection)
        .expect("Failed to fetch version");

        // Assert - version should be from hotbundle (3.51+ series)
        assert!(
            version.starts_with("3.5"),
            "Expected SQLite 3.5x series from hotbundle, got: {}",
            version
        );
    }

    #[test]
    fn test_insert_and_query_user() {
        // Arrange
        let mut connection = setup_test_db();
        let new_user = NewUser {
            name: "Bob",
            email: "bob@example.com",
        };

        // Act
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&mut connection)
            .expect("Failed to insert user");

        let results = users::table
            .select(User::as_select())
            .load(&mut connection)
            .expect("Failed to load users");

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Bob");
        assert_eq!(results[0].email, "bob@example.com");
    }

    #[test]
    fn test_multiple_users() {
        // Arrange
        let mut connection = setup_test_db();
        let users_data = vec![
            NewUser {
                name: "Alice",
                email: "alice@example.com",
            },
            NewUser {
                name: "Bob",
                email: "bob@example.com",
            },
            NewUser {
                name: "Charlie",
                email: "charlie@example.com",
            },
        ];

        // Act
        for user in users_data {
            diesel::insert_into(users::table)
                .values(&user)
                .execute(&mut connection)
                .expect("Failed to insert user");
        }

        let results = users::table
            .select(User::as_select())
            .load(&mut connection)
            .expect("Failed to load users");

        // Assert
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].name, "Alice");
        assert_eq!(results[1].name, "Bob");
        assert_eq!(results[2].name, "Charlie");
    }

    #[test]
    fn test_filter_users() {
        // Arrange
        let mut connection = setup_test_db();
        let users_data = vec![
            NewUser {
                name: "Alice",
                email: "alice@example.com",
            },
            NewUser {
                name: "Bob",
                email: "bob@test.com",
            },
        ];

        for user in users_data {
            diesel::insert_into(users::table)
                .values(&user)
                .execute(&mut connection)
                .expect("Failed to insert user");
        }

        // Act
        let results = users::table
            .filter(users::email.like("%example.com"))
            .select(User::as_select())
            .load(&mut connection)
            .expect("Failed to load users");

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alice");
    }

    #[test]
    fn test_update_user() {
        // Arrange
        let mut connection = setup_test_db();
        let new_user = NewUser {
            name: "Alice",
            email: "alice@example.com",
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&mut connection)
            .expect("Failed to insert user");

        // Act
        diesel::update(users::table.filter(users::name.eq("Alice")))
            .set(users::email.eq("alice.updated@example.com"))
            .execute(&mut connection)
            .expect("Failed to update user");

        let results = users::table
            .select(User::as_select())
            .load(&mut connection)
            .expect("Failed to load users");

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].email, "alice.updated@example.com");
    }

    #[test]
    fn test_delete_user() {
        // Arrange
        let mut connection = setup_test_db();
        let users_data = vec![
            NewUser {
                name: "Alice",
                email: "alice@example.com",
            },
            NewUser {
                name: "Bob",
                email: "bob@example.com",
            },
        ];

        for user in users_data {
            diesel::insert_into(users::table)
                .values(&user)
                .execute(&mut connection)
                .expect("Failed to insert user");
        }

        // Act
        diesel::delete(users::table.filter(users::name.eq("Alice")))
            .execute(&mut connection)
            .expect("Failed to delete user");

        let results = users::table
            .select(User::as_select())
            .load(&mut connection)
            .expect("Failed to load users");

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Bob");
    }

    #[test]
    fn test_sqlite3mc_version() {
        // Arrange
        let mut connection =
            SqliteConnection::establish(":memory:").expect("Failed to create connection");

        // Act
        let version: Result<String, _> = diesel::select(diesel::dsl::sql::<
            diesel::sql_types::Text,
        >(
            "sqlite3mc_version()"
        ))
        .get_result(&mut connection);

        // Assert
        assert!(version.is_ok(), "sqlite3mc_version() function should exist");
        let version_str = version.unwrap();
        assert!(
            version_str.contains("SQLite3 Multiple Ciphers"),
            "Expected SQLite3MultipleCiphers version string, got: {}",
            version_str
        );
    }

    #[test]
    fn test_encrypted_database_chacha20() {
        use tempfile::NamedTempFile;

        // Arrange
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let db_path = temp_file.path().to_str().unwrap();

        // Create encrypted database with ChaCha20 (default)
        {
            let mut connection =
                SqliteConnection::establish(db_path).expect("Failed to create connection");

            // Set encryption key BEFORE any operations
            diesel::sql_query("PRAGMA key = 'my-secret-password'")
                .execute(&mut connection)
                .expect("Failed to set encryption key");

            // Create table and insert data
            diesel::sql_query(
                "CREATE TABLE users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    email TEXT NOT NULL
                )",
            )
            .execute(&mut connection)
            .expect("Failed to create table");

            diesel::insert_into(users::table)
                .values(&NewUser {
                    name: "Alice",
                    email: "alice@example.com",
                })
                .execute(&mut connection)
                .expect("Failed to insert user");

            // Verify we can read the data
            let result: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM users")
                .get_result(&mut connection)
                .expect("Failed to query encrypted database");

            assert_eq!(result.count, 1);
        }

        // Act - try to open without key
        {
            let connection = SqliteConnection::establish(db_path);
            if let Ok(mut conn) = connection {
                let result: Result<CountResult, _> =
                    diesel::sql_query("SELECT COUNT(*) as count FROM users").get_result(&mut conn);

                // Assert - should fail without key
                assert!(
                    result.is_err(),
                    "Should not be able to read encrypted database without key"
                );
            }
        }

        // Act - open with correct key
        {
            let mut connection =
                SqliteConnection::establish(db_path).expect("Failed to open database");

            diesel::sql_query("PRAGMA key = 'my-secret-password'")
                .execute(&mut connection)
                .expect("Failed to set key");

            let result: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM users")
                .get_result(&mut connection)
                .expect("Failed to query with correct key");

            // Assert - should succeed with correct key
            assert_eq!(result.count, 1);
        }
    }

    #[test]
    fn test_encrypted_database_aes256() {
        use tempfile::NamedTempFile;

        // Arrange
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let db_path = temp_file.path().to_str().unwrap();

        // Create encrypted database with AES-256
        {
            let mut connection =
                SqliteConnection::establish(db_path).expect("Failed to create connection");

            // Set cipher BEFORE key
            diesel::sql_query("PRAGMA cipher = 'aes256cbc'")
                .execute(&mut connection)
                .expect("Failed to set cipher");

            diesel::sql_query("PRAGMA key = 'aes-password'")
                .execute(&mut connection)
                .expect("Failed to set encryption key");

            // Create table and insert data
            diesel::sql_query(
                "CREATE TABLE users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    email TEXT NOT NULL
                )",
            )
            .execute(&mut connection)
            .expect("Failed to create table");

            diesel::insert_into(users::table)
                .values(&NewUser {
                    name: "Bob",
                    email: "bob@example.com",
                })
                .execute(&mut connection)
                .expect("Failed to insert user");

            // Verify AES-256 encryption works
            let result: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM users")
                .get_result(&mut connection)
                .expect("Failed to query");

            assert_eq!(result.count, 1);
        }

        // Reopen with AES-256
        {
            let mut connection =
                SqliteConnection::establish(db_path).expect("Failed to open database");

            diesel::sql_query("PRAGMA cipher = 'aes256cbc'")
                .execute(&mut connection)
                .expect("Failed to set cipher");

            diesel::sql_query("PRAGMA key = 'aes-password'")
                .execute(&mut connection)
                .expect("Failed to set key");

            let name: String = users::table
                .select(users::name)
                .first(&mut connection)
                .expect("Failed to query user");

            // Assert
            assert_eq!(name, "Bob");
        }
    }

    #[test]
    fn test_encrypted_database_sqlcipher_compat() {
        use tempfile::NamedTempFile;

        // Arrange
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let db_path = temp_file.path().to_str().unwrap();

        // Create database with SQLCipher compatibility mode
        {
            let mut connection =
                SqliteConnection::establish(db_path).expect("Failed to create connection");

            diesel::sql_query("PRAGMA cipher = 'sqlcipher'")
                .execute(&mut connection)
                .expect("Failed to set cipher");

            diesel::sql_query("PRAGMA key = 'sqlcipher-key'")
                .execute(&mut connection)
                .expect("Failed to set encryption key");

            diesel::sql_query("CREATE TABLE secrets (id INTEGER, data TEXT)")
                .execute(&mut connection)
                .expect("Failed to create table");

            diesel::sql_query("INSERT INTO secrets VALUES (1, 'top secret')")
                .execute(&mut connection)
                .expect("Failed to insert");

            let result: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM secrets")
                .get_result(&mut connection)
                .expect("Failed to query");

            // Assert
            assert_eq!(result.count, 1);
        }
    }
}
