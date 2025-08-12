use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=zsign");
    let openssl_include = std::env::var("DEP_OPENSSL_INCLUDE").unwrap();
    let openssl_libs = std::env::var("DEP_OPENSSL_LIBS").unwrap();

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .warnings(false)
        .include("zsign")
        .include("zsign/common")
        .include(&openssl_include);

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

    println!("cargo:rustc-link-search=native={}", openssl_libs);
    println!("cargo:rustc-link-lib=ssl");
    println!("cargo:rustc-link-lib=crypto");

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
