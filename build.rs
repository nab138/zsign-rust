use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=zsign");

    let openssl_dir = env::var_os("OUT_DIR").map(|s| PathBuf::from(s).join("openssl-build"));
    if openssl_dir.is_none() || !openssl_dir.as_ref().unwrap().exists() {
        panic!("OpenSSL build directory does not exist");
    }

    let openssl_dir = openssl_dir.unwrap();

    let include_dir = openssl_dir.join("include");
    let lib_dir = openssl_dir.join("lib");

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .warnings(false)
        .include("zsign")
        .include("zsign/common")
        .include(&include_dir);

    if cfg!(target_env = "msvc") {
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

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

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
