#include "common/common.h"
#include "macho.h"
#include "bundle.h"
#include "openssl.h"
#include "common/timer.h"
#include "zsign.h"

#ifdef _WIN32
#include "common_win32.h"
#endif

#define ZSIGN_VERSION "0.7"

static string safe_string(const char* str) {
    return str ? string(str) : string();
}

static vector<string> make_dylib_vector(const char** dylib_files, int dylib_count) {
    vector<string> result;
    if (dylib_files) {
        for (int i = 0; i < dylib_count; i++) {
            if (dylib_files[i]) {
                result.push_back(string(dylib_files[i]));
            }
        }
    }
    return result;
}

extern "C" int sign_ipa(
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
)
{
	ZTimer atimer;
	ZTimer gtimer;

	string strInputPath = safe_string(input_path);
	string strCertFile = safe_string(cert_file);
	string strPKeyFile = safe_string(pkey_file);
	string strProvFile = safe_string(prov_file);
	string strPassword = safe_string(password);
	string strBundleId = safe_string(bundle_id);
	string strBundleName = safe_string(bundle_name);
	string strBundleVersion = safe_string(bundle_version);
	string strEntitlementsFile = safe_string(entitlements_file);
	string strTempFolder = safe_string(temp_folder);
	vector<string> arrDylibFiles = make_dylib_vector(dylib_files, dylib_count);
	
	bool bAdhoc = (adhoc != 0);
	bool bSHA256Only = (sha256_only != 0);
	bool bWeakInject = (weak_inject != 0);
	bool bForce = (force != 0);
	bool bCheckSignature = (check_signature != 0);
	bool bDebug = (debug != 0);
	bool bQuiet = (quiet != 0);

	if (strInputPath.empty()) {
		ZLog::Error(">>> Input path is required!\n");
		return -1;
	}

	string strPath = ZFile::GetFullPath(strInputPath.c_str());
	if (!ZFile::IsFileExists(strPath.c_str())) {
		ZLog::ErrorV(">>> Invalid path! %s\n", strPath.c_str());
		return -1;
	}

	if (strTempFolder.empty()) {
		strTempFolder = ZFile::GetTempFolder();
	}
	if (!ZFile::IsFolder(strTempFolder.c_str())) {
		ZLog::ErrorV(">>> Invalid temp folder! %s\n", strTempFolder.c_str());
		return -1;
	}

	if (bQuiet) {
		ZLog::SetLogLever(ZLog::E_NONE);
	} else if (bDebug) {
		ZLog::SetLogLever(ZLog::E_DEBUG);
	}

	if (bDebug) {
		ZFile::CreateFolder("./.zsign_debug");
		ZLog::DebugV(">>> Input path: %s\n", strPath.c_str());
	}

	if (!ZFile::IsFolder(strPath.c_str())) { // macho file
		ZMachO* macho = new ZMachO();
		if (!macho->Init(strPath.c_str())) {
			ZLog::ErrorV(">>> Invalid mach-o file! %s\n", strPath.c_str());
			return -1;
		}

		if (!bAdhoc && arrDylibFiles.empty() && (strPKeyFile.empty() || strProvFile.empty())) {
			if (bCheckSignature) {
				return macho->CheckSignature() ? 0 : -2;
			} else {
				macho->PrintInfo();
				return 0;
			}
		}

		ZSignAsset zsa;
		if (!zsa.Init(strCertFile.c_str(), strPKeyFile.c_str(), strProvFile.c_str(), 
					  strEntitlementsFile.c_str(), strPassword.c_str(), bAdhoc, bSHA256Only, true)) {
			return -1;
		}

		if (!arrDylibFiles.empty()) {
			for (const string& dyLibFile : arrDylibFiles) {
				if (!macho->InjectDylib(bWeakInject, dyLibFile.c_str())) {
					return -1;
				}
			}
		}

		atimer.Reset();
		ZLog::PrintV(">>> Signing:\t%s %s\n", strPath.c_str(), (bAdhoc ? " (Ad-hoc)" : ""));
		string strInfoSHA1;
		string strInfoSHA256;
		string strCodeResourcesData;
		bool bRet = macho->Sign(&zsa, bForce, strBundleId, strInfoSHA1, strInfoSHA256, strCodeResourcesData);
		atimer.PrintResult(bRet, ">>> Signed %s!", bRet ? "OK" : "Failed");
		return bRet ? 0 : -1;
	}

	ZSignAsset zsa;
	if (!zsa.Init(strCertFile.c_str(), strPKeyFile.c_str(), strProvFile.c_str(), 
				  strEntitlementsFile.c_str(), strPassword.c_str(), bAdhoc, bSHA256Only, false)) {
		return -1;
	}

	atimer.Reset();
	ZBundle bundle;
	bool bRet = bundle.SignFolder(&zsa, strPath, strBundleId, strBundleVersion, 
								  strBundleName, arrDylibFiles, bForce, bWeakInject, true);
	atimer.PrintResult(bRet, ">>> Signed %s!", bRet ? "OK" : "Failed");

	gtimer.Print(">>> Done.");
	return bRet ? 0 : -1;
}

extern "C" const char* get_zsign_version()
{
	return ZSIGN_VERSION;
}
