//! HecateOS Digital Signature Library
//!
//! Provides cryptographic signing and verification for packages, updates, and ISO images

use anyhow::{Result, Context};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Sha512, Digest};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

/// Signature manifest for a file or package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureManifest {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub signer: SignerInfo,
    pub files: Vec<FileSignature>,
    pub metadata: SignatureMetadata,
}

/// Information about the signer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerInfo {
    pub name: String,
    pub email: Option<String>,
    pub key_id: String,
    pub public_key: String,
}

/// Signature for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSignature {
    pub path: String,
    pub size: u64,
    pub checksums: FileChecksums,
    pub signature: String,
}

/// Multiple checksums for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChecksums {
    pub sha256: String,
    pub sha512: String,
    pub blake3: String,
}

/// Additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMetadata {
    pub purpose: SignaturePurpose,
    pub expires: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub parent_signature: Option<String>,
}

/// Purpose of the signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignaturePurpose {
    Package,
    Update,
    ISO,
    Repository,
    Certificate,
}

/// Key pair for signing
pub struct KeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl KeyPair {
    /// Generate a new key pair
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Load key pair from files
    pub fn load(private_key_path: &Path, public_key_path: &Path) -> Result<Self> {
        let private_bytes = std::fs::read(private_key_path)
            .context("Failed to read private key")?;
        let public_bytes = std::fs::read(public_key_path)
            .context("Failed to read public key")?;
        
        let signing_key = SigningKey::from_bytes(
            &private_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid private key size"))?
        );
        
        let verifying_key = VerifyingKey::from_bytes(
            &public_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid public key size"))?
        )?;
        
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Save key pair to files
    pub fn save(&self, private_key_path: &Path, public_key_path: &Path) -> Result<()> {
        // Save private key (must be kept secret!)
        let private_bytes = self.signing_key.to_bytes();
        let mut private_file = File::create(private_key_path)?;
        private_file.write_all(&private_bytes)?;
        
        // Set restrictive permissions on private key
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = private_file.metadata()?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600); // Read/write for owner only
            std::fs::set_permissions(private_key_path, permissions)?;
        }
        
        // Save public key
        let public_bytes = self.verifying_key.to_bytes();
        std::fs::write(public_key_path, public_bytes)?;
        
        Ok(())
    }

    /// Get key ID (first 16 chars of hex-encoded public key)
    pub fn key_id(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
            .chars()
            .take(16)
            .collect()
    }
}

/// Sign a single file
pub fn sign_file(file_path: &Path, key_pair: &KeyPair) -> Result<FileSignature> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    
    let size = contents.len() as u64;
    
    // Calculate checksums
    let sha256 = hex::encode(Sha256::digest(&contents));
    let sha512 = hex::encode(Sha512::digest(&contents));
    let blake3 = hex::encode(blake3::hash(&contents).as_bytes());
    
    // Sign the SHA256 hash
    let signature = key_pair.signing_key.sign(&contents);
    let signature_hex = hex::encode(signature.to_bytes());
    
    Ok(FileSignature {
        path: file_path.to_string_lossy().to_string(),
        size,
        checksums: FileChecksums {
            sha256,
            sha512,
            blake3,
        },
        signature: signature_hex,
    })
}

/// Verify a file signature
pub fn verify_file(
    file_path: &Path,
    file_sig: &FileSignature,
    public_key: &VerifyingKey,
) -> Result<bool> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    
    // Verify size
    if contents.len() as u64 != file_sig.size {
        return Ok(false);
    }
    
    // Verify checksums
    let sha256 = hex::encode(Sha256::digest(&contents));
    if sha256 != file_sig.checksums.sha256 {
        return Ok(false);
    }
    
    // Verify signature
    let signature_bytes = hex::decode(&file_sig.signature)?;
    let signature = Signature::from_bytes(
        &signature_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid signature size"))?
    );
    
    Ok(public_key.verify(&contents, &signature).is_ok())
}

