#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int sign_ipa(
    const char* input_path,
    
    const char* cert_file,
    const char* pkey_file,
    const char* prov_file,
    const char* password,
    int adhoc,
    int sha256_only,
    
    const char* bundle_id,
    const char* bundle_name,
    const char* bundle_version,
    const char* entitlements_file, 
    
    const char** dylib_files,
    int dylib_count,
    int weak_inject,
    
    int force,
    int check_signature,
    const char* temp_folder,
    
    int debug,
    int quiet
);

const char* get_zsign_version();

#ifdef __cplusplus
}
#endif
