use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::ssh;

pub const GAM_FILE_NAME: &str = ".gam.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshAccount {
    pub name: String,
    pub email: String,
    pub key_file: String,
    pub host: String,
    pub description: Option<String>,
    #[serde(default)]
    pub git_user_name: Option<String>,
    #[serde(default)]
    pub git_user_email: Option<String>,
}

impl SshAccount {
    pub fn public(&self, active: bool) -> AccountPublic {
        AccountPublic {
            id: self.name.clone(),
            email: self.email.clone(),
            host: self.host.clone(),
            alias: ssh::alias_for(self),
            git_user_name: self.git_user_name.clone(),
            git_user_email: self.git_user_email.clone(),
            description: self.description.clone(),
            active,
        }
    }
}

/// Portable marker committed in a repo. Never contains keys, emails, or paths.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GamFile {
    pub account: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_alias: Option<String>,
}

impl GamFile {
    pub fn new(account: impl Into<String>, host_alias: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            host_alias: Some(host_alias.into()),
        }
    }

    pub fn read(repo_root: &Path) -> Result<Option<Self>> {
        let path = repo_root.join(GAM_FILE_NAME);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let parsed: GamFile = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(Some(parsed))
    }

    pub fn write(&self, repo_root: &Path) -> Result<()> {
        let path = repo_root.join(GAM_FILE_NAME);
        let value = serde_json::json!({
            "account": self.account,
            "host_alias": self.host_alias,
        });
        let body = serde_json::to_string_pretty(&value).context("Failed to serialize .gam.json")?
            + "\n";
        fs::write(&path, body).with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBinding {
    pub local_path: String,
    pub account_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub accounts: HashMap<String, SshAccount>,
    pub current_account: Option<String>,
    #[serde(default)]
    pub projects: Vec<ProjectBinding>,
}

impl Config {
    pub fn public_accounts(&self) -> Vec<AccountPublic> {
        let mut accounts: Vec<AccountPublic> = self
            .accounts
            .iter()
            .map(|(id, acc)| acc.public(self.current_account.as_deref() == Some(id)))
            .collect();
        accounts.sort_by(|a, b| a.id.cmp(&b.id));
        accounts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPublic {
    pub id: String,
    pub email: String,
    pub host: String,
    pub alias: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_user_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPublic {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    pub exists: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResolveSource {
    Explicit,
    GamFile,
    LocalPath,
    Origin,
    GitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountPublic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectPublic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ResolveSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<AccountPublic>>,
}

impl ResolveResult {
    pub fn found(
        account: AccountPublic,
        project: ProjectPublic,
        source: ResolveSource,
    ) -> Self {
        Self {
            ok: true,
            account: Some(account),
            project: Some(project),
            source: Some(source),
            code: None,
            message: None,
            accounts: None,
        }
    }

    pub fn error(
        code: &str,
        message: impl Into<String>,
        accounts: Option<Vec<AccountPublic>>,
    ) -> Self {
        Self {
            ok: false,
            account: None,
            project: None,
            source: None,
            code: Some(code.to_string()),
            message: Some(message.into()),
            accounts,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsureResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountPublic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectPublic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ResolveSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applied: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<AccountPublic>>,
}

impl From<ResolveResult> for EnsureResult {
    fn from(value: ResolveResult) -> Self {
        Self {
            ok: value.ok,
            account: value.account,
            project: value.project,
            source: value.source,
            applied: Vec::new(),
            code: value.code,
            message: value.message,
            accounts: value.accounts,
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("GAM_CONFIG") {
        return Ok(PathBuf::from(p));
    }
    let home_dir = home::home_dir().context("Could not find home directory")?;
    Ok(home_dir.join(".ssh").join("gam_config.json"))
}

pub fn ssh_dir() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("GAM_SSH_DIR") {
        return Ok(PathBuf::from(p));
    }
    let home_dir = home::home_dir().context("Could not find home directory")?;
    Ok(home_dir.join(".ssh"))
}

pub fn legacy_config_path() -> Result<PathBuf> {
    let home_dir = home::home_dir().context("Could not find home directory")?;
    Ok(home_dir.join(".ssh").join("ssh_manager_config.json"))
}

/// Normalize a git remote URL to `host/owner/repo` (no `.git`, no credentials).
pub fn normalize_origin(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    let without_git = url.trim_end_matches(".git");

    let rest = if let Some(rest) = without_git.strip_prefix("git@") {
        rest.replace(':', "/")
    } else if let Some(rest) = without_git.strip_prefix("ssh://") {
        let rest = rest.strip_prefix("git@").unwrap_or(rest);
        rest.replace(':', "/")
    } else if let Some(rest) = without_git.strip_prefix("https://") {
        rest.split('@').next_back().unwrap_or(rest).to_string()
    } else if let Some(rest) = without_git.strip_prefix("http://") {
        rest.split('@').next_back().unwrap_or(rest).to_string()
    } else {
        without_git.replace(':', "/")
    };

    let parts: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 {
        Some(parts.join("/"))
    } else if rest.is_empty() {
        None
    } else {
        Some(rest)
    }
}

/// Compare remotes ignoring SSH host aliases (`github.com/org/repo` == `github-work/org/repo`).
pub fn origin_repo_key(url: &str) -> Option<String> {
    let normalized = normalize_origin(url)?;
    let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 {
        Some(format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1]))
    } else {
        Some(normalized)
    }
}

pub fn paths_eq(a: &Path, b: &Path) -> bool {
    let ca = a.canonicalize().unwrap_or_else(|_| a.to_path_buf());
    let cb = b.canonicalize().unwrap_or_else(|_| b.to_path_buf());
    ca == cb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_ssh_and_https() {
        assert_eq!(
            normalize_origin("git@github.com:org/repo.git").as_deref(),
            Some("github.com/org/repo")
        );
        assert_eq!(
            normalize_origin("https://github.com/org/repo.git").as_deref(),
            Some("github.com/org/repo")
        );
        assert_eq!(
            origin_repo_key("git@github-work:org/repo.git").as_deref(),
            origin_repo_key("https://github.com/org/repo.git").as_deref()
        );
    }

    #[test]
    fn gam_file_roundtrip_has_no_secrets() {
        let dir = tempfile::tempdir().unwrap();
        let file = GamFile::new("work", "github-work");
        file.write(dir.path()).unwrap();
        let raw = fs::read_to_string(dir.path().join(GAM_FILE_NAME)).unwrap();
        assert!(raw.contains("\"account\": \"work\""));
        assert!(raw.contains("github-work"));
        for forbidden in ["email", "key_file", "passphrase", "token", "local_path", "@"] {
            assert!(
                !raw.contains(forbidden),
                "`.gam.json` must not contain `{forbidden}`: {raw}"
            );
        }
        let parsed = GamFile::read(dir.path()).unwrap().unwrap();
        assert_eq!(parsed, file);
    }

    #[test]
    fn public_account_omits_key_file() {
        let account = SshAccount {
            name: "work".into(),
            email: "a@b.com".into(),
            key_file: "id_work_github_com".into(),
            host: "github.com".into(),
            description: None,
            git_user_name: Some("Gio".into()),
            git_user_email: Some("a@b.com".into()),
        };
        let json = serde_json::to_string(&account.public(true)).unwrap();
        assert!(!json.contains("key_file"));
        assert!(!json.contains("id_work"));
        assert!(json.contains("github-work"));
    }
}
