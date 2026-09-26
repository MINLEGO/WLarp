//! Gestionnaire d'identifiants Windows (CredMan + DPAPI) : la clé OpenRouter n'est
//! jamais stockée en clair, jamais dans la base SQLite.

use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{GetLastError, LocalFree, HLOCAL};
use windows_sys::Win32::Security::Credentials::{
    CredDeleteW, CredFree, CredReadW, CredWriteW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
    CREDENTIALW,
};
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
};

fn utf16(s: &str) -> Vec<u16> {
    format!("WLarp:{s}")
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}

fn utf16_raw(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Copie un blob renvoyé par CryptProtect*/Unprotect* (alloué par LocalAlloc) puis le libère.
unsafe fn take_local(blob: &CRYPT_INTEGER_BLOB) -> Vec<u8> {
    if blob.pbData.is_null() || blob.cbData == 0 {
        return Vec::new();
    }
    let v = std::slice::from_raw_parts(blob.pbData, blob.cbData as usize).to_vec();
    LocalFree(blob.pbData as HLOCAL);
    v
}

/// Chiffre (DPAPI user) puis enregistre dans le trousseau Windows.
pub fn set_secret(service: &str, secret: &str) -> Result<(), String> {
    if secret.trim().is_empty() {
        return Err("Clé API vide.".into());
    }
    unsafe {
        let input = CRYPT_INTEGER_BLOB {
            cbData: secret.len() as u32,
            pbData: secret.as_ptr() as *mut u8,
        };
        let mut enc = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };
        if CryptProtectData(
            &input,
            std::ptr::null(),
            std::ptr::null(),
            null_mut(),
            std::ptr::null(),
            0,
            &mut enc,
        ) == 0
        {
            return Err("Chiffrement DPAPI impossible.".into());
        }
        let enc_bytes = take_local(&enc);
        let mut target = utf16(service);
        let mut user = utf16_raw("WLarp");
        let mut cred: CREDENTIALW = std::mem::zeroed();
        cred.Type = CRED_TYPE_GENERIC;
        cred.Persist = CRED_PERSIST_LOCAL_MACHINE;
        cred.TargetName = target.as_mut_ptr();
        cred.UserName = user.as_mut_ptr();
        cred.CredentialBlob = enc_bytes.as_ptr() as *mut u8;
        cred.CredentialBlobSize = enc_bytes.len() as u32;
        if CredWriteW(&cred, 0) == 0 {
            return Err(format!(
                "Écriture dans le trousseau Windows impossible (erreur {}).",
                GetLastError()
            ));
        }
    }
    Ok(())
}

/// Clé déchiffrée, ou None si absente/illisible.
pub fn get_secret(service: &str) -> Option<String> {
    unsafe {
        let mut target = utf16(service);
        let mut raw: *mut CREDENTIALW = null_mut();
        if CredReadW(target.as_mut_ptr(), CRED_TYPE_GENERIC, 0, &mut raw) == 0 {
            return None;
        }
        let bytes = std::slice::from_raw_parts(
            (*raw).CredentialBlob as *const u8,
            (*raw).CredentialBlobSize as usize,
        )
        .to_vec();
        CredFree(raw as *mut std::ffi::c_void);
        let enc = CRYPT_INTEGER_BLOB {
            cbData: bytes.len() as u32,
            pbData: bytes.as_ptr() as *mut u8,
        };
        let mut plain = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };
        if CryptUnprotectData(
            &enc,
            null_mut(),
            std::ptr::null(),
            null_mut(),
            null_mut(),
            0,
            &mut plain,
        ) == 0
        {
            return None;
        }
        String::from_utf8(take_local(&plain)).ok().filter(|s| !s.is_empty())
    }
}

/// Retire du trousseau (silencieux si absent).
pub fn delete_secret(service: &str) {
    unsafe {
        let mut target = utf16(service);
        CredDeleteW(target.as_mut_ptr(), CRED_TYPE_GENERIC, 0);
    }
}