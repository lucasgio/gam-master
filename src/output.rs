use std::io::{IsTerminal, Write};
use std::process::ExitCode;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::config::{AccountPublic, EnsureResult, ProjectBinding, ResolveResult, ResolveSource};

pub fn wants_prompt(json: bool) -> bool {
    !json && std::io::stdin().is_terminal()
}

pub fn print_json<T: Serialize>(value: &T) -> Result<()> {
    let body = serde_json::to_string_pretty(value).context("Failed to serialize JSON")?;
    println!("{body}");
    Ok(())
}

pub fn exit_from_ok(ok: bool) -> ExitCode {
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

pub fn print_accounts_human(accounts: &[AccountPublic]) {
    if accounts.is_empty() {
        println!("📭 No accounts found. Use 'gam add' to create one.");
        return;
    }
    println!("\n📋 SSH Accounts:\n");
    for account in accounts {
        let active = if account.active {
            "🟢 (active)"
        } else {
            "⚪"
        };
        println!("  {} {} ({})", active, account.id, account.email);
        println!("      Host: {}  alias: {}", account.host, account.alias);
        if let Some(desc) = &account.description {
            println!("      Description: {desc}");
        }
        println!();
    }
}

pub fn print_resolve_human(result: &ResolveResult) {
    if !result.ok {
        eprintln!(
            "❌ {}",
            result
                .message
                .as_deref()
                .unwrap_or("Could not resolve account")
        );
        if let Some(accounts) = &result.accounts {
            if !accounts.is_empty() {
                eprintln!("Available accounts:");
                for a in accounts {
                    eprintln!("  - {} ({})", a.id, a.email);
                }
            }
        }
        return;
    }
    let account = result.account.as_ref().unwrap();
    let source = match result.source {
        Some(ResolveSource::Explicit) => "flag --account",
        Some(ResolveSource::GamFile) => ".gam.json",
        Some(ResolveSource::LocalPath) => "local project registry",
        Some(ResolveSource::Origin) => "origin URL",
        Some(ResolveSource::GitConfig) => "git local identity",
        None => "unknown",
    };
    println!("🟢 Account '{}' ({}) via {source}", account.id, account.email);
    if let Some(project) = &result.project {
        println!("   Path: {}", project.path);
        if let Some(origin) = &project.origin {
            println!("   Origin: {origin}");
        }
    }
}

pub fn print_ensure_human(result: &EnsureResult) {
    let resolve = ResolveResult {
        ok: result.ok,
        account: result.account.clone(),
        project: result.project.clone(),
        source: result.source,
        code: result.code.clone(),
        message: result.message.clone(),
        accounts: result.accounts.clone(),
    };
    print_resolve_human(&resolve);
    if result.ok && !result.applied.is_empty() {
        println!("   Applied: {}", result.applied.join(", "));
    }
}

pub fn print_projects_human(projects: &[ProjectBinding]) {
    if projects.is_empty() {
        println!("📭 No projects registered. Use 'gam attach' in a repo.");
        return;
    }
    println!("\n📁 Projects:\n");
    for p in projects {
        println!("  {} → {}", p.local_path, p.account_id);
        if let Some(origin) = &p.origin {
            println!("      origin: {origin}");
        }
        println!();
    }
}

pub fn eprint_err(err: &anyhow::Error) {
    let _ = writeln!(std::io::stderr(), "❌ {err:#}");
}
