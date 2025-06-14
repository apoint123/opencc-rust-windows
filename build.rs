use std::{collections::HashSet, env, path::PathBuf};

const MIN_VERSION: &str = "1.1.2";
const MAX_VERSION: &str = "1.2.0";

fn main() {
    let target = env::var("TARGET").unwrap();
    if target == "x86_64-pc-windows-msvc" {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let manifest_dir = PathBuf::from(manifest_dir);

        let lib_path = manifest_dir.join("vendor").join("windows-x64").join("lib");

        if !lib_path.exists() {
            panic!(
                "Vendored library path does not exist: {}. Please check the crate's file structure.",
                lib_path.display()
            );
        }
        if !lib_path.is_dir() {
            panic!(
                "Vendored library path is not a directory: {}.",
                lib_path.display()
            );
        }

        println!("cargo:rustc-link-search=native={}", lib_path.display());

        println!("cargo:rustc-link-lib=static=opencc");
        println!("cargo:rustc-link-lib=static=marisa");
        println!("cargo:rustc-link-lib=static=darts");

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=vendor/windows-x64/lib/opencc.lib");

        return;
    }

    println!("cargo:warning=Target is not x86_64-pc-windows-msvc, falling back to other methods.");

    if cfg!(target_env = "msvc") {
        if let Ok(lib) = vcpkg::find_package("opencc") {
            println!(
                "cargo:warning=Found OpenCC via vcpkg, forcing static linking for MSVC target."
            );

            for path in &lib.link_paths {
                println!("cargo:rustc-link-search=native={}", path.display());
            }

            println!("cargo:rustc-link-lib=static=opencc");
            println!("cargo:rustc-link-lib=static=marisa");
            println!("cargo:rustc-link-lib=static=darts");

            return;
        }

        println!("cargo:warning=vcpkg did not find OpenCC. Falling back to other methods.");
    }

    if env::var("DOCS_RS").is_ok() {
        return;
    }

    if cfg!(target_os = "freebsd") {
        env_var_set_default("OPENCC_INCLUDE_DIRS", "/usr/include/opencc");
        env_var_set_default("OPENCC_LIB_DIRS", "/usr/lib");
        env_var_set_default("OPENCC_LIBS", "opencc");
    }

    let lib_dirs = find_opencc_lib_dirs();
    for d in &lib_dirs {
        if !d.exists() {
            panic!(
                "OpenCC library directory does not exist: {}",
                d.to_string_lossy()
            );
        }
        println!("cargo:rustc-link-search=native={}", d.to_string_lossy());
    }

    let include_dirs = find_opencc_include_dirs();
    for d in &include_dirs {
        if !d.exists() {
            panic!(
                "OpenCC include directory does not exist: {}",
                d.to_string_lossy()
            );
        }
        println!("cargo:include={}", d.to_string_lossy());
    }
    println!("cargo:rerun-if-env-changed=OPENCC_LIBS");

    let libs_env = env::var("OPENCC_LIBS").ok();

    let libs = match libs_env {
        Some(ref v) => v.split(':').map(|x| x.to_owned()).collect(),
        None => {
            if target.contains("windows") {
                vec![
                    "opencc".to_string(),
                    "marisa".to_string(),
                    "darts".to_string(),
                ]
            } else if target.contains("freebsd") {
                vec!["opencc".to_string()]
            } else {
                run_pkg_config().libs
            }
        }
    };

    let kind = determine_mode(&lib_dirs, libs.as_slice());
    for lib in libs.into_iter() {
        println!("cargo:rustc-link-lib={}={}", kind, lib);
    }

    println!("cargo:rerun-if-env-changed=OPENCC_DYLIB_STDCPP");
    if let Ok(kind) = env::var("OPENCC_DYLIB_STDCPP") {
        if kind != "0" {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
    }

    println!("cargo:rerun-if-env-changed=OPENCC_STATIC_STDCPP");
    if let Ok(kind) = env::var("OPENCC_STATIC_STDCPP") {
        if kind != "0" {
            println!("cargo:rustc-link-lib=static=stdc++");
        }
    }
}

fn env_var_set_default(name: &str, value: &str) {
    if env::var(name).is_err() {
        unsafe { env::set_var(name, value) };
    }
}

fn find_opencc_lib_dirs() -> Vec<PathBuf> {
    println!("cargo:rerun-if-env-changed=OPENCC_LIB_DIRS");

    let sep = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };

    env::var("OPENCC_LIB_DIRS")
        .map(|x| x.split(sep).map(PathBuf::from).collect::<Vec<PathBuf>>())
        .or_else(|_| Ok(vec![find_opencc_dir()?.join("lib")]))
        .or_else(|_: env::VarError| -> Result<_, env::VarError> { Ok(run_pkg_config().link_paths) })
        .expect("Couldn't find OpenCC library directory")
}

