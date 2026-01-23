use anyhow::Result;
use clap::Parser;
use colored::*;
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about = "HecateOS code quality enforcer")]
struct Cli {
    /// Path to lint (default: current directory)
    #[arg(default_value = ".")]
    path: String,

    /// Auto-fix issues when possible
    #[arg(short, long)]
    fix: bool,

    /// Check only specific rules
    #[arg(short = 'r', long)]
    rules: Option<Vec<String>>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug)]
struct LintIssue {
    file: String,
    line: usize,
    rule: String,
    message: String,
    fixable: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let mut issues = Vec::new();
    
    // Run all lint checks
    check_rust_files(&cli.path, &mut issues, cli.fix)?;
    check_documentation(&cli.path, &mut issues)?;
    check_config_files(&cli.path, &mut issues)?;
    
    // Display results
    if issues.is_empty() {
        println!("{} No issues found!", "✓".green().bold());
    } else {
        println!("{} Found {} issue(s):", "⚠".yellow().bold(), issues.len());
        for issue in &issues {
            println!(
                "  {}:{}  {} - {}{}",
                issue.file,
                issue.line,
                issue.rule.yellow(),
                issue.message,
                if issue.fixable { " (fixable)" } else { "" }
            );
        }
        
        if cli.fix {
            println!("\n{} Fixed {} auto-fixable issues", 
                "✓".green(), 
                issues.iter().filter(|i| i.fixable).count()
            );
        } else if issues.iter().any(|i| i.fixable) {
            println!("\n{} Run with --fix to automatically fix some issues", 
                "Tip".cyan().bold());
        }
    }
    
    Ok(())
}

fn check_rust_files(path: &str, issues: &mut Vec<LintIssue>, fix: bool) -> Result<()> {
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some("rs".as_ref()))
    {
        let file_path = entry.path();
        let content = fs::read_to_string(file_path)?;
        
        // Check for missing license headers
        if !content.starts_with("//") && !content.starts_with("/*") {
            issues.push(LintIssue {
                file: file_path.display().to_string(),
                line: 1,
                rule: "license-header".to_string(),
                message: "Missing license header".to_string(),
                fixable: true,
            });
            
            if fix {
                let header = "// Copyright (c) 2026 HecateOS Team\n// SPDX-License-Identifier: MIT\n\n";
                let new_content = format!("{}{}", header, content);
                fs::write(file_path, new_content)?;
            }
        }
        
        // Check for TODO/FIXME comments
        for (line_num, line) in content.lines().enumerate() {
            if line.contains("TODO") || line.contains("FIXME") {
                issues.push(LintIssue {
                    file: file_path.display().to_string(),
                    line: line_num + 1,
                    rule: "todo-comment".to_string(),
                    message: format!("Found {}", if line.contains("TODO") { "TODO" } else { "FIXME" }),
                    fixable: false,
                });
            }
        }
        
        // Check for long lines
        for (line_num, line) in content.lines().enumerate() {
            if line.len() > 120 {
                issues.push(LintIssue {
                    file: file_path.display().to_string(),
                    line: line_num + 1,
                    rule: "line-length".to_string(),
                    message: format!("Line exceeds 120 characters ({})", line.len()),
                    fixable: false,
                });
            }
        }
    }
    
    Ok(())
}

fn check_documentation(path: &str, issues: &mut Vec<LintIssue>) -> Result<()> {
    let required_docs = vec![
        "README.md",
        "LICENSE",
        "CHANGELOG.md",
    ];
    
    for doc in required_docs {
        let doc_path = Path::new(path).join(doc);
        if !doc_path.exists() {
            issues.push(LintIssue {
                file: doc,
                line: 0,
                rule: "missing-doc".to_string(),
                message: format!("Required documentation file missing"),
                fixable: false,
            });
        }
    }
    
    Ok(())
}

fn check_config_files(path: &str, issues: &mut Vec<LintIssue>) -> Result<()> {
    // Check TOML files for validity
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some("toml".as_ref()))
    {
        let file_path = entry.path();
        let content = fs::read_to_string(file_path)?;
        
        if let Err(e) = content.parse::<toml_edit::Document>() {
            issues.push(LintIssue {
                file: file_path.display().to_string(),
                line: 0,
                rule: "invalid-toml".to_string(),
                message: format!("Invalid TOML: {}", e),
                fixable: false,
            });
        }
    }
    
    Ok(())
}