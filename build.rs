use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=zsign");

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .warnings(false)
        .include("zsign")
        .include("zsign/common");

    if cfg!(target_env = "msvc") {
        // Avoid std::byte ambiguity with Windows headers
        build.flag_if_supported("/std:c++14");
        build.flag_if_supported("/EHsc");
        build.define("_HAS_STD_BYTE", Some("0"));
        build.define("_CRT_SECURE_NO_WARNINGS", None);
    } else {
        build.std("c++11");
    }

    // Collect OpenSSL include path
    let mut added_ssl = false;
    for var in ["DEP_OPENSSL_SYS_INCLUDE", "DEP_OPENSSL_INCLUDE"] {
        if let Ok(val) = env::var(var) {
            for p in val.split(';') {
                if !p.is_empty() {
                    build.include(p);
                    added_ssl = true;
                }
            }
        }
    }

    // Fallback: search target build dirs if not found
    if !added_ssl {
        if let Ok(profile) = env::var("PROFILE") {
            let target_dir = Path::new("target").join(&profile).join("build");
            if let Ok(entries) = fs::read_dir(target_dir) {
                for e in entries.flatten() {
                    let path = e.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("openssl-sys-") {
                            let cand = path
                                .join("out")
                                .join("openssl-build")
                                .join("install")
                                .join("include");
                            if cand.join("openssl").join("pem.h").is_file() {
                                build.include(&cand);
                                added_ssl = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    if !added_ssl {
        println!("cargo:warning=OpenSSL include path not found (no DEP_OPENSSL_SYS_INCLUDE).");
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