/// Sign multiple files and create a manifest
pub fn sign_directory(
    dir_path: &Path,
    key_pair: &KeyPair,
    signer_name: String,
    purpose: SignaturePurpose,
) -> Result<SignatureManifest> {
    let mut files = Vec::new();
    
    // Walk directory and sign all files
    for entry in walkdir::WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let relative_path = path.strip_prefix(dir_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        
        let mut file_sig = sign_file(path, key_pair)?;
        file_sig.path = relative_path;
        files.push(file_sig);
    }
    
    Ok(SignatureManifest {
        version: "1.0.0".to_string(),
        timestamp: Utc::now(),
        signer: SignerInfo {
            name: signer_name,
            email: None,
            key_id: key_pair.key_id(),
            public_key: hex::encode(key_pair.verifying_key.to_bytes()),
        },
        files,
        metadata: SignatureMetadata {
            purpose,
            expires: Some(Utc::now() + chrono::Duration::days(365)),
            revoked: false,
            parent_signature: None,
        },
    })
}

/// Verify a signature manifest
pub fn verify_manifest(
    manifest: &SignatureManifest,
    base_path: &Path,
) -> Result<bool> {
    // Parse public key from manifest
    let public_key_bytes = hex::decode(&manifest.signer.public_key)?;
    let public_key = VerifyingKey::from_bytes(
        &public_key_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid public key size"))?
    )?;
    
    // Check expiration
    if let Some(expires) = manifest.metadata.expires {
        if Utc::now() > expires {
            return Ok(false);
        }
    }
    
    // Check revocation
    if manifest.metadata.revoked {
        return Ok(false);
    }
    
    // Verify each file
    for file_sig in &manifest.files {
        let file_path = base_path.join(&file_sig.path);
        if !verify_file(&file_path, file_sig, &public_key)? {
            return Ok(false);
        }
    }
    
    Ok(true)
}

/// Trust store for managing trusted public keys
pub struct TrustStore {
    trusted_keys: Vec<TrustedKey>,
    store_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedKey {
    pub name: String,
    pub key_id: String,
    pub public_key: String,
    pub added: DateTime<Utc>,
    pub expires: Option<DateTime<Utc>>,
    pub revoked: bool,
}

impl TrustStore {
    /// Load trust store from file
    pub fn load(store_path: &Path) -> Result<Self> {
        let trusted_keys = if store_path.exists() {
            let content = std::fs::read_to_string(store_path)?;
            serde_json::from_str(&content)?
        } else {
            Vec::new()
        };
        
        Ok(Self {
            trusted_keys,
            store_path: store_path.to_path_buf(),
        })
    }

    /// Save trust store to file
    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.trusted_keys)?;
        std::fs::write(&self.store_path, content)?;
        Ok(())
    }

    /// Add a trusted key
    pub fn add_key(&mut self, name: String, public_key: &VerifyingKey) -> Result<()> {
        let key_bytes = public_key.to_bytes();
        let key_hex = hex::encode(key_bytes);
        let key_id = key_hex.chars().take(16).collect();
        
        self.trusted_keys.push(TrustedKey {
            name,
            key_id,
            public_key: key_hex,
            added: Utc::now(),
            expires: Some(Utc::now() + chrono::Duration::days(365 * 2)),
            revoked: false,
        });
        
        self.save()?;
        Ok(())
    }

    /// Check if a key is trusted
    pub fn is_trusted(&self, key_id: &str) -> bool {
        self.trusted_keys.iter().any(|k| 
            k.key_id == key_id && 
            !k.revoked &&
            k.expires.map_or(true, |e| Utc::now() < e)
        )
    }

    /// Revoke a key
    pub fn revoke_key(&mut self, key_id: &str) -> Result<()> {
        for key in &mut self.trusted_keys {
            if key.key_id == key_id {
                key.revoked = true;
            }
        }
        self.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_key_generation() {
        let keypair = KeyPair::generate();
        assert_eq!(keypair.key_id().len(), 16);
    }

    #[test]
    fn test_file_signing() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"test content").unwrap();
        
        let keypair = KeyPair::generate();
        let signature = sign_file(&file_path, &keypair).unwrap();
        
        assert!(verify_file(&file_path, &signature, &keypair.verifying_key).unwrap());
    }

    #[test]
    fn test_manifest_signing() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("file1.txt"), b"content1").unwrap();
        std::fs::write(dir.path().join("file2.txt"), b"content2").unwrap();
        
        let keypair = KeyPair::generate();
        let manifest = sign_directory(
            dir.path(),
            &keypair,
            "Test Signer".to_string(),
            SignaturePurpose::Package,
        ).unwrap();
        
        assert_eq!(manifest.files.len(), 2);
        assert!(verify_manifest(&manifest, dir.path()).unwrap());
    }
}