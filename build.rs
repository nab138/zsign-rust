use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=zsign");
    let include_dir = env::var("DEP_OPENSSL_INCLUDE")
        .expect("DEP_OPENSSL_INCLUDE not set — is openssl-sys in build-dependencies?");

    let lib_dir = env::var("DEP_OPENSSL_LIB").expect("DEP_OPENSSL_LIB not set");

    let libkinds = env::var("DEP_OPENSSL_LIBS").unwrap_or_else(|_| "ssl:crypto".into());

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

    println!("cargo:rustc-link-search=native={}", lib_dir);
    for lib in libkinds.split(':') {
        println!("cargo:rustc-link-lib={}", lib);
    }

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
