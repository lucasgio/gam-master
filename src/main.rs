use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use gam_cli::manager::{print_add_human, print_status_human, AddAccountRequest, SshManager};
use gam_cli::output::{self, wants_prompt};
use gam_cli::ui;
use gam_cli::{EnsureResult, ResolveResult};

#[derive(Parser, Debug)]
#[command(name = "gam")]
#[command(version)]
#[command(about = "Manage multiple Git SSH identities, per repository")]
#[command(long_about = ui::ROOT_LONG_ABOUT)]
#[command(after_help = ui::ROOT_AFTER_HELP)]
#[command(styles = ui::clap_styles())]
#[command(propagate_version = true)]
struct Args {
    /// Show extra diagnostics (keys, fingerprints, current repo)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Machine-readable JSON on stdout (implies non-interactive)
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new SSH account and key
    #[command(after_help = "Examples:\n  gam add\n  gam add --name work --email you@org.com --json\n\nThen add the printed public key to GitHub/GitLab and run `gam attach` in a repo.")]
    Add {
        /// Account id (required unless interactive)
        #[arg(long)]
        name: Option<String>,
        /// Email used as SSH key comment and default git email
        #[arg(long)]
        email: Option<String>,
        /// git host (default: github.com)
        #[arg(long, default_value = "github.com")]
        host: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        git_user_name: Option<String>,
        #[arg(long)]
        git_user_email: Option<String>,
        /// Replace an existing account and key files
        #[arg(long)]
        overwrite: bool,
        /// Write Host alias in ~/.ssh/config (default true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        update_ssh_config: bool,
        /// Ask for a passphrase in a native OS dialog (not in the MCP/JSON payload)
        #[arg(long)]
        passphrase_prompt: bool,
    },
    /// List configured accounts
    #[command(after_help = "Examples:\n  gam list\n  gam list -v\n  gam list --json")]
    List,
    /// Switch the legacy global active account
    #[command(after_help = "Examples:\n  gam switch --account work\n\nThis remaps Host github.com (or the account host) in ~/.ssh/config.\nPrefer `gam attach` so each repo keeps its own identity.")]
    Switch {
        /// Account id (required unless running interactively)
        #[arg(long)]
        account: Option<String>,
    },
    /// Remove an account and its key
    Remove {
        #[arg(long)]
        account: Option<String>,
        #[arg(long)]
        yes: bool,
    },
    /// Show the global active account and this repo's git identity
    #[command(after_help = "Examples:\n  gam status\n  gam status -v")]
    Status,
    /// Delete every GAM account and managed SSH keys
    Reset {
        #[arg(long)]
        yes: bool,
    },
    /// Bind the current git repo to an account and write .gam.json
    #[command(after_help = "Examples:\n  cd /path/to/repo\n  gam attach --account work\n\nOptional next step:\n  git remote set-url origin git@<ssh-alias>:org/repo.git")]
    Attach {
        #[arg(long)]
        account: Option<String>,
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Apply the mapped identity for a repo (no .gam.json write)
    Ensure {
        #[arg(long)]
        account: Option<String>,
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Resolve which account a repo should use (read-only)
    Resolve {
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// List registered local project bindings
    Projects,
    /// Diagnose SSH keys, config files and the current git identity
    #[command(after_help = "Examples:\n  gam doctor\n  gam doctor -v")]
    Doctor,
    /// Start the MCP stdio server for AI agents
    Mcp,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            output::eprint_err(&err);
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode> {
    let args = Args::parse();
    let json = args.json;
    let verbose = args.verbose;

    match args.command {
        Some(Commands::Mcp) => {
            gam_cli::mcp::run()?;
            Ok(ExitCode::SUCCESS)
        }
        Some(Commands::Add {
            name,
            email,
            host,
            description,
            git_user_name,
            git_user_email,
            overwrite,
            update_ssh_config,
            passphrase_prompt,
        }) => {
            let mut mgr = SshManager::new()?;
            if name.is_none() && email.is_none() && wants_prompt(json) {
                mgr.add_account()?;
                return Ok(ExitCode::SUCCESS);
            }
            let name = name.ok_or_else(|| anyhow::anyhow!("Missing --name (required in non-interactive mode)"))?;
            let email = email.ok_or_else(|| anyhow::anyhow!("Missing --email (required in non-interactive mode)"))?;
            let result = mgr.add_named(AddAccountRequest {
                name,
                email,
                host,
                description,
                git_user_name,
                git_user_email,
                overwrite,
                update_ssh_config,
                passphrase: None,
                use_passphrase: passphrase_prompt,
            })?;
            if json {
                output::print_json(&result)?;
            } else {
                print_add_human(&result);
            }
            Ok(output::exit_from_ok(result.ok))
        }
        Some(Commands::List) => {
            let mgr = SshManager::new()?;
            if json {
                output::print_json(&mgr.accounts_view())?;
            } else {
                mgr.print_accounts(verbose);
            }
            Ok(ExitCode::SUCCESS)
        }
        Some(Commands::Switch { account }) => {
            let mut mgr = SshManager::new()?;
            let account = require_account(&mgr, account, json, "Select account to activate:")?;
            let result = mgr.switch_named(&account)?;
            if json {
                output::print_json(&result)?;
            } else if result.ok {
                println!("✅ Switched to account '{account}'");
            } else {
                output::print_resolve_human(&result);
            }
            Ok(output::exit_from_ok(result.ok))
        }
        Some(Commands::Remove { account, yes }) => {
            let mut mgr = SshManager::new()?;
            if mgr.config.accounts.is_empty() {
                if json {
                    output::print_json(&ResolveResult::error(
                        "no_accounts",
                        "No accounts found.",
                        Some(Vec::new()),
                    ))?;
                } else {
                    println!("📭 No accounts found.");
                }
                return Ok(ExitCode::from(1));
            }
            let account = require_account(&mgr, account, json, "Select account to remove:")?;
            if !yes {
                if !wants_prompt(json) {
                    bail!("Refusing to remove '{account}' without --yes");
                }
                let confirm = inquire::Confirm::new(&format!(
                    "Are you sure you want to remove account '{account}'?"
                ))
                .with_default(false)
                .prompt()?;
                if !confirm {
                    if json {
                        output::print_json(&serde_json::json!({"ok": false, "code": "cancelled"}))?;
                    } else {
                        println!("❌ Removal cancelled.");
                    }
                    return Ok(ExitCode::from(1));
                }
            }
            let removed = mgr.remove_named(&account)?;
            if json {
                output::print_json(&serde_json::json!({"ok": removed, "account": account}))?;
            } else if removed {
                println!("✅ Account '{account}' removed successfully!");
            } else {
                println!("❌ Account '{account}' not found.");
            }
            Ok(output::exit_from_ok(removed))
        }
        Some(Commands::Status) => {
            let status = SshManager::new()?.status()?;
            if json {
                output::print_json(&status)?;
            } else {
                print_status_human(&status);
            }
            Ok(output::exit_from_ok(status.ok))
        }
        Some(Commands::Reset { yes }) => {
            let mut mgr = SshManager::new()?;
            if mgr.config.accounts.is_empty() {
                if !json {
                    println!("📭 No accounts found. Nothing to reset.");
                } else {
                    output::print_json(&serde_json::json!({"ok": true, "message": "nothing to reset"}))?;
                }
                return Ok(ExitCode::SUCCESS);
            }
            if !yes {
                if !wants_prompt(json) {
                    bail!("Refusing to reset without --yes");
                }
                println!("\n⚠️  WARNING: You are about to DELETE ALL accounts and SSH keys managed by gam.");
                println!("⚠️  This will also remove all gam entries from your ~/.ssh/config.");
                println!("⚠️  This action CANNOT be undone.\n");
                let confirm = inquire::Confirm::new("Are you sure you want to reset EVERYTHING?")
                    .with_default(false)
                    .prompt()?;
                if !confirm {
                    println!("❌ Reset cancelled.");
                    return Ok(ExitCode::from(1));
                }
            }
            mgr.reset_all()?;
            if json {
                output::print_json(&serde_json::json!({"ok": true}))?;
            } else {
                println!("\n✅ Application reset successfully. All accounts and keys have been removed.");
            }
            Ok(ExitCode::SUCCESS)
        }
        Some(Commands::Attach { account, path }) => {
            let mut mgr = SshManager::new()?;
            let path = path_or_cwd(path)?;
            let account = require_account(
                &mgr,
                account,
                json,
                "Select account to attach to this repo:",
            )?;
            let result = mgr.attach_named(&path, &account)?;
            emit_ensure(json, &result)
        }
        Some(Commands::Ensure { account, path }) => {
            let mut mgr = SshManager::new()?;
            let path = path_or_cwd(path)?;
            let result = mgr.ensure(&path, account.as_deref())?;
            emit_ensure(json, &result)
        }
        Some(Commands::Resolve { path }) => {
            let mgr = SshManager::new()?;
            let path = path_or_cwd(path)?;
            let result = mgr.resolve(&path, None)?;
            emit_resolve(json, &result)
        }
        Some(Commands::Projects) => {
            let mgr = SshManager::new()?;
            if json {
                output::print_json(&mgr.config.projects)?;
            } else {
                output::print_projects_human(&mgr.config.projects);
            }
            Ok(ExitCode::SUCCESS)
        }
        Some(Commands::Doctor) => {
            let mgr = SshManager::new()?;
            let report = mgr.doctor()?;
            if json {
                output::print_json(&report)?;
            } else {
                mgr.print_doctor(&report, verbose);
            }
            Ok(output::exit_from_ok(report.ok || mgr.config.accounts.is_empty()))
        }
        None => {
            SshManager::new()?.interactive_menu()?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn path_or_cwd(path: Option<PathBuf>) -> Result<PathBuf> {
    match path {
        Some(p) => Ok(p),
        None => Ok(std::env::current_dir()?),
    }
}

fn require_account(
    mgr: &SshManager,
    account: Option<String>,
    json: bool,
    prompt: &str,
) -> Result<String> {
    if let Some(id) = account {
        return Ok(id);
    }
    if mgr.config.accounts.is_empty() {
        bail!("No accounts found. Use 'gam add' to create one.");
    }
    if !wants_prompt(json) {
        bail!("Missing --account (required in non-interactive mode)");
    }
    mgr.prompt_account_name(prompt)
}

fn emit_resolve(json: bool, result: &ResolveResult) -> Result<ExitCode> {
    if json {
        output::print_json(result)?;
    } else {
        output::print_resolve_human(result);
    }
    Ok(output::exit_from_ok(result.ok))
}

fn emit_ensure(json: bool, result: &EnsureResult) -> Result<ExitCode> {
    if json {
        output::print_json(result)?;
    } else {
        output::print_ensure_human(result);
    }
    Ok(output::exit_from_ok(result.ok))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use gam_cli::ssh;
    use gam_cli::SshAccount;

    #[test]
    fn root_help_includes_examples_and_doctor() {
        let help = Args::command().render_long_help().to_string();
        assert!(help.contains("Examples"), "help should include examples:\n{help}");
        assert!(help.contains("doctor"), "help should mention doctor:\n{help}");
        assert!(help.contains("--verbose") || help.contains("-v"));
        assert!(help.contains("lucasgio.github.io/gam-cli"), "help should point at the docs site:\n{help}");
    }

    #[test]
    fn list_help_mentions_verbose() {
        let cmd = Args::command();
        let mut list = cmd
            .find_subcommand("list")
            .expect("list subcommand")
            .clone();
        let help = list.render_long_help().to_string();
        assert!(help.contains("-v") || help.contains("verbose") || help.contains("list -v"));
    }

    #[test]
    fn alias_for_account() {
        let account = SshAccount {
            name: "work".into(),
            email: "a@b.com".into(),
            key_file: "id_work".into(),
            host: "github.com".into(),
            description: None,
            git_user_name: None,
            git_user_email: None,
        };
        assert_eq!(ssh::alias_for(&account), "github-work");
    }
}
