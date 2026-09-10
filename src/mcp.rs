use std::path::PathBuf;

use anyhow::Result;
use rmcp::{
    handler::server::wrapper::Parameters, schemars, tool, tool_router, transport::stdio, ServiceExt,
};
use serde::Deserialize;

use crate::manager::{AddAccountRequest, SshManager};

#[derive(Clone, Default)]
pub struct GamMcp;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PathArgs {
    /// Absolute or relative path to a git work tree. Defaults to the process cwd.
    path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AccountArgs {
    /// Account id as stored in GAM (e.g. "work").
    account: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EnsureArgs {
    /// Absolute or relative path to a git work tree. Defaults to the process cwd.
    path: Option<String>,
    /// Optional account id. If omitted, GAM resolves via .gam.json, local registry, origin, then git identity.
    account: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AddAccountArgs {
    /// Short account id, e.g. "work" or "github-somnio".
    name: String,
    /// Email for the SSH key comment and default git user.email.
    email: String,
    /// Git host. Defaults to github.com.
    host: Option<String>,
    description: Option<String>,
    /// git config user.name. Defaults to name.
    git_user_name: Option<String>,
    /// git config user.email. Defaults to email.
    git_user_email: Option<String>,
    /// Replace an existing GAM account and key files. Default false.
    overwrite: Option<bool>,
    /// Write Host alias in ~/.ssh/config. Default true.
    update_ssh_config: Option<bool>,
    /// Open a native OS passphrase dialog (macOS / Windows Credential / Linux zenity|kdialog). The secret never appears in MCP logs. Default true.
    use_passphrase: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AttachArgs {
    /// Account id as stored in GAM (e.g. "work").
    account: String,
    /// Absolute or relative path to a git work tree. Defaults to the process cwd.
    path: Option<String>,
}

#[tool_router(server_handler)]
impl GamMcp {
    #[tool(description = "List configured Git SSH accounts. Never returns private key material.")]
    fn list_accounts(&self) -> String {
        match SshManager::new() {
            Ok(mgr) => json(&mgr.accounts_view()),
            Err(e) => err_json(&e),
        }
    }

    #[tool(description = "Resolve which GAM account belongs to a local git repo. Read-only.")]
    fn resolve_project(&self, Parameters(args): Parameters<PathArgs>) -> String {
        match load_and(|mgr| mgr.resolve(&path_or_cwd(args.path)?, None)) {
            Ok(v) => json(&v),
            Err(e) => err_json(&e),
        }
    }

    #[tool(description = "List local project bindings: path, origin URL, and account id.")]
    fn list_projects(&self) -> String {
        match SshManager::new() {
            Ok(mgr) => json(&mgr.config.projects),
            Err(e) => err_json(&e),
        }
    }

    #[tool(description = "Apply the mapped Git identity (user.name, user.email, core.sshCommand) for a repo. Call this before git fetch/push/commit. Does not write secrets. Mutates local git config.")]
    fn ensure_identity(&self, Parameters(args): Parameters<EnsureArgs>) -> String {
        match load_and_mut(|mgr| {
            mgr.ensure(&path_or_cwd(args.path)?, args.account.as_deref())
        }) {
            Ok(v) => json(&v),
            Err(e) => err_json(&e),
        }
    }

    #[tool(description = "Create a GAM SSH account: generate ed25519 key, save config, update ~/.ssh/config, load ssh-agent. By default opens a native OS dialog for the passphrase (never send the passphrase as an argument). Returns the public key only. Mutates ~/.ssh.")]
    fn add_account(&self, Parameters(args): Parameters<AddAccountArgs>) -> String {
        match load_and_mut(|mgr| {
            mgr.add_named(AddAccountRequest {
                name: args.name,
                email: args.email,
                host: args
                    .host
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| "github.com".into()),
                description: args.description,
                git_user_name: args.git_user_name,
                git_user_email: args.git_user_email,
                overwrite: args.overwrite.unwrap_or(false),
                update_ssh_config: args.update_ssh_config.unwrap_or(true),
                passphrase: None,
                use_passphrase: args.use_passphrase.unwrap_or(true),
            })
        }) {
            Ok(v) => json(&v),
            Err(e) => err_json(&e),
        }
    }

    #[tool(description = "Bind a repo to an account, write .gam.json (account + host alias only), and apply local git identity. Mutates the repo.")]
    fn attach_project(&self, Parameters(args): Parameters<AttachArgs>) -> String {
        match load_and_mut(|mgr| mgr.attach_named(&path_or_cwd(args.path)?, &args.account)) {
            Ok(v) => json(&v),
            Err(e) => err_json(&e),
        }
    }

    #[tool(description = "Legacy global switch: mark an account as current and remap Host <hostname> in ~/.ssh/config. Prefer ensure_identity for per-repo work.")]
    fn switch_account(&self, Parameters(args): Parameters<AccountArgs>) -> String {
        match load_and_mut(|mgr| mgr.switch_named(&args.account)) {
            Ok(v) => json(&v),
            Err(e) => err_json(&e),
        }
    }
}

fn path_or_cwd(path: Option<String>) -> Result<PathBuf> {
    match path {
        Some(p) if !p.trim().is_empty() => Ok(PathBuf::from(p)),
        _ => std::env::current_dir().map_err(anyhow::Error::from),
    }
}

fn load_and<T>(f: impl FnOnce(&SshManager) -> Result<T>) -> Result<T> {
    let mgr = SshManager::new()?;
    f(&mgr)
}

fn load_and_mut<T>(f: impl FnOnce(&mut SshManager) -> Result<T>) -> Result<T> {
    let mut mgr = SshManager::new()?;
    f(&mut mgr)
}

fn json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|e| err_json(&anyhow::anyhow!("{e}")))
}

fn err_json(err: &anyhow::Error) -> String {
    serde_json::json!({
        "ok": false,
        "code": "internal",
        "message": format!("{err:#}"),
    })
    .to_string()
}

pub fn run() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let service = GamMcp.serve(stdio()).await?;
        service.waiting().await?;
        Ok(())
    })
}
