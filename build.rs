use std::env;
use std::path::PathBuf;

fn main() {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .warnings(false)
        .include("zsign")
        .include("zsign/common");

    if cfg!(target_env = "msvc") {
        build.flag_if_supported("/std:c++17");
    } else {
        build.std("c++11");
    }

    if let Ok(inc) = env::var("DEP_OPENSSL_INCLUDE") {
        for p in inc.split(';') {
            // split in case Windows gives a ';' list
            if !p.is_empty() {
                build.include(p);
            }
        }
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
        build.file("zsign/common/common_win32.cpp");
        build.file("zsign/common/iconv.cpp");
        build.file("zsign/common/getopt.cpp");
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
        .expect("Couldn't write bindings!");
}
