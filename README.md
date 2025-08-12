# zsign-rust

[![Build zsign-rust](https://github.com/nab138/zsign-rust/actions/workflows/build.yml/badge.svg)](https://github.com/nab138/zsign-rust/actions/workflows/build.yml)

Rust bindings for [zsign](https://github.com/zhlynn/zsign)

Note that the ability to zip, unzip, and install has been removed from zsign for simplicity. If you want this functionality you will have to implement it yourself.

## Example Usage

```rust
let result = zsign_rust::ZSignOptions::new(path_to_app_folder)
    .with_cert_file(path_to_certificate)
    .with_pkey_file(path_to_private_key)
    .with_prov_file(path_to_provisioning_profile)
    .sign();
```

### Credits

- A lot of code was borrowed from https://github.com/sfackler/rust-openssl/blob/master/openssl-sys
