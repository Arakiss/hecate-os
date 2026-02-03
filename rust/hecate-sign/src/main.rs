//! HecateOS Signature Tool CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use hecate_sign::{KeyPair, TrustStore, SignaturePurpose, sign_directory, verify_manifest};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hecate-sign")]
#[command(about = "HecateOS digital signature tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new key pair
    Generate {
        /// Output directory for keys
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
        
        /// Key name prefix
        #[arg(short, long, default_value = "hecate")]
        name: String,
    },
    
    /// Sign a file or directory
    Sign {
        /// Path to sign
        path: PathBuf,
        
        /// Private key file
        #[arg(short = 'k', long)]
        key: PathBuf,
        
        /// Public key file
        #[arg(short = 'p', long)]
        pubkey: PathBuf,
        
        /// Signer name
        #[arg(short, long)]
        signer: String,
        
        /// Output manifest file
        #[arg(short, long, default_value = "signature.json")]
        output: PathBuf,
    },
    
    /// Verify a signature
    Verify {
        /// Signature manifest file
        manifest: PathBuf,
        
        /// Base path for files
        #[arg(short, long, default_value = ".")]
        base: PathBuf,
    },
    
    /// Manage trust store
    Trust {
        #[command(subcommand)]
        action: TrustAction,
    },
}

#[derive(Subcommand)]
enum TrustAction {
    /// Add a key to trust store
    Add {
        /// Key name
        name: String,
        
        /// Public key file
        pubkey: PathBuf,
    },
    
    /// List trusted keys
    List,
    
    /// Revoke a key
    Revoke {
        /// Key ID to revoke
        key_id: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { output, name } => {
            println!("{}", "Generating new key pair...".bright_cyan());
            
            let keypair = KeyPair::generate();
            let key_id = keypair.key_id();
            
            let private_path = output.join(format!("{}.key", name));
            let public_path = output.join(format!("{}.pub", name));
            
            keypair.save(&private_path, &public_path)?;
            
            println!("{}", "Key pair generated successfully!".green());
            println!("  Private key: {}", private_path.display());
            println!("  Public key:  {}", public_path.display());
            println!("  Key ID:      {}", key_id.bright_yellow());
            println!("\n{}", "⚠ Keep the private key secure!".red().bold());
        }
        
        Commands::Sign { path, key, pubkey, signer, output } => {
            println!("Signing {}...", path.display());
            
            let keypair = KeyPair::load(&key, &pubkey)?;
            let manifest = sign_directory(
                &path,
                &keypair,
                signer,
                SignaturePurpose::Package,
            )?;
            
            let json = serde_json::to_string_pretty(&manifest)?;
            std::fs::write(&output, json)?;
            
            println!("{}", "Signature created successfully!".green());
            println!("  Manifest: {}", output.display());
            println!("  Files signed: {}", manifest.files.len());
        }
        
        Commands::Verify { manifest, base } => {
            println!("Verifying signature...");
            
            let content = std::fs::read_to_string(&manifest)?;
            let manifest: hecate_sign::SignatureManifest = serde_json::from_str(&content)?;
            
            if verify_manifest(&manifest, &base)? {
                println!("{}", "✓ Signature valid!".green().bold());
                println!("  Signer: {}", manifest.signer.name);
                println!("  Key ID: {}", manifest.signer.key_id);
                println!("  Timestamp: {}", manifest.timestamp);
            } else {
                println!("{}", "✗ Signature INVALID!".red().bold());
                std::process::exit(1);
            }
        }
        
        Commands::Trust { action } => {
            let trust_store_path = PathBuf::from("/etc/hecate/trust.json");
            let mut store = TrustStore::load(&trust_store_path)?;
            
            match action {
                TrustAction::Add { name, pubkey } => {
                    let key_bytes = std::fs::read(&pubkey)?;
                    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
                        &key_bytes.try_into()
                            .map_err(|_| anyhow::anyhow!("Invalid key size"))?
                    )?;
                    
                    store.add_key(name.clone(), &verifying_key)?;
                    println!("{} added to trust store", name.green());
                }
                
                TrustAction::List => {
                    println!("{}", "Trusted keys:".bright_cyan());
                    // Implementation would list keys from store
                }
                
                TrustAction::Revoke { key_id } => {
                    store.revoke_key(&key_id)?;
                    println!("Key {} revoked", key_id.red());
                }
            }
        }
    }

    Ok(())
}