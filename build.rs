use std::env;

/// Tells whether we're building for Windows. This is more suitable than a plain
/// `cfg!(windows)`, since the latter does not properly handle cross-compilation
///
/// Note that there is no way to know at compile-time which system we'll be
/// targeting, and this test must be made at run-time (of the build script) See
/// <https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts>
fn win_target() -> bool {
    env::var("CARGO_CFG_WINDOWS").is_ok()
}

/// Tells whether we're building for Android.
/// See [`win_target`]
fn android_target() -> bool {
    env::var("CARGO_CFG_TARGET_OS").is_ok_and(|v| v == "android")
}

/// Tells whether a given compiler will be used `compiler_name` is compared to
/// the content of `CARGO_CFG_TARGET_ENV` (and is always lowercase)
///
/// See [`win_target`]
fn is_compiler(compiler_name: &str) -> bool {
    env::var("CARGO_CFG_TARGET_ENV").is_ok_and(|v| v == compiler_name)
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let lib_name = "sqlite3"; // Must be "sqlite3" for hotbundle to work

    let src_dir = "sqlite3";

    println!("cargo:include={}/{src_dir}", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rerun-if-changed={src_dir}/sqlite3mc.c");
    println!("cargo:rerun-if-changed={src_dir}/sqlite3patched.c");
    println!("cargo:rerun-if-changed={src_dir}/wasm32-wasi-vfs.c");

    let mut cfg = cc::Build::new();
    cfg.file(format!("{src_dir}/sqlite3mc.c"))
        .include(format!("{src_dir}"))
        .include(format!("{src_dir}/aegis/include"))
        .include(format!("{src_dir}/aegis/common"))
        .include(format!("{src_dir}/ascon"))
        .include(format!("{src_dir}/argon2/include"))
        .include(format!("{src_dir}/argon2/src"))
        .flag("-DSQLITE_CORE")
        .flag("-DSQLITE_DEFAULT_FOREIGN_KEYS=1")
        .flag("-DSQLITE_ENABLE_API_ARMOR")
        .flag("-DSQLITE_ENABLE_COLUMN_METADATA")
        .flag("-DSQLITE_ENABLE_DBSTAT_VTAB")
        .flag("-DSQLITE_ENABLE_FTS3")
        .flag("-DSQLITE_ENABLE_FTS3_PARENTHESIS")
        .flag("-DSQLITE_ENABLE_FTS4")
        .flag("-DSQLITE_ENABLE_FTS5")
        .flag("-DSQLITE_ENABLE_JSON1")
        .flag("-DSQLITE_ENABLE_LOAD_EXTENSION=1")
        .flag("-DSQLITE_ENABLE_MEMORY_MANAGEMENT")
        .flag("-DSQLITE_ENABLE_RTREE")
        .flag("-DSQLITE_ENABLE_GEOPOLY")
        .flag("-DSQLITE_ENABLE_STAT4")
        .flag("-DSQLITE_SOUNDEX")
        .flag("-DSQLITE_THREADSAFE=1")
        .flag("-DSQLITE_USE_URI")
        .flag("-DSQLITE_TEMP_STORE=2")
        .flag("-DHAVE_USLEEP=1")
        .flag("-DHAVE_ISNAN")
        .flag("-D_POSIX_THREAD_SAFE_FUNCTIONS") // cross compile with MinGW
        .warnings(false);

    // SQLite3MultipleCiphers: Select default cipher based on features
    let default_cipher = if cfg!(feature = "cipher-aes128") {
        "CODEC_TYPE_AES128"
    } else if cfg!(feature = "cipher-aes256") {
        "CODEC_TYPE_AES256"
    } else if cfg!(feature = "cipher-sqlcipher") {
        "CODEC_TYPE_SQLCIPHER"
    } else if cfg!(feature = "cipher-ascon") {
        "CODEC_TYPE_ASCON128"
    } else if cfg!(feature = "cipher-aegis") {
        "CODEC_TYPE_AEGIS"
    } else {
        // Default to ChaCha20
        "CODEC_TYPE_CHACHA20"
    };

    cfg.flag(format!("-DCODEC_TYPE={default_cipher}"))
        .flag("-DHAVE_CIPHER_AES_128_CBC=1")
        .flag("-DHAVE_CIPHER_AES_256_CBC=1")
        .flag("-DHAVE_CIPHER_CHACHA20=1")
        .flag("-DHAVE_CIPHER_SQLCIPHER=1")
        .flag("-DHAVE_CIPHER_RC4=1")
        .flag("-DHAVE_CIPHER_ASCON128=1")
        .flag("-DHAVE_CIPHER_AEGIS=1");

    if cfg!(feature = "double-quoted-string-literals") {
        cfg.flag("-DSQLITE_DQS=1");
    } else {
        cfg.flag("-DSQLITE_DQS=0");
    }

    // on android sqlite can't figure out where to put the temp files.
    // the bundled sqlite on android also uses `SQLITE_TEMP_STORE=3`.
    // https://android.googlesource.com/platform/external/sqlite/+/2c8c9ae3b7e6f340a19a0001c2a889a211c9d8b2/dist/Android.mk
    // For SQLite3MultipleCiphers, we also disable AES hardware support on Android
    if android_target() {
        cfg.flag("-DSQLITE_TEMP_STORE=3");
        cfg.flag("-DSQLITE3MC_OMIT_AES_HARDWARE_SUPPORT=1");
    }

    if cfg!(feature = "with-asan") {
        cfg.flag("-fsanitize=address");
    }

    // If explicitly requested: enable static linking against the Microsoft Visual
    // C++ Runtime to avoid dependencies on vcruntime140.dll and similar libraries.
    if cfg!(target_feature = "crt-static") && is_compiler("msvc") {
        cfg.static_crt(true);
    }

    if !win_target() {
        cfg.flag("-DHAVE_LOCALTIME_R");
    }
    if env::var("TARGET").is_ok_and(|v| v.starts_with("wasm32-wasi")) {
        cfg.flag("-USQLITE_THREADSAFE")
            .flag("-DSQLITE_THREADSAFE=0")
            // https://github.com/rust-lang/rust/issues/74393
            .flag("-DLONGDOUBLE_TYPE=double")
            .flag("-D_WASI_EMULATED_MMAN")
            .flag("-D_WASI_EMULATED_GETPID")
            .flag("-D_WASI_EMULATED_SIGNAL")
            .flag("-D_WASI_EMULATED_PROCESS_CLOCKS")
            // Enable emulated pthread for argon2 (encryption key derivation)
            .flag("-D_WASI_EMULATED_PTHREAD")
            // Tell SQLite3MultipleCiphers we're on WASM
            .flag("-D__WASM__")
            // Include our pthread compatibility header
            .flag("-include")
            .flag("sqlite3/wasi-pthread-compat.h");

        if cfg!(feature = "wasm32-wasi-vfs") {
            cfg.file("sqlite3/wasm32-wasi-vfs.c");
        }

        // Add WASI helpers (provides getentropy() and pthread_exit())
        cfg.file("sqlite3/wasi-helpers.c");

        // Note: wasi-emulated-pthread not needed for cdylib builds
        // Our wasi-helpers.c provides the necessary pthread stubs
    }
    if cfg!(feature = "unlock_notify") {
        cfg.flag("-DSQLITE_ENABLE_UNLOCK_NOTIFY");
    }
    if cfg!(feature = "preupdate_hook") {
        cfg.flag("-DSQLITE_ENABLE_PREUPDATE_HOOK");
    }
    if cfg!(feature = "session") {
        cfg.flag("-DSQLITE_ENABLE_SESSION");
        cfg.flag("-DSQLITE_ENABLE_PREUPDATE_HOOK");
    }

    if let Ok(limit) = env::var("SQLITE_MAX_VARIABLE_NUMBER") {
        cfg.flag(format!("-DSQLITE_MAX_VARIABLE_NUMBER={limit}"));
    }
    println!("cargo:rerun-if-env-changed=SQLITE_MAX_VARIABLE_NUMBER");

    if let Ok(limit) = env::var("SQLITE_MAX_EXPR_DEPTH") {
        cfg.flag(format!("-DSQLITE_MAX_EXPR_DEPTH={limit}"));
    }
    println!("cargo:rerun-if-env-changed=SQLITE_MAX_EXPR_DEPTH");

    if let Ok(limit) = env::var("SQLITE_MAX_COLUMN") {
        cfg.flag(format!("-DSQLITE_MAX_COLUMN={limit}"));
    }
    println!("cargo:rerun-if-env-changed=SQLITE_MAX_COLUMN");

    if let Ok(extras) = env::var("LIBSQLITE3_FLAGS") {
        for extra in extras.split_whitespace() {
            if extra.starts_with("-D") || extra.starts_with("-U") {
                cfg.flag(extra);
            } else if extra.starts_with("SQLITE_") {
                cfg.flag(format!("-D{extra}"));
            } else {
                panic!("Don't understand {extra} in LIBSQLITE3_FLAGS");
            }
        }
    }
    println!("cargo:rerun-if-env-changed=LIBSQLITE3_FLAGS");

    cfg.compile(lib_name);

    println!("cargo:lib_dir={out_dir}");

    // Provide metadata about SQLite3MultipleCiphers
    println!("cargo:rustc-cfg=sqlite3mc");

    // For WASM cdylib builds: prevent linker from stripping C code in release mode
    if env::var("TARGET").is_ok_and(|v| v.starts_with("wasm32-wasi")) {
        // Tell the linker to export all symbols from the static library
        println!("cargo:rustc-link-arg=--export-all");
        println!("cargo:rustc-link-arg=--no-gc-sections");
    }
}
