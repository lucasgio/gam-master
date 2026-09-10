use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use inquire::{Confirm, Password, Select, Text};
use regex::Regex;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::config::{
    self, AccountPublic, Config, EnsureResult, GamFile, ProjectBinding, ProjectPublic,
    ResolveResult, ResolveSource, SshAccount,
};
use crate::{git, ssh};

pub struct SshManager {
    config_path: PathBuf,
    ssh_dir: PathBuf,
    pub config: Config,
}

impl SshManager {
    pub fn new() -> Result<Self> {
        let config_path = config::config_path()?;
        let ssh_dir = config::ssh_dir()?;
        Self::from_paths(config_path, ssh_dir)
    }

    pub fn from_paths(config_path: PathBuf, ssh_dir: PathBuf) -> Result<Self> {
        if !ssh_dir.exists() {
            fs::create_dir_all(&ssh_dir).context("Failed to create .ssh directory")?;
        }

        let (config, loaded_from_legacy) = if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
            let cfg = serde_json::from_str(&content).context("Failed to parse config file")?;
            (cfg, false)
        } else if let Ok(legacy) = config::legacy_config_path() {
            if std::env::var("GAM_CONFIG").is_err() && legacy.exists() {
                let content =
                    fs::read_to_string(&legacy).context("Failed to read legacy config file")?;
                let cfg =
                    serde_json::from_str(&content).context("Failed to parse legacy config file")?;
                (cfg, true)
            } else {
                (Config::default(), false)
            }
        } else {
            (Config::default(), false)
        };

        let manager = SshManager {
            config_path: config_path.clone(),
            ssh_dir,
            config,
        };

        if loaded_from_legacy {
            let _ = manager.save_config();
        }