fn find_opencc_include_dirs() -> Vec<PathBuf> {
    println!("cargo:rerun-if-env-changed=OPENCC_INCLUDE_DIRS");

    let sep = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };

    env::var("OPENCC_INCLUDE_DIRS")
        .map(|x| x.split(sep).map(PathBuf::from).collect::<Vec<PathBuf>>())
        .or_else(|_| Ok(vec![find_opencc_dir()?.join("include")]))
        .or_else(|_: env::VarError| -> Result<_, env::VarError> {
            Ok(run_pkg_config().include_paths)
        })
        .expect("Couldn't find OpenCC include directory")
}

fn find_opencc_dir() -> Result<PathBuf, env::VarError> {
    println!("cargo:rerun-if-env-changed=OPENCC_DIR");
    env::var("OPENCC_DIR").map(PathBuf::from)
}

fn determine_mode<T: AsRef<str>>(libdirs: &[PathBuf], libs: &[T]) -> &'static str {
    println!("cargo:rerun-if-env-changed=OPENCC_STATIC");
    let kind = env::var("OPENCC_STATIC").ok();
    match kind.as_ref().map(|s| &s[..]) {
        Some("0") => return "dylib",
        Some(_) => return "static",
        None => {}
    }

    let files = libdirs
        .iter()
        .flat_map(|d| {
            d.read_dir().unwrap_or_else(|e| {
                panic!("Failed to read library directory '{}': {}", d.display(), e)
            })
        })
        .map(|e| e.unwrap_or_else(|err| panic!("Failed to read directory entry: {}", err)))
        .map(|e| e.file_name())
        .filter_map(|e| e.into_string().ok())
        .collect::<HashSet<_>>();

    let can_static = libs.iter().all(|l| {
        files.contains(&format!("lib{}.a", l.as_ref()))
            || files.contains(&format!("{}.lib", l.as_ref()))
    });
    let can_dylib = libs.iter().all(|l| {
        files.contains(&format!("lib{}.so", l.as_ref()))
            || files.contains(&format!("{}.dll", l.as_ref()))
            || files.contains(&format!("lib{}.dylib", l.as_ref()))
    });

    match (can_static, can_dylib) {
        (true, false) => return "static",
        (false, true) => return "dylib",
        (false, false) => {
            panic!(
                "OpenCC libdirs at `{:?}` do not contain the required files to either statically \
                 or dynamically link OpenCC",
                libdirs
            );
        }
        (true, true) => {}
    }

    "dylib"
}

fn run_pkg_config() -> pkg_config::Library {
    pkg_config::Config::new()
        .cargo_metadata(false)
        .range_version(MIN_VERSION..MAX_VERSION)
        .probe("opencc")
        .map_err(|e| {
            let version_err = format!(
                "OpenCC version must be >= {} and < {}",
                MIN_VERSION, MAX_VERSION
            );
            panic!("pkg-config failed to find OpenCC: {}. {}", e, version_err);
        })
        .unwrap()
}
