extern crate cc;
#[cfg(feature = "vendored-openssl")]
extern crate openssl_src;
extern crate pkg_config;
extern crate vcpkg;

mod find_normal;
#[cfg(feature = "vendored-openssl")]
mod find_vendored;

use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=zsign");

    let target = env::var("TARGET").unwrap();

    let (lib_dirs, include_dir) = find_openssl(&target);

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .warnings(false)
        .include("zsign")
        .include("zsign/common")
        .include(&include_dir);

    if cfg!(target_env = "msvc") {
        build.define("ZSIGN_RUST_NO_PRAGMA_LINK", None);
        build.flag_if_supported("/std:c++14");
        build.flag_if_supported("/EHsc");
        build.define("_HAS_STD_BYTE", Some("0"));
        build.define("_CRT_SECURE_NO_WARNINGS", None);
    } else {
        build.std("c++11");
    }

    build
        .file("zsign/zsign.cpp")
        .file("zsign/bundle.cpp")
        .file("zsign/macho.cpp")
        .file("zsign/openssl.cpp")
        .file("zsign/archo.cpp")
        .file("zsign/signing.cpp")
        .file("zsign/common/base64.cpp")
        .file("zsign/common/fs.cpp")
        .file("zsign/common/json.cpp")
        .file("zsign/common/log.cpp")
        .file("zsign/common/sha.cpp")
        .file("zsign/common/timer.cpp")
        .file("zsign/common/util.cpp");

    if cfg!(target_os = "windows") {
        for f in [
            "zsign/common/iconv.cpp",
            "zsign/common/getopt.cpp",
            "zsign/common/common_win32.cpp",
        ] {
            if Path::new(f).exists() {
                build.file(f);
            }
        }
    }

    build.compile("zsign");

    for lib_dir in lib_dirs.iter() {
        println!(
            "cargo:rustc-link-search=native={}",
            lib_dir.to_string_lossy()
        );
    }
    println!("cargo:include={}", include_dir.to_string_lossy());

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}

fn find_openssl(target: &str) -> (Vec<PathBuf>, PathBuf) {
    #[cfg(feature = "vendored-openssl")]
    {
        // vendor if the feature is present, unless
        // OPENSSL_NO_VENDOR exists and isn't `0`
        if env("OPENSSL_NO_VENDOR").map_or(true, |s| s == "0") {
            return find_vendored::get_openssl(target);
        }
    }
    find_normal::get_openssl(target)
}

fn env_inner(name: &str) -> Option<OsString> {
    let var = env::var_os(name);
    println!("cargo:rerun-if-env-changed={}", name);

    match var {
        Some(ref v) => println!("{} = {}", name, v.to_string_lossy()),
        None => println!("{} unset", name),
    }

    var
}

fn env(name: &str) -> Option<OsString> {
    let prefix = env::var("TARGET").unwrap().to_uppercase().replace('-', "_");
    let prefixed = format!("{}_{}", prefix, name);
    env_inner(&prefixed).or_else(|| env_inner(name))
}