        Ok(manager)
    }

    pub fn save_config(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let json =
            serde_json::to_string_pretty(&self.config).context("Failed to serialize config")?;
        fs::write(&self.config_path, json).context("Failed to write config file")?;
        Ok(())
    }

    pub fn accounts_view(&self) -> Vec<AccountPublic> {
        self.config.public_accounts()
    }

    pub fn print_accounts(&self, verbose: bool) {
        if self.config.accounts.is_empty() {
            crate::ui::empty_state(
                "No accounts found.",
                &["gam add     Create your first SSH identity"],
            );
            return;
        }

        let accounts = self.accounts_view();
        println!();
        println!(
            "📋 {} account(s)  ·  config {}",
            accounts.len(),
            crate::ui::path_home(&self.config_path)
        );
        println!();
        for account in &accounts {
            self.print_account_card(account, verbose);
            println!();
        }
        if !verbose {
            crate::ui::hint("gam list -v    fingerprints, public key path and SSH config");
        }
        crate::ui::hint("gam attach     bind the current git repo to one of these accounts");
    }

    fn print_account_card(&self, account: &AccountPublic, verbose: bool) {
        let marker = if account.active { "🟢" } else { "⚪" };
        let active_tag = if account.active { "  (active)" } else { "" };
        println!("  {marker} {}{active_tag}", account.id);
        crate::ui::kv("Email", &account.email);
        crate::ui::kv("Host", &account.host);
        crate::ui::kv("SSH alias", &account.alias);
        if let Some(desc) = &account.description {
            crate::ui::kv("About", desc);
        }
        match (&account.git_user_name, &account.git_user_email) {
            (Some(n), Some(e)) => crate::ui::kv("Git", format!("{n} <{e}>")),
            (Some(n), None) => crate::ui::kv("Git name", n),
            (None, Some(e)) => crate::ui::kv("Git email", e),
            _ => crate::ui::kv("Git", "(not set — attach will skip user.name/email)"),
        }

        let Some(full) = self.config.accounts.get(&account.id) else {
            return;
        };
        let key_path = self.ssh_dir.join(&full.key_file);
        let key_state = if key_path.exists() { "present" } else { "MISSING" };
        crate::ui::kv(
            "Key",
            format!("{} ({key_state})", crate::ui::path_home(&key_path)),
        );

        if verbose {
            let pub_path = key_path.with_extension("pub");
            if pub_path.exists() {
                crate::ui::kv("Public", crate::ui::path_home(&pub_path));
                if let Some(fp) = key_fingerprint(&pub_path) {
                    crate::ui::kv("Fingerprint", fp);
                }
            } else {
                crate::ui::kv("Public", "missing");
            }
            let ssh_config = self.ssh_dir.join("config");
            let in_config = ssh_config
                .exists()
                .then(|| fs::read_to_string(&ssh_config).ok())
                .flatten()
                .map(|c| c.contains(&format!("Host {}", account.alias)))
                .unwrap_or(false);
            crate::ui::kv(
                "ssh config",
                if in_config {
                    format!("Host {} ✓", account.alias)
                } else {
                    format!("no Host {} block", account.alias)
                },
            );
        }
    }

    pub fn doctor(&self) -> Result<DoctorReport> {
        let mut issues = 0usize;

        let git_version = Command::new("git")
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
        if git_version.is_none() {
            issues += 1;
        }

        let ssh_version = Command::new("ssh").arg("-V").output().ok().map(|o| {
            let msg = if o.stderr.is_empty() {
                String::from_utf8_lossy(&o.stdout).into_owned()
            } else {
                String::from_utf8_lossy(&o.stderr).into_owned()
            };
            msg.trim().to_string()
        });
        if ssh_version.is_none() {
            issues += 1;
        }

        let mut accounts = Vec::new();
        let mut names: Vec<String> = self.config.accounts.keys().cloned().collect();
        names.sort();
        for name in names {
            let account = self.config.accounts.get(&name).unwrap();
            let key_path = self.ssh_dir.join(&account.key_file);
            let pub_path = key_path.with_extension("pub");
            let key_present = key_path.exists();
            let pub_present = pub_path.exists();
            if !key_present {
                issues += 1;
            }
            if !pub_present {
                issues += 1;
            }
            accounts.push(DoctorAccount {
                id: name,
                key_present,
                pub_present,
                has_git_email: account.git_user_email.is_some(),
            });
        }

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let repo = match git::repo_root(&cwd)? {
            None => None,
            Some(root) => Some(RepoSnapshot {
                path: root.display().to_string(),
                origin: git::origin_url(&root),
                user_name: git::local_config(&root, "user.name"),
                user_email: git::local_config(&root, "user.email"),
                ssh_command: git::local_config(&root, "core.sshCommand"),
            }),
        };

        Ok(DoctorReport {
            ok: issues == 0,
            issues,
            git_version,
            ssh_version,
            config_path: self.config_path.display().to_string(),
            config_exists: self.config_path.exists(),
            accounts,
            repo,
        })
    }

    pub fn print_doctor(&self, report: &DoctorReport, verbose: bool) {
        println!();
        println!("🩺 GAM doctor");
        println!();
        println!("Environment");
        match &report.git_version {
            Some(v) => println!("   ✅ git  {v}"),
            None => println!("   ❌ git not found in PATH"),
        }
        match &report.ssh_version {
            Some(v) => println!("   ✅ ssh  {v}"),
            None => println!("   ❌ ssh not found in PATH"),
        }
        if report.config_exists {
            println!(
                "   ✅ config {}",
                crate::ui::path_home(Path::new(&report.config_path))
            );
        } else {
            println!(
                "   ⚠️  no config at {} (will be created on first add)",
                crate::ui::path_home(Path::new(&report.config_path))
            );
        }

        println!();
        println!("Accounts ({})", report.accounts.len());
        if report.accounts.is_empty() {
            crate::ui::hint("gam add");
        }
        for acc in &report.accounts {
            let mut flags = Vec::new();
            flags.push(if acc.key_present { "key ✓" } else { "key MISSING" });
            flags.push(if acc.pub_present { "pub ✓" } else { "pub MISSING" });
            if !acc.has_git_email {
                flags.push("no git email");
            }
            println!("   • {}  {}", acc.id, flags.join(" · "));
            if verbose {
                if let Some(full) = self
                    .accounts_view()
                    .into_iter()
                    .find(|a| a.id == acc.id)
                {
                    self.print_account_card(&full, true);
                    println!();
                }
            }
        }

        println!();
        println!("📁 Current directory");
        match &report.repo {
            None => {
                println!("   Not a git repository.");
                crate::ui::hint("cd into a repo, then gam attach");
            }
            Some(ctx) => {
                crate::ui::kv_indent("   ", "Root", crate::ui::path_home(Path::new(&ctx.path)));
                crate::ui::kv_indent(
                    "   ",
                    "Origin",
                    ctx.origin.as_deref().unwrap_or("(no origin)"),
                );
                crate::ui::kv_indent(
                    "   ",
                    "user.name",
                    ctx.user_name.as_deref().unwrap_or("(unset)"),
                );
                crate::ui::kv_indent(
                    "   ",
                    "user.email",
                    ctx.user_email.as_deref().unwrap_or("(unset)"),
                );
                crate::ui::kv_indent(
                    "   ",
                    "sshCommand",
                    ctx.ssh_command.as_deref().unwrap_or("(default ssh)"),
                );
            }
        }

        println!();
        if self.config.accounts.is_empty() {
            println!("ℹ️  No accounts yet. That is OK — run gam add when you are ready.");
        } else if report.issues == 0 {
            println!("✅ No problems found.");
        } else {
            println!(
                "⚠️  {} issue(s) found. See lines marked ❌ or MISSING.",
                report.issues
            );
            crate::ui::next_steps(&["gam list -v", "gam add", "docs/how-to/usar-varias-cuentas.md"]);
        }
    }

    pub fn validate_email(email: &str) -> bool {
        let email_regex = Regex::new(r"^[\w\.-]+@[\w\.-]+\.[a-zA-Z]{2,}$").unwrap();
        email_regex.is_match(email)
    }

    pub fn add_named(&mut self, req: AddAccountRequest) -> Result<AddAccountResult> {
        let name = req.name.trim().to_string();
        if name.is_empty() || name.contains('/') || name.contains('\\') {
            return Ok(AddAccountResult::error(
                "invalid_name",
                "Account name is empty or contains path separators.",
            ));
        }
        if !Self::validate_email(&req.email) {
            return Ok(AddAccountResult::error(
                "invalid_email",
                format!("Invalid email: {}", req.email),
            ));
        }
        let host = req.host.trim().to_string();
        if host.is_empty() {
            return Ok(AddAccountResult::error("invalid_host", "Host is required."));
        }

        if self.config.accounts.contains_key(&name) && !req.overwrite {
            return Ok(AddAccountResult::error(
                "account_exists",
                format!("Account '{name}' already exists. Pass overwrite=true to replace it."),
            ));
        }

        let git_user_name = req
            .git_user_name
            .filter(|s| !s.trim().is_empty())
            .or_else(|| Some(name.clone()));
        let git_user_email = req
            .git_user_email
            .filter(|s| !s.trim().is_empty())
            .or_else(|| Some(req.email.clone()));
        let description = req
            .description
            .filter(|s| !s.trim().is_empty());

        let key_file = format!("id_{}_{}", name.replace(' ', "_"), host.replace('.', "_"));
        let key_path = self.ssh_dir.join(&key_file);

        if (key_path.exists() || key_path.with_extension("pub").exists()) && !req.overwrite {
            return Ok(AddAccountResult::error(
                "key_exists",
                format!(
                    "Key {} already exists. Pass overwrite=true to replace it.",
                    key_path.display()
                ),
            ));
        }
        if req.overwrite {
            let _ = fs::remove_file(&key_path);
            let _ = fs::remove_file(key_path.with_extension("pub"));
            if let Some(old) = self.config.accounts.remove(&name) {
                let _ = ssh::remove_ssh_config_for_account(&self.ssh_dir, &old);
            }
        }

        let passphrase = if req.use_passphrase {
            match crate::passphrase::prompt_confirmed(&name) {
                Ok(p) => p,
                Err(crate::passphrase::PassphrasePromptError::Cancelled) => {
                    return Ok(AddAccountResult::error(
                        "cancelled",
                        "Passphrase prompt was cancelled.",
                    ));
                }
                Err(crate::passphrase::PassphrasePromptError::Unavailable(msg)) => {
                    return Ok(AddAccountResult::error(
                        "passphrase_prompt_unavailable",
                        msg,
                    ));
                }
                Err(e) => {
                    return Ok(AddAccountResult::error("passphrase_prompt_failed", e.to_string()));
                }
            }
        } else {
            req.passphrase.clone().unwrap_or_default()
        };
        let passphrase_protected = !passphrase.is_empty();
        let status = Command::new("ssh-keygen")
            .arg("-t")
            .arg("ed25519")
            .arg("-C")
            .arg(&req.email)
            .arg("-f")
            .arg(&key_path)
            .arg("-N")
            .arg(if passphrase_protected {
                passphrase.as_str()
            } else {
                ""
            })
            .arg("-q")
            .status()
            .context("Failed to run ssh-keygen")?;
        if !status.success() {
            return Err(anyhow::anyhow!("ssh-keygen failed"));
        }

        #[cfg(unix)]
        {
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&key_path, perms).context("Failed to set key permissions to 600")?;
        }

        let mut ssh_agent_loaded = false;
        let mut add_cmd = Command::new("ssh-add");
        if cfg!(target_os = "macos") && passphrase_protected {
            add_cmd.arg("--apple-use-keychain");
        }
        if let Ok(add_status) = add_cmd.arg(&key_path).status() {
            ssh_agent_loaded = add_status.success();
        }

        let account = SshAccount {
            name: name.clone(),
            email: req.email.clone(),
            key_file,
            host: host.clone(),
            description,
            git_user_name,
            git_user_email,
        };
        let alias = ssh::alias_for(&account);
        let mut ssh_config_updated = false;
        if req.update_ssh_config {
            ssh_config_updated = ssh::update_ssh_config(&self.ssh_dir, &account)?;
        }

        self.config.accounts.insert(name.clone(), account);
        self.save_config().context("Failed to save configuration")?;

        let public_key = fs::read_to_string(key_path.with_extension("pub"))
            .ok()
            .map(|s| s.trim().to_string());
        let add_key_url = add_key_url_for_host(&host);
        let suggested_remote = format!("git@{alias}:org/repo.git");
        let saved = self.config.accounts.get(&name).unwrap();
        let mut next_steps = vec![
            format!(
                "Add the public key on {host}{}",
                add_key_url
                    .as_deref()
                    .map(|u| format!(" ({u})"))
                    .unwrap_or_default()
            ),
            format!("In a repo: gam attach --account {name}"),
            format!("Optional: git remote set-url origin {suggested_remote}"),
        ];
        if !ssh_agent_loaded {
            next_steps.insert(
                0,
                format!("ssh-add {}", crate::ui::path_home(&key_path)),
            );
        }

        Ok(AddAccountResult {
            ok: true,
            account: Some(saved.public(false)),
            public_key,
            alias: Some(alias),
            suggested_remote: Some(suggested_remote),
            add_key_url,
            ssh_config_updated,
            ssh_agent_loaded,
            passphrase_protected,
            next_steps,
            code: None,
            message: Some(format!("Account '{name}' added.")),
        })
    }

    pub fn add_account(&mut self) -> Result<()> {
        println!("\n🔑 Adding a new SSH account\n");

        let name = Text::new("Account name (e.g., 'work', 'personal', 'github-work'):")
            .prompt()
            .context("Failed to get account name")?;

        let email = loop {
            let input = Text::new("Email address:")
                .prompt()
                .context("Failed to get email")?;

            if Self::validate_email(&input) {
                break input;
            } else {
                println!("❌ Please enter a valid email address");
            }
        };

        let host = Select::new(
            "Select the host type:",
            vec!["github.com", "gitlab.com", "bitbucket.org", "Custom"],
        )
        .prompt()
        .context("Failed to get host selection")?;

        let host = if host == "Custom" {
            Text::new("Enter custom host:")
                .prompt()
                .context("Failed to get custom host")?
        } else {
            host.to_string()
        };

        let description = Text::new("Description (optional):")
            .with_default("")
            .prompt()
            .context("Failed to get description")?;

        let git_user_name = Text::new("Git User Name (for git config user.name):")
            .with_default(&name)
            .prompt()
            .ok();

        let git_user_email = Text::new("Git User Email (for git config user.email):")
            .with_default(&email)
            .prompt()
            .ok();

        let use_passphrase = Confirm::new("Do you want to set a passphrase for this key?")
            .with_default(true)
            .prompt()
            .context("Failed to get passphrase confirmation")?;

        let passphrase = if use_passphrase {
            Some(
                Password::new("Enter passphrase for the SSH key:")
                    .without_confirmation()
                    .prompt()
                    .context("Failed to get passphrase")?,
            )
        } else {
            None
        };

        let update_ssh_config = Confirm::new("Do you want to update your SSH config file?")
            .with_default(true)
            .prompt()
            .context("Failed to get SSH config confirmation")?;

        let overwrite = if self.config.accounts.contains_key(&name) {
            Confirm::new(&format!("Account '{name}' already exists. Overwrite?"))
                .with_default(false)
                .prompt()?
        } else {
            let key_file = format!("id_{}_{}", name.replace(' ', "_"), host.replace('.', "_"));
            let key_path = self.ssh_dir.join(&key_file);
            if key_path.exists() || key_path.with_extension("pub").exists() {
                Confirm::new(&format!(
                    "Key {} already exists. Overwrite?",
                    key_path.display()
                ))
                .with_default(false)
                .prompt()?
            } else {
                false
            }
        };

        println!("\n🔄 Generating SSH key...");
        let result = self.add_named(AddAccountRequest {
            name: name.clone(),
            email,
            host: host.clone(),
            description: Some(description),
            git_user_name,
            git_user_email,
            overwrite,
            update_ssh_config,
            passphrase,
            use_passphrase: false,
        })?;

        if !result.ok {
            println!(
                "❌ {}",
                result.message.as_deref().unwrap_or("Could not add account")
            );
            return Ok(());
        }

        print_add_human(&result);

        if host == "github.com" {
            if let Some(url) = &result.add_key_url {
                let open_browser =
                    Confirm::new("Do you want to open GitHub settings to add this key now?")
                        .with_default(true)
                        .prompt()
                        .context("Failed to get browser confirmation")?;
                if open_browser {
                    println!("🌍 Opening {url} in your browser...");
                    if let Err(e) = open::that(url) {
                        println!("⚠️  Failed to open browser: {e}");
                        println!("Please visit {url} and paste the public key.");
                    }
                }
            }
        }
        Ok(())
    }

    pub fn switch_named(&mut self, account_id: &str) -> Result<ResolveResult> {
        if self.config.accounts.is_empty() {
            return Ok(ResolveResult::error(
                "no_accounts",
                "No accounts found. Use 'gam add' to create one.",
                Some(Vec::new()),
            ));
        }
        let Some(account) = self.config.accounts.get(account_id).cloned() else {
            return Ok(ResolveResult::error(
                "account_not_found",
                format!("Account '{account_id}' not found."),
                Some(self.accounts_view()),
            ));
        };

        self.config.current_account = Some(account_id.to_string());
        self.save_config().context("Failed to save configuration")?;
        let key_path = ssh::key_path(&self.ssh_dir, &account);
        ssh::upsert_active_mapping(&self.ssh_dir, &account.host, &key_path)?;

        Ok(ResolveResult {
            ok: true,
            account: Some(account.public(true)),
            project: None,
            source: Some(ResolveSource::Explicit),
            code: None,
            message: None,
            accounts: None,
        })
    }

    pub fn prompt_account_name(&self, prompt: &str) -> Result<String> {
        let mut account_names: Vec<String> = self.config.accounts.keys().cloned().collect();
        account_names.sort();
        Select::new(prompt, account_names)
            .prompt()
            .context("Failed to get account selection")
    }

    pub fn status(&self) -> Result<StatusView> {
        let Some(current) = &self.config.current_account else {
            return Ok(StatusView {
                ok: false,
                account: None,
                ssh_authenticated: None,
                code: Some("no_active_account".into()),
                message: Some("No active account set. Use 'gam switch' to select one.".into()),
            });
        };
        let Some(account) = self.config.accounts.get(current) else {
            return Ok(StatusView {
                ok: false,
                account: None,
                ssh_authenticated: None,
                code: Some("account_not_found".into()),
                message: Some(format!("Current account '{current}' not found in configuration")),
            });
        };

        let key_path = ssh::key_path(&self.ssh_dir, account);
        let ssh_authenticated = test_ssh(&key_path, &account.host);

        Ok(StatusView {
            ok: true,
            account: Some(account.public(true)),
            ssh_authenticated,
            code: None,
            message: None,
        })
    }

    pub fn remove_named(&mut self, account_id: &str) -> Result<bool> {
        let Some(account) = self.config.accounts.remove(account_id) else {
            return Ok(false);
        };

        let key_path = ssh::key_path(&self.ssh_dir, &account);
        let pub_key_path = format!("{}.pub", key_path.display());
        let _ = fs::remove_file(&key_path);
        let _ = fs::remove_file(&pub_key_path);
        ssh::remove_from_ssh_agent(&key_path);
        let _ = ssh::remove_ssh_config_for_account(&self.ssh_dir, &account);

        if Some(account_id) == self.config.current_account.as_deref() {
            self.config.current_account = None;
            let _ = ssh::clear_active_mapping_for_host(&self.ssh_dir, &account.host);
        }

        self.config
            .projects
            .retain(|p| p.account_id != account_id);

        self.save_config().context("Failed to save configuration")?;
        Ok(true)
    }

    pub fn reset_all(&mut self) -> Result<()> {
        let account_names: Vec<String> = self.config.accounts.keys().cloned().collect();
        for name in account_names {
            if let Some(account) = self.config.accounts.get(&name) {
                let key_path = ssh::key_path(&self.ssh_dir, account);
                let pub_key_path = format!("{}.pub", key_path.display());
                let _ = fs::remove_file(&key_path);
                let _ = fs::remove_file(&pub_key_path);
                ssh::remove_from_ssh_agent(&key_path);
                let _ = ssh::remove_ssh_config_for_account(&self.ssh_dir, account);
                let _ = ssh::clear_active_mapping_for_host(&self.ssh_dir, &account.host);
            }
        }
        self.config.accounts.clear();
        self.config.current_account = None;
        self.config.projects.clear();
        self.save_config().context("Failed to save empty configuration")?;
        Ok(())
    }

    pub fn ssh_config_text(&self) -> Result<Option<String>> {
        ssh::read_ssh_config(&self.ssh_dir)
    }

    pub fn resolve(
        &self,
        path: &Path,
        explicit_account: Option<&str>,
    ) -> Result<ResolveResult> {
        let Some(root) = git::repo_root(path)? else {
            return Ok(ResolveResult::error(
                "not_a_repo",
                format!("Not a git repository: {}", path.display()),
                None,
            ));
        };

        let origin = git::origin_url(&root);
        let project = ProjectPublic {
            path: root.display().to_string(),
            origin: origin.clone(),
            exists: true,
        };

        if let Some(id) = explicit_account {
            return Ok(self.result_for_account(id, project, ResolveSource::Explicit));
        }

        if let Some(file) = GamFile::read(&root)? {
            return Ok(self.result_for_account(&file.account, project, ResolveSource::GamFile));
        }

        if let Some(binding) = self.find_binding_by_path(&root) {
            return Ok(self.result_for_account(
                &binding.account_id.clone(),
                project,
                ResolveSource::LocalPath,
            ));
        }

        if let Some(origin_url) = origin.as_deref() {
            if let Some(binding) = self.find_binding_by_origin(origin_url) {
                return Ok(self.result_for_account(
                    &binding.account_id.clone(),
                    project,
                    ResolveSource::Origin,
                ));
            }
        }

        if let Some(email) = git::local_config(&root, "user.email") {
            if let Some((id, _)) = self.config.accounts.iter().find(|(_, a)| {
                a.git_user_email.as_deref() == Some(email.as_str()) || a.email == email
            }) {
                return Ok(self.result_for_account(id, project, ResolveSource::GitConfig));
            }
        }

        Ok(ResolveResult::error(
            "no_mapping",
            "No account mapped for this repository. Add a .gam.json or run 'gam attach --account <id>'.",
            Some(self.accounts_view()),
        ))
    }

    fn result_for_account(
        &self,
        account_id: &str,
        project: ProjectPublic,
        source: ResolveSource,
    ) -> ResolveResult {
        match self.config.accounts.get(account_id) {
            Some(account) => {
                let active = self.config.current_account.as_deref() == Some(account_id);
                ResolveResult::found(account.public(active), project, source)
            }
            None => ResolveResult::error(
                "account_not_found",
                format!("Account '{account_id}' is not configured on this machine."),
                Some(self.accounts_view()),
            ),
        }
    }

    fn find_binding_by_path(&self, root: &Path) -> Option<&ProjectBinding> {
        self.config
            .projects
            .iter()
            .find(|p| config::paths_eq(Path::new(&p.local_path), root))
    }

    fn find_binding_by_origin(&self, origin: &str) -> Option<&ProjectBinding> {
        let key = config::origin_repo_key(origin)?;
        self.config.projects.iter().find(|p| {
            p.origin
                .as_deref()
                .and_then(config::origin_repo_key)
                .as_deref()
                == Some(key.as_str())
        })
    }

    pub fn apply_identity(&self, repo: &Path, account: &SshAccount) -> Result<Vec<String>> {
        let mut applied = Vec::new();
        if let Some(name) = &account.git_user_name {
            git::set_local_config(repo, "user.name", name)?;
            applied.push("user.name".into());
        }
        if let Some(email) = &account.git_user_email {
            git::set_local_config(repo, "user.email", email)?;
            applied.push("user.email".into());
        }
        let key_path = ssh::key_path(&self.ssh_dir, account);
        let ssh_command = format!("ssh -i {} -o IdentitiesOnly=yes", key_path.display());
        git::set_local_config(repo, "core.sshCommand", &ssh_command)?;
        applied.push("core.sshCommand".into());
        Ok(applied)
    }

    fn upsert_binding(&mut self, repo: &Path, account_id: &str) {
        let origin = git::origin_url(repo);
        let path = repo
            .canonicalize()
            .unwrap_or_else(|_| repo.to_path_buf())
            .display()
            .to_string();
        if let Some(existing) = self
            .config
            .projects
            .iter_mut()
            .find(|p| config::paths_eq(Path::new(&p.local_path), repo))
        {
            existing.account_id = account_id.to_string();
            existing.origin = origin.or(existing.origin.clone());
            existing.local_path = path;
        } else {
            self.config.projects.push(ProjectBinding {
                local_path: path,
                account_id: account_id.to_string(),
                origin,
            });
        }
    }

    pub fn attach_named(&mut self, path: &Path, account_id: &str) -> Result<EnsureResult> {
        let resolved = self.resolve(path, Some(account_id))?;
        if !resolved.ok {
            return Ok(resolved.into());
        }
        let root = PathBuf::from(&resolved.project.as_ref().unwrap().path);
        let account = self.config.accounts.get(account_id).cloned().unwrap();
        let applied = self.apply_identity(&root, &account)?;
        self.upsert_binding(&root, account_id);
        GamFile::new(&account.name, ssh::alias_for(&account)).write(&root)?;
        self.save_config()?;

        let mut result = EnsureResult::from(self.resolve(&root, Some(account_id))?);
        result.applied = applied;
        result.applied.push(".gam.json".into());
        Ok(result)
    }

    pub fn ensure(&mut self, path: &Path, explicit_account: Option<&str>) -> Result<EnsureResult> {
        let resolved = self.resolve(path, explicit_account)?;
        if !resolved.ok {
            return Ok(resolved.into());
        }
        let account_id = resolved.account.as_ref().unwrap().id.clone();
        let root = PathBuf::from(&resolved.project.as_ref().unwrap().path);
        let source = resolved.source;
        let account = self.config.accounts.get(&account_id).cloned().unwrap();
        let applied = self.apply_identity(&root, &account)?;
        self.upsert_binding(&root, &account_id);
        self.save_config()?;

        Ok(EnsureResult {
            ok: true,
            account: resolved.account,
            project: resolved.project,
            source,
            applied,
            code: None,
            message: None,
            accounts: None,
        })
    }

    pub fn interactive_menu(&mut self) -> Result<()> {
        loop {
            let options = vec![
                "📝 Add new account",
                "📋 List accounts",
                "🔄 Switch account (legacy global)",
                "🔗 Attach to current repo",
                "🎯 Ensure identity for current repo",
                "📁 List projects",
                "📊 Show status",
                "🩺 Doctor (diagnose setup)",
                "📄 View SSH config",
                "🗑️  Remove account",
                "⚠️  Reset application",
                "🚪 Exit",
            ];

            let selection =
                Select::new("\n🔑 Git Account Manager (gam) - What would you like to do?", options)
                    .prompt()
                    .context("Failed to get menu selection")?;

            match selection {
                "📝 Add new account" => self.add_account()?,
                "📋 List accounts" => crate::output::print_accounts_human(&self.accounts_view()),
                "🔄 Switch account (legacy global)" => {
                    if self.config.accounts.is_empty() {
                        println!("📭 No accounts found. Use 'gam add' to create one.");
                    } else {
                        let selected = self.prompt_account_name("Select account to activate:")?;
                        let result = self.switch_named(&selected)?;
                        if result.ok {
                            println!("✅ Switched to account '{selected}'");
                        } else {
                            crate::output::print_resolve_human(&result);
                        }
                    }
                }
                "🔗 Attach to current repo" => {
                    if self.config.accounts.is_empty() {
                        println!("📭 No accounts found. Use 'gam add' to create one.");
                    } else {
                        let cwd = std::env::current_dir()?;
                        let selected =
                            self.prompt_account_name("Select account to attach to this repo:")?;
                        let result = self.attach_named(&cwd, &selected)?;
                        crate::output::print_ensure_human(&result);
                    }
                }
                "🎯 Ensure identity for current repo" => {
                    let cwd = std::env::current_dir()?;
                    let result = self.ensure(&cwd, None)?;
                    crate::output::print_ensure_human(&result);
                }
                "📁 List projects" => crate::output::print_projects_human(&self.config.projects),
                "📊 Show status" => print_status_human(&self.status()?),
                "🩺 Doctor (diagnose setup)" => {
                    let report = self.doctor()?;
                    self.print_doctor(&report, false);
                }
                "📄 View SSH config" => match self.ssh_config_text()? {
                    None => println!("📭 No SSH config file found."),
                    Some(content) => {
                        println!("\n📄 SSH config path: {}\n", self.ssh_dir.join("config").display());
                        println!("──────── BEGIN ~/.ssh/config ────────");
                        print!("{content}");
                        if !content.ends_with('\n') {
                            println!();
                        }
                        println!("────────  END ~/.ssh/config  ────────");
                    }
                },
                "🗑️  Remove account" => {
                    if self.config.accounts.is_empty() {
                        println!("📭 No accounts found.");
                    } else {
                        let selected = self.prompt_account_name("Select account to remove:")?;
                        let confirm = Confirm::new(&format!(
                            "Are you sure you want to remove account '{selected}'?"
                        ))
                        .with_default(false)
                        .prompt()
                        .context("Failed to get confirmation")?;
                        if !confirm {
                            println!("❌ Removal cancelled.");
                        } else if self.remove_named(&selected)? {
                            println!("✅ Account '{selected}' removed successfully!");
                        }
                    }
                }
                "⚠️  Reset application" => {
                    if self.config.accounts.is_empty() {
                        println!("📭 No accounts found. Nothing to reset.");
                    } else {
                        println!("\n⚠️  WARNING: You are about to DELETE ALL accounts and SSH keys managed by gam.");
                        println!("⚠️  This will also remove all gam entries from your ~/.ssh/config.");
                        println!("⚠️  This action CANNOT be undone.\n");
                        let confirm = Confirm::new("Are you sure you want to reset EVERYTHING?")
                            .with_default(false)
                            .prompt()
                            .context("Failed to get confirmation")?;
                        if !confirm {
                            println!("❌ Reset cancelled.");
                        } else {
                            self.reset_all()?;
                            println!("\n✅ Application reset successfully. All accounts and keys have been removed.");
                        }
                    }
                }
                "🚪 Exit" => {
                    println!("👋 Goodbye!");
                    break;
                }
                _ => unreachable!(),
            }

            println!("\nPress Enter to continue...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StatusView {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountPublic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_authenticated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DoctorAccount {
    pub id: String,
    pub key_present: bool,
    pub pub_present: bool,
    pub has_git_email: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RepoSnapshot {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_command: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub issues: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_version: Option<String>,
    pub config_path: String,
    pub config_exists: bool,
    pub accounts: Vec<DoctorAccount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<RepoSnapshot>,
}

#[derive(Debug, Clone, Default)]
pub struct AddAccountRequest {
    pub name: String,
    pub email: String,
    pub host: String,
    pub description: Option<String>,
    pub git_user_name: Option<String>,
    pub git_user_email: Option<String>,
    pub overwrite: bool,
    pub update_ssh_config: bool,
    /// Interactive CLI only: already-known secret. MCP never sets this.
    pub passphrase: Option<String>,
    /// Open a native OS dialog (macOS / Windows / Linux GUI) and confirm twice.
    pub use_passphrase: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AddAccountResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountPublic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_remote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_key_url: Option<String>,
    pub ssh_config_updated: bool,
    pub ssh_agent_loaded: bool,
    pub passphrase_protected: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl AddAccountResult {
    pub fn error(code: &str, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            account: None,
            public_key: None,
            alias: None,
            suggested_remote: None,
            add_key_url: None,
            ssh_config_updated: false,
            ssh_agent_loaded: false,
            passphrase_protected: false,
            next_steps: Vec::new(),
            code: Some(code.to_string()),
            message: Some(message.into()),
        }
    }
}

pub fn print_status_human(status: &StatusView) {
    if !status.ok {
        println!(
            "📭 {}",
            status.message.as_deref().unwrap_or("No active account")
        );
        return;
    }
    let account = status.account.as_ref().unwrap();
    println!(
        "\n🟢 Current active account: {} ({})",
        account.id, account.email
    );
    println!("   Host: {}  alias: {}", account.host, account.alias);
    if let Some(desc) = &account.description {
        println!("   Description: {desc}");
    }
    match status.ssh_authenticated {
        Some(true) => println!("✅ SSH connection successful!"),
        Some(false) => println!(
            "❌ SSH connection failed - key not added to {} or incorrect key",
            account.host
        ),
        None => {}
    }
}

pub fn print_add_human(result: &AddAccountResult) {
    if !result.ok {
        println!(
            "❌ {}",
            result.message.as_deref().unwrap_or("Could not add account")
        );
        return;
    }
    if let Some(account) = &result.account {
        println!("\n🎉 Account '{}' added ({})", account.id, account.email);
        println!("   Host: {}  alias: {}", account.host, account.alias);
    }
    if let Some(key) = &result.public_key {
        println!("\n📋 Public key (add this on the host — it is not secret):");
        println!("{key}");
    }
    if let Some(alias) = &result.alias {
        println!("🔗 Suggested remote: git@{alias}:org/repo.git");
    }
    if result.ssh_config_updated {
        println!("✅ SSH config updated");
    }
    if result.ssh_agent_loaded {
        println!("✅ Key loaded into ssh-agent");
    }
    if result.passphrase_protected {
        println!("🔒 Key is passphrase-protected (secret was never printed)");
    }
    if !result.next_steps.is_empty() {
        let steps: Vec<&str> = result.next_steps.iter().map(String::as_str).collect();
        crate::ui::next_steps(&steps);
    }
}

fn add_key_url_for_host(host: &str) -> Option<String> {
    match host {
        "github.com" => Some("https://github.com/settings/ssh/new".into()),
        "gitlab.com" => Some("https://gitlab.com/-/user_settings/ssh_keys".into()),
        "bitbucket.org" => Some("https://bitbucket.org/account/settings/ssh-keys/".into()),
        _ => None,
    }
}

fn test_ssh(key_path: &Path, host: &str) -> Option<bool> {
    let output = Command::new("ssh")
        .arg("-T")
        .arg("-i")
        .arg(key_path)
        .arg(format!("git@{host}"))
        .output()
        .ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("successfully authenticated") {
        Some(true)
    } else if stderr.contains("Permission denied") {
        Some(false)
    } else {
        None
    }
}

fn key_fingerprint(pub_path: &Path) -> Option<String> {
    let output = Command::new("ssh-keygen")
        .args(["-lf"])
        .arg(pub_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn fixture() -> (TempDir, SshManager) {
        let dir = TempDir::new().unwrap();
        let ssh_dir = dir.path().join("ssh");
        fs::create_dir_all(&ssh_dir).unwrap();
        let config_path = dir.path().join("gam_config.json");
        let mut mgr = SshManager::from_paths(config_path, ssh_dir).unwrap();
        mgr.config.accounts.insert(
            "work".into(),
            SshAccount {
                name: "work".into(),
                email: "work@example.com".into(),
                key_file: "id_work".into(),
                host: "github.com".into(),
                description: None,
                git_user_name: Some("Work User".into()),
                git_user_email: Some("work@example.com".into()),
            },
        );
        mgr.config.accounts.insert(
            "personal".into(),
            SshAccount {
                name: "personal".into(),
                email: "me@example.com".into(),
                key_file: "id_personal".into(),
                host: "github.com".into(),
                description: None,
                git_user_name: Some("Me".into()),
                git_user_email: Some("me@example.com".into()),
            },
        );
        mgr.save_config().unwrap();
        (dir, mgr)
    }

    fn init_repo(path: &Path) {
        fs::create_dir_all(path).unwrap();
        let status = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["-c", "init.defaultBranch=main", "init", "-q"])
            .status()
            .unwrap();
        assert!(status.success());
        let _ = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["config", "user.email", "tmp@example.com"])
            .status();
        let _ = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["config", "user.name", "tmp"])
            .status();
        let _ = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["remote", "add", "origin", "git@github.com:org/app.git"])
            .status();
    }

    #[test]
    fn resolve_prefers_gam_file_over_path_registry() {
        let (dir, mut mgr) = fixture();
        let repo = dir.path().join("repo");
        init_repo(&repo);
        let root = git::repo_root(&repo).unwrap().unwrap();
        mgr.config.projects.push(ProjectBinding {
            local_path: root.display().to_string(),
            account_id: "personal".into(),
            origin: Some("git@github.com:org/app.git".into()),
        });
        GamFile::new("work", "github-work").write(&root).unwrap();

        let result = mgr.resolve(&root, None).unwrap();
        assert!(result.ok);
        assert_eq!(result.source, Some(ResolveSource::GamFile));
        assert_eq!(result.account.unwrap().id, "work");
    }

    #[test]
    fn resolve_by_origin_when_path_differs() {
        let (dir, mut mgr) = fixture();
        let repo = dir.path().join("clone");
        init_repo(&repo);
        mgr.config.projects.push(ProjectBinding {
            local_path: dir.path().join("other-machine-path").display().to_string(),
            account_id: "work".into(),
            origin: Some("https://github.com/org/app.git".into()),
        });

        let result = mgr.resolve(&repo, None).unwrap();
        assert!(result.ok);
        assert_eq!(result.source, Some(ResolveSource::Origin));
        assert_eq!(result.account.unwrap().id, "work");
    }

    #[test]
    fn attach_writes_gam_file_and_registry() {
        let (dir, mut mgr) = fixture();
        let repo = dir.path().join("repo");
        init_repo(&repo);
        let result = mgr.attach_named(&repo, "work").unwrap();
        assert!(result.ok);
        assert!(result.applied.contains(&".gam.json".into()));
        let file = GamFile::read(&repo).unwrap().unwrap();
        assert_eq!(file.account, "work");
        assert_eq!(file.host_alias.as_deref(), Some("github-work"));
        assert!(mgr
            .config
            .projects
            .iter()
            .any(|p| p.account_id == "work"));
        let raw = fs::read_to_string(repo.join(".gam.json")).unwrap();
        assert!(!raw.contains("key_file"));
        assert!(!raw.contains("example.com"));
    }

    #[test]
    fn ensure_does_not_write_gam_file() {
        let (dir, mut mgr) = fixture();
        let repo = dir.path().join("repo");
        init_repo(&repo);
        mgr.config.projects.push(ProjectBinding {
            local_path: repo.canonicalize().unwrap().display().to_string(),
            account_id: "personal".into(),
            origin: None,
        });
        let result = mgr.ensure(&repo, None).unwrap();
        assert!(result.ok);
        assert!(!repo.join(".gam.json").exists());
        assert_eq!(git::local_config(&repo, "user.email").as_deref(), Some("me@example.com"));
    }

    #[test]
    fn add_named_creates_key_and_ssh_alias_without_private_key_in_json() {
        let dir = TempDir::new().unwrap();
        let ssh_dir = dir.path().join("ssh");
        fs::create_dir_all(&ssh_dir).unwrap();
        let mut mgr = SshManager::from_paths(dir.path().join("gam_config.json"), ssh_dir.clone()).unwrap();
        let result = mgr
            .add_named(AddAccountRequest {
                name: "agent-work".into(),
                email: "agent@example.com".into(),
                host: "github.com".into(),
                description: Some("from test".into()),
                git_user_name: Some("Agent".into()),
                git_user_email: Some("agent@example.com".into()),
                overwrite: false,
                update_ssh_config: true,
                passphrase: None,
                use_passphrase: false,
            })
            .unwrap();
        assert!(result.ok, "{result:?}");
        assert_eq!(result.account.as_ref().unwrap().id, "agent-work");
        assert_eq!(result.alias.as_deref(), Some("github-agent-work"));
        let pub_key = result.public_key.as_deref().unwrap();
        assert!(pub_key.starts_with("ssh-ed25519 "));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("BEGIN OPENSSH"));
        assert!(!json.contains("key_file"));
        assert!(ssh_dir.join("id_agent-work_github_com").exists());
        let ssh_config = fs::read_to_string(ssh_dir.join("config")).unwrap();
        assert!(ssh_config.contains("Host github-agent-work"));
        let dup = mgr
            .add_named(AddAccountRequest {
                name: "agent-work".into(),
                email: "agent@example.com".into(),
                host: "github.com".into(),
                overwrite: false,
                update_ssh_config: true,
                ..Default::default()
            })
            .unwrap();
        assert!(!dup.ok);
        assert_eq!(dup.code.as_deref(), Some("account_exists"));
    }

    #[test]
    fn add_named_rejects_bad_email() {
        let dir = TempDir::new().unwrap();
        let ssh_dir = dir.path().join("ssh");
        fs::create_dir_all(&ssh_dir).unwrap();
        let mut mgr = SshManager::from_paths(dir.path().join("gam_config.json"), ssh_dir).unwrap();
        let result = mgr
            .add_named(AddAccountRequest {
                name: "x".into(),
                email: "not-an-email".into(),
                host: "github.com".into(),
                update_ssh_config: false,
                ..Default::default()
            })
            .unwrap();
        assert!(!result.ok);
        assert_eq!(result.code.as_deref(), Some("invalid_email"));
    }
}
