#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use thiserror::Error;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Debug, Default)]
pub struct ZSignOptions {
    pub input_path: String,

    pub cert_file: Option<String>,
    pub pkey_file: Option<String>,
    pub prov_file: Option<String>,
    pub password: Option<String>,
    pub adhoc: bool,
    pub sha256_only: bool,

    pub bundle_id: Option<String>,
    pub bundle_name: Option<String>,
    pub bundle_version: Option<String>,
    pub entitlements_file: Option<String>,

    pub dylib_files: Vec<String>,
    pub weak_inject: bool,

    pub force: bool,
    pub check_signature: bool,
    pub temp_folder: Option<String>,

    pub debug: bool,
    pub quiet: bool,
}

#[derive(Debug, Clone, Error)]
pub enum ZSignError {
    #[error("Invalid input parameters")]
    InvalidInput,
    #[error("Signing operation failed")]
    SigningFailed,
    #[error("File is not signed")]
    NotSigned,
    #[error("Unknown error, code {0}")]
    Unknown(i32),
}

impl ZSignOptions {
    pub fn new<S: Into<String>>(input_path: S) -> Self {
        Self {
            input_path: input_path.into(),
            ..Default::default()
        }
    }

    pub fn with_adhoc_signing(mut self) -> Self {
        self.adhoc = true;
        self
    }

    pub fn with_cert_file<S: Into<String>>(mut self, cert_file: S) -> Self {
        self.cert_file = Some(cert_file.into());
        self
    }

    pub fn with_pkey_file<S: Into<String>>(mut self, pkey_file: S) -> Self {
        self.pkey_file = Some(pkey_file.into());
        self
    }

    pub fn with_prov_file<S: Into<String>>(mut self, prov_file: S) -> Self {
        self.prov_file = Some(prov_file.into());
        self
    }

    pub fn with_password<S: Into<String>>(mut self, password: S) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_bundle_id<S: Into<String>>(mut self, bundle_id: S) -> Self {
        self.bundle_id = Some(bundle_id.into());
        self
    }

    pub fn with_bundle_name<S: Into<String>>(mut self, bundle_name: S) -> Self {
        self.bundle_name = Some(bundle_name.into());
        self
    }

    pub fn with_bundle_version<S: Into<String>>(mut self, bundle_version: S) -> Self {
        self.bundle_version = Some(bundle_version.into());
        self
    }

    pub fn with_entitlements_file<S: Into<String>>(mut self, entitlements_file: S) -> Self {
        self.entitlements_file = Some(entitlements_file.into());
        self
    }

    pub fn with_temp_folder<S: Into<String>>(mut self, temp_folder: S) -> Self {
        self.temp_folder = Some(temp_folder.into());
        self
    }

    pub fn with_weak_inject(mut self) -> Self {
        self.weak_inject = true;
        self
    }

    pub fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    pub fn with_check_signature(mut self) -> Self {
        self.check_signature = true;
        self
    }

    pub fn with_quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    pub fn with_debug(mut self) -> Self {
        self.debug = true;
        self
    }

    pub fn add_dylib<S: Into<String>>(mut self, dylib_path: S) -> Self {
        self.dylib_files.push(dylib_path.into());
        self
    }

    pub fn sign(&self) -> Result<(), ZSignError> {
        let input_path = CString::new(self.input_path.clone()).unwrap();
        let cert_file = self
            .cert_file
            .as_ref()
            .map(|s| CString::new(s.clone()).unwrap());
        let pkey_file = self
            .pkey_file
            .as_ref()
            .map(|s| CString::new(s.clone()).unwrap());
        let prov_file = self
            .prov_file
            .as_ref()
            .map(|s| CString::new(s.clone()).unwrap());
        let password = self
            .password
            .as_ref()
            .map(|s| CString::new(s.clone()).unwrap());
        let bundle_id = self
            .bundle_id
            .as_ref()
            .map(|s| CString::new(s.clone()).unwrap());
        let bundle_name = self
            .bundle_name
            .as_ref()
            .map(|s| CString::new(s.clone()).unwrap());
        let bundle_version = self
            .bundle_version
            .as_ref()
            .map(|s| CString::new(s.clone()).unwrap());
        let entitlements_file = self
            .entitlements_file
            .as_ref()
            .map(|s| CString::new(s.clone()).unwrap());
        let temp_folder = self
            .temp_folder
            .as_ref()
            .map(|s| CString::new(s.clone()).unwrap());

        let dylib_cstrings: Vec<CString> = self
            .dylib_files
            .iter()
            .map(|s| CString::new(s.clone()).unwrap())
            .collect();
        let mut dylib_ptrs: Vec<*const c_char> =
            dylib_cstrings.iter().map(|cs| cs.as_ptr()).collect();

        unsafe {
            let result = sign_ipa(
                input_path.as_ptr(),
                cert_file.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                pkey_file.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                prov_file.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                password.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                if self.adhoc { 1 } else { 0 },
                if self.sha256_only { 1 } else { 0 },
                bundle_id.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                bundle_name.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                bundle_version.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                entitlements_file
                    .as_ref()
                    .map_or(ptr::null(), |s| s.as_ptr()),
                if dylib_ptrs.is_empty() {
                    ptr::null_mut()
                } else {
                    dylib_ptrs.as_mut_ptr()
                },
                dylib_ptrs.len() as i32,
                if self.weak_inject { 1 } else { 0 },
                if self.force { 1 } else { 0 },
                if self.check_signature { 1 } else { 0 },
                temp_folder.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                if self.debug { 1 } else { 0 },
                if self.quiet { 1 } else { 0 },
            );

            match result {
                0 => Ok(()),
                -1 => Err(ZSignError::SigningFailed),
                -2 => Err(ZSignError::NotSigned),
                code => Err(ZSignError::Unknown(code)),
            }
        }
    }
}

pub fn get_version() -> String {
    unsafe {
        let version_ptr = get_zsign_version();
        let version_cstr = CStr::from_ptr(version_ptr);
        version_cstr.to_str().unwrap().to_string()
    }
}
