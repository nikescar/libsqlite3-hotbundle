// Re-export SQLite C API functions for WASM/cdylib usage
// This prevents the linker from stripping the C code in release builds
#[allow(non_camel_case_types, dead_code)]
mod ffi {
    use std::os::raw::{c_char, c_int, c_void};

    extern "C" {
        pub fn sqlite3_libversion() -> *const c_char;
        pub fn sqlite3_open(filename: *const c_char, ppDb: *mut *mut c_void) -> c_int;
        pub fn sqlite3_close(pDb: *mut c_void) -> c_int;
        pub fn sqlite3_exec(
            pDb: *mut c_void,
            sql: *const c_char,
            callback: *mut c_void,
            arg: *mut c_void,
            errmsg: *mut *mut c_char,
        ) -> c_int;
    }
}

// Export key SQLite functions with #[no_mangle] to prevent dead code elimination
// These will be available to JavaScript when using the WASM module
#[no_mangle]
pub extern "C" fn sqlite3_libversion() -> *const std::os::raw::c_char {
    unsafe { ffi::sqlite3_libversion() }
}

#[no_mangle]
pub extern "C" fn sqlite3_open(
    filename: *const std::os::raw::c_char,
    ppDb: *mut *mut std::os::raw::c_void,
) -> std::os::raw::c_int {
    unsafe { ffi::sqlite3_open(filename, ppDb) }
}

#[no_mangle]
pub extern "C" fn sqlite3_close(pDb: *mut std::os::raw::c_void) -> std::os::raw::c_int {
    unsafe { ffi::sqlite3_close(pDb) }
}

#[no_mangle]
pub extern "C" fn sqlite3_exec(
    pDb: *mut std::os::raw::c_void,
    sql: *const std::os::raw::c_char,
    callback: *mut std::os::raw::c_void,
    arg: *mut std::os::raw::c_void,
    errmsg: *mut *mut std::os::raw::c_char,
) -> std::os::raw::c_int {
    unsafe { ffi::sqlite3_exec(pDb, sql, callback, arg, errmsg) }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_bundled_version() {
        assert_eq!(effective_sqlite_version(), Ok(bundled_sqlite_version()));
    }

    fn effective_sqlite_version() -> rusqlite::Result<String> {
        Ok(rusqlite::Connection::open_in_memory()?.query_row(
            "SELECT sqlite_version();",
            [],
            |row| Ok(row.get(0)?),
        )?)
    }

    fn bundled_sqlite_version() -> String {
        let header_text = include_str!("../sqlite3/sqlite3.h");
        let mut expected_version = None;
        for line in header_text.lines() {
            let words: Vec<&str> = line.trim().split_ascii_whitespace().collect();
            match words.as_slice() {
                ["#define", "SQLITE_VERSION", version_str] => {
                    let Some(version_str) = version_str
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'))
                    else {
                        panic!("couldn't unwrap SQLITE_VERSION #define value {version_str:?}");
                    };
                    expected_version = Some(version_str.to_owned());
                    break;
                }
                _ => {}
            }
        }
        expected_version.expect("couldn't find SQLITE_VERSION in the header")
    }
}
