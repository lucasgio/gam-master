mod ui;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use inquire::{Confirm, Password, Select, Text};
use regex::Regex;
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};
use open;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Parser, Debug)]
#[command(name = "gam-cli")]
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

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new SSH account and key
    #[command(after_help = "Examples:\n  gam-cli add\n\nThen add the printed public key to GitHub/GitLab and run `gam-cli attach` in a repo.")]
    Add,
    /// List configured accounts
    #[command(after_help = "Examples:\n  gam-cli list\n  gam-cli list -v")]
    List,
    /// Switch the global active account (legacy)
    #[command(after_help = "Examples:\n  gam-cli switch\n\nThis remaps Host github.com (or the account host) in ~/.ssh/config.\nPrefer `gam-cli attach` so each repo keeps its own identity.")]
    Switch,
    /// Remove an account and its key
    Remove,
    /// Show the global active account and this repo's git identity
    #[command(after_help = "Examples:\n  gam-cli status\n  gam-cli status -v")]
    Status,
    /// Delete every GAM account and managed SSH keys
    Reset,
    /// Bind the current git repo to an account (user.name/email + SSH key)
    #[command(after_help = "Examples:\n  cd /path/to/repo\n  gam-cli attach\n\nOptional next step:\n  git remote set-url origin git@<ssh-alias>:org/repo.git")]
    Attach,
    /// Diagnose SSH keys, config files and the current git identity
    #[command(after_help = "Examples:\n  gam-cli doctor\n  gam-cli doctor -v")]
    Doctor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SshAccount {
    name: String,
    email: String,
    key_file: String,
    host: String,
    description: Option<String>,
    #[serde(default)]
    git_user_name: Option<String>,
    #[serde(default)]
    git_user_email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    accounts: HashMap<String, SshAccount>,
    current_account: Option<String>,
}

struct SshManager {
    config_path: PathBuf,
    ssh_dir: PathBuf,
    config: Config,
    verbose: bool,
}

impl SshManager {
    fn new(verbose: bool) -> Result<Self> {
        let home_dir = home::home_dir().context("Could not find home directory")?;
        let ssh_dir = home_dir.join(".ssh");
        let new_config_path = ssh_dir.join("gam_config.json");
        let legacy_config_path = ssh_dir.join("ssh_manager_config.json");
        
        // Ensure .ssh directory exists
        if !ssh_dir.exists() {
            fs::create_dir_all(&ssh_dir).context("Failed to create .ssh directory")?;
        }
        
        // Load config (prefer new path, fallback to legacy)
        let (config, loaded_from_legacy) = if new_config_path.exists() {
            let content = fs::read_to_string(&new_config_path)
                .context("Failed to read config file")?;
            let cfg = serde_json::from_str(&content)
                .context("Failed to parse config file")?;
            (cfg, false)
        } else if legacy_config_path.exists() {
            let content = fs::read_to_string(&legacy_config_path)
                .context("Failed to read legacy config file")?;
            let cfg = serde_json::from_str(&content)
                .context("Failed to parse legacy config file")?;
            (cfg, true)
        } else {
            (Config::default(), false)
        };
        
        let manager = SshManager {
            config_path: new_config_path.clone(),
            ssh_dir,
            config,
            verbose,
        };

        if loaded_from_legacy {
            let _ = manager.save_config();
        }

        Ok(manager)
    }
    
    fn save_config(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.config)
            .context("Failed to serialize config")?;
        fs::write(&self.config_path, json)
            .context("Failed to write config file")?;
        Ok(())
    }
    
    fn validate_email(email: &str) -> bool {
        let email_regex = Regex::new(r"^[\w\.-]+@[\w\.-]+\.[a-zA-Z]{2,}$").unwrap();
        email_regex.is_match(email)
    }
    
    fn add_account(&mut self) -> Result<()> {
        println!("\n🔑 Adding a new SSH account\n");
        
        let name = Text::new("Account name (e.g., 'work', 'personal', 'github-work'):")
            .prompt()
            .context("Failed to get account name")?;
        
        if self.config.accounts.contains_key(&name) {
            println!("❌ Account '{}' already exists!", name);
            return Ok(());
        }
        
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
        
        let description = if description.is_empty() {
            None
        } else {
            Some(description)
        };
        
        // Git Identity (User Name & Email)
        let git_user_name = Text::new("Git User Name (for git config user.name):")
            .with_default(&name)
            .prompt()
            .ok(); 

        let git_user_name = if let Some(n) = git_user_name {
            if n.trim().is_empty() { None } else { Some(n) }
        } else {
            None
        };

        let git_user_email = Text::new("Git User Email (for git config user.email):")
            .with_default(&email)
            .prompt()
            .ok();

        let git_user_email = if let Some(e) = git_user_email {
            if e.trim().is_empty() { None } else { Some(e) }
        } else {
            None
        };
        
        // Ask for passphrase
        let use_passphrase = Confirm::new("Do you want to set a passphrase for this key?")
            .with_default(true)
            .prompt()
            .context("Failed to get passphrase confirmation")?;
        
        let passphrase = if use_passphrase {
            Some(Password::new("Enter passphrase for the SSH key:")
                .without_confirmation()
                .prompt()
                .context("Failed to get passphrase")?)
        } else {
            None
        };
        

        

        // Generate SSH key
        let key_file = format!("id_{}_{}", name.replace(" ", "_"), host.replace(".", "_"));
        let key_path = self.ssh_dir.join(&key_file);

        // Handle overwrite if key already exists
        if key_path.exists() || key_path.with_extension("pub").exists() {
            let overwrite = Confirm::new(&format!(
                "Key {} already exists. Overwrite?",
                key_path.display()
            ))
            .with_default(false)
            .prompt()
            .context("Failed to confirm overwrite")?;

            if !overwrite {
                println!("❌ Key generation cancelled.");
                return Ok(());
            }

            let _ = fs::remove_file(&key_path);
            let _ = fs::remove_file(key_path.with_extension("pub"));
        }

        println!("\n🔄 Generating SSH key...");

        let status = Command::new("ssh-keygen")
            .arg("-t")
            .arg("ed25519")
            .arg("-C")
            .arg(&email)
            .arg("-f")
            .arg(&key_path)
            .arg("-N")
            .arg(passphrase.as_deref().unwrap_or(""))
            .arg("-q")
            .status()
            .context("Failed to run ssh-keygen")?;

        if !status.success() {
            return Err(anyhow::anyhow!("ssh-keygen failed"));
        }

        // Ensure private key permissions are 600 on Unix systems
        #[cfg(unix)]
        {
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&key_path, perms).context("Failed to set key permissions to 600")?;
        }
        
        println!("✅ SSH key generated successfully!");
        
        // Add to ssh-agent (and Keychain on macOS when applicable)
        println!("🔄 Adding key to ssh-agent...");
        let mut add_cmd = Command::new("ssh-add");
        if cfg!(target_os = "macos") && passphrase.is_some() {
            add_cmd.arg("--apple-use-keychain");
        }
        let add_status = add_cmd
            .arg(&key_path)
            .status()
            .context("Failed to add key to ssh-agent")?;
        if add_status.success() {
            if cfg!(target_os = "macos") && passphrase.is_some() {
                println!("✅ Key added to ssh-agent and keychain!");
            } else {
                println!("✅ Key added to ssh-agent!");
            }
        }
        
        // Create account
        let account = SshAccount {
            name: name.clone(),
            email,
            key_file,
            host: host.clone(),
            description,
            git_user_name,
            git_user_email,
        };
        
        self.config.accounts.insert(name.clone(), account);
        self.save_config().context("Failed to save configuration")?;
        
        // Show public key
        let pub_key_path = format!("{}.pub", key_path.display());
        if let Ok(pub_key) = fs::read_to_string(&pub_key_path) {
            println!("\n📋 Your public key (copy this to {}):", host);
            println!("{}", pub_key.trim());
            println!(
                "\n🔗 Suggested SSH alias: {} (use git@{}:org/repo.git)",
                Self::alias_for(self.config.accounts.get(&name).unwrap()),
                Self::alias_for(self.config.accounts.get(&name).unwrap())
            );

            // Open GitHub settings if host is github.com
            if host == "github.com" {
                let open_browser = Confirm::new("Do you want to open GitHub settings to add this key now?")
                    .with_default(true)
                    .prompt()
                    .context("Failed to get browser confirmation")?;
                
                if open_browser {
                    println!("🌍 Opening https://github.com/settings/ssh/new in your browser...");
                    if let Err(e) = open::that("https://github.com/settings/ssh/new") {
                        println!("⚠️  Failed to open browser: {}", e);
                        println!("Please manually visit https://github.com/settings/ssh/new and paste your key.");
                    } else {
                        println!("✅ Browser opened! Paste the key above into the 'Key' field.");
                        println!("💡 Tip: Give it a Title like '{}'", name);
                    }
                }
            }
        }
        
        // Ask if they want to update SSH config
        let update_config = Confirm::new("Do you want to update your SSH config file?")
            .with_default(true)
            .prompt()
            .context("Failed to get SSH config confirmation")?;
        
        if update_config {
            self.update_ssh_config(&name)?;
        }
        
        println!("\n🎉 Account '{name}' added successfully!");
        println!();
        println!("  Config:   {}", ui::path_home(&self.config_path));
        println!("  Key:      {}", ui::path_home(&key_path));
        println!(
            "  SSH alias git@{}:org/repo.git",
            Self::alias_for(self.config.accounts.get(&name).unwrap())
        );
        ui::next_steps(&[
            "Add the public key on the host (GitHub → Settings → SSH keys)",
            "In a repository run: gam-cli attach",
            "Optional: git remote set-url origin git@<alias>:org/repo.git",
        ]);
        Ok(())
    }
    
    fn update_ssh_config(&self, account_name: &str) -> Result<()> {
        let account = self.config.accounts.get(account_name)
            .context("Account not found")?;
        
        let ssh_config_path = self.ssh_dir.join("config");
        let key_path = self.ssh_dir.join(&account.key_file);
        
        // Create a per-account alias to avoid conflicts for the same host
        let alias = Self::alias_for(account);
        let host_config = format!(
            "\n# {} - {}\nHost {}\n    HostName {}\n    User git\n    IdentityFile {}\n    AddKeysToAgent yes\n    UseKeychain yes\n    IdentitiesOnly yes\n",
            account.name,
            account.description.as_deref().unwrap_or(&account.email),
            alias,
            account.host,
            key_path.display()
        );
        
        let current_config = if ssh_config_path.exists() {
            fs::read_to_string(&ssh_config_path)
                .context("Failed to read SSH config")?
        } else {
            String::new()
        };
        
        // Check if this host is already configured
        let host_marker = format!("# {} - ", account.name);
        if current_config.contains(&host_marker) {
            println!("ℹ️  SSH config for '{}' already exists, skipping...", account.name);
            return Ok(());
        }
        
        let updated_config = current_config + &host_config;
        
        fs::write(&ssh_config_path, updated_config)
            .context("Failed to write SSH config")?;
        
        println!("✅ SSH config updated!");
        Ok(())
    }

    // Compute a unique alias per account, e.g., "github-work"
    fn alias_for(account: &SshAccount) -> String {
        let host_prefix = account
            .host
            .split('.')
            .next()
            .unwrap_or(&account.host)
            .to_string();
        let name_part = account.name.replace(' ', "-");
        format!("{}-{}", host_prefix, name_part)
    }

    // Ensure an "active" mapping for the given host to use the provided key
    fn upsert_active_mapping(&self, host: &str, key_path: &PathBuf) -> Result<()> {
        let ssh_config_path = self.ssh_dir.join("config");
        let mut current_config = if ssh_config_path.exists() {
            fs::read_to_string(&ssh_config_path).context("Failed to read SSH config")?
        } else {
            String::new()
        };

        let start_marker_new = format!("# gam ACTIVE START [{}]\n", host);
        let end_marker_new = format!("# gam ACTIVE END [{}]\n", host);
        let start_marker_old = format!("# ssh-manager ACTIVE START [{}]\n", host);
        let end_marker_old = format!("# ssh-manager ACTIVE END [{}]\n", host);

        let mut block = String::new();
        block.push_str(&start_marker_new);
        block.push_str(&format!(
            "Host {}\n    HostName {}\n    User git\n    IdentityFile {}\n    AddKeysToAgent yes\n    UseKeychain yes\n    IdentitiesOnly yes\n",
            host,
            host,
            key_path.display()
        ));
        block.push_str(&end_marker_new);

        let (start_marker, end_marker) = if current_config.contains(&start_marker_new) { (start_marker_new.clone(), end_marker_new.clone()) } else { (start_marker_old.clone(), end_marker_old.clone()) };
        if let Some(start_idx) = current_config.find(&start_marker) {
            if let Some(after_start) = current_config.get(start_idx + start_marker.len()..) {
                if let Some(end_rel_idx) = after_start.find(&end_marker) {
                    let end_idx = start_idx + start_marker.len() + end_rel_idx + end_marker.len();
                    let mut new_config = String::with_capacity(current_config.len() + block.len());
                    new_config.push_str(&current_config[..start_idx]);
                    new_config.push_str(&block);
                    new_config.push_str(&current_config[end_idx..]);
                    current_config = new_config;
                } else {
                    // Start found but no end; replace from start with block
                    let mut new_config = String::with_capacity(current_config.len() + block.len());
                    new_config.push_str(&current_config[..start_idx]);
                    new_config.push_str(&block);
                    current_config = new_config;
                }
            }
        } else {
            if !current_config.ends_with('\n') && !current_config.is_empty() {
                current_config.push('\n');
            }
            current_config.push_str(&block);
        }

        fs::write(&ssh_config_path, current_config).context("Failed to write SSH config")?;
        println!("✅ Active SSH mapping updated for {}", host);
        Ok(())
    }

    fn clear_active_mapping_for_host(&self, host: &str) -> Result<()> {
        let ssh_config_path = self.ssh_dir.join("config");
        if !ssh_config_path.exists() {
            return Ok(());
        }

        let mut current_config = fs::read_to_string(&ssh_config_path).context("Failed to read SSH config")?;
        let markers = [
            (format!("# gam ACTIVE START [{}]\n", host), format!("# gam ACTIVE END [{}]\n", host)),
            (format!("# ssh-manager ACTIVE START [{}]\n", host), format!("# ssh-manager ACTIVE END [{}]\n", host)),
        ];
        for (start_marker, end_marker) in markers {
            if let Some(start_idx) = current_config.find(&start_marker) {
                if let Some(after_start) = current_config.get(start_idx + start_marker.len()..) {
                    if let Some(end_rel_idx) = after_start.find(&end_marker) {
                        let end_idx = start_idx + start_marker.len() + end_rel_idx + end_marker.len();
                        let mut new_config = String::with_capacity(current_config.len());
                        new_config.push_str(&current_config[..start_idx]);
                        new_config.push_str(&current_config[end_idx..]);
                        current_config = new_config;
                    }
                }
            }
        }
        fs::write(&ssh_config_path, current_config).context("Failed to write SSH config")?;
        println!("ℹ️  Active SSH mapping cleared for {}", host);
        Ok(())
    }

    fn remove_from_ssh_agent(&self, key_path: &PathBuf) {
        let _ = Command::new("ssh-add").arg("-d").arg(key_path).status();
    }

    fn remove_ssh_config_for_account(&self, account: &SshAccount) -> Result<()> {
        let ssh_config_path = self.ssh_dir.join("config");
        if !ssh_config_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&ssh_config_path)
            .context("Failed to read SSH config")?;

        let alias = Self::alias_for(account);
        let name_escaped = regex::escape(&account.name);
        let alias_escaped = regex::escape(&alias);

        // Remove block that starts with our comment header and includes the alias Host block
        let header_pattern = format!(
            r"(?m)\n?# {}\s-\s[^\n]*\nHost {}\n(?:[ \t].*\n)*",
            name_escaped, alias_escaped
        );
        let re_header = Regex::new(&header_pattern).context("Failed to compile regex")?;
        let after_header = re_header.replace_all(&content, "");

        // Fallback: remove a Host-alias block without the header if present
        let host_pattern = format!(r"(?m)^\n?Host {}\n(?:[ \t].*\n)*", alias_escaped);
        let re_host = Regex::new(&host_pattern).context("Failed to compile regex")?;
        let new_content = re_host.replace_all(&after_header, "");

        if new_content != content {
            fs::write(&ssh_config_path, new_content.as_ref())
                .context("Failed to write SSH config")?;
            println!("✅ SSH config entry for '{}' removed.", alias);
        } else {
            println!("ℹ️  No SSH config entry found for '{}' (nothing to remove).", alias);
        }
        Ok(())
    }
    
    fn list_accounts(&self) -> Result<()> {
        if self.config.accounts.is_empty() {
            ui::empty_state(
                "No accounts found.",
                &["gam-cli add     Create your first SSH identity"],
            );
            return Ok(());
        }

        let mut names: Vec<String> = self.config.accounts.keys().cloned().collect();
        names.sort();

        println!();
        println!("📋 {} account(s)  ·  config {}", names.len(), ui::path_home(&self.config_path));
        println!();

        for name in names {
            let account = self.config.accounts.get(&name).unwrap();
            self.print_account_card(&name, account);
            println!();
        }

        if !self.verbose {
            ui::hint("gam-cli list -v    fingerprints, public key path and SSH config");
        }
        ui::hint("gam-cli attach     bind the current git repo to one of these accounts");
        Ok(())
    }

    fn print_account_card(&self, name: &str, account: &SshAccount) {
        let active = Some(name) == self.config.current_account.as_deref();
        let marker = if active { "🟢" } else { "⚪" };
        let active_tag = if active { "  (active)" } else { "" };
        println!("  {marker} {name}{active_tag}");
        ui::kv("Email", &account.email);
        ui::kv("Host", &account.host);
        ui::kv("SSH alias", Self::alias_for(account));
        if let Some(desc) = &account.description {
            ui::kv("About", desc);
        }
        match (&account.git_user_name, &account.git_user_email) {
            (Some(n), Some(e)) => ui::kv("Git", format!("{n} <{e}>")),
            (Some(n), None) => ui::kv("Git name", n),
            (None, Some(e)) => ui::kv("Git email", e),
            _ => ui::kv("Git", "(not set — attach will skip user.name/email)"),
        }

        let key_path = self.ssh_dir.join(&account.key_file);
        let key_state = if key_path.exists() {
            "present"
        } else {
            "MISSING"
        };
        ui::kv("Key", format!("{} ({key_state})", ui::path_home(&key_path)));

        if self.verbose {
            let pub_path = key_path.with_extension("pub");
            if pub_path.exists() {
                ui::kv("Public", ui::path_home(&pub_path));
                if let Some(fp) = key_fingerprint(&pub_path) {
                    ui::kv("Fingerprint", fp);
                }
            } else {
                ui::kv("Public", "missing");
            }
            let ssh_config = self.ssh_dir.join("config");
            let alias = Self::alias_for(account);
            let in_config = ssh_config
                .exists()
                .then(|| fs::read_to_string(&ssh_config).ok())
                .flatten()
                .map(|c| c.contains(&format!("Host {alias}")))
                .unwrap_or(false);
            ui::kv(
                "ssh config",
                if in_config {
                    format!("Host {alias} ✓")
                } else {
                    format!("no Host {alias} block")
                },
            );
        }
    }
    
    fn switch_account(&mut self) -> Result<()> {
        if self.config.accounts.is_empty() {
            ui::empty_state(
                "No accounts found.",
                &["gam-cli add     Create your first SSH identity"],
            );
            return Ok(());
        }
        
        let account_names: Vec<String> = self.config.accounts.keys().cloned().collect();
        
        let selected = Select::new("Select account to activate:", account_names)
            .prompt()
            .context("Failed to get account selection")?;
        
        self.config.current_account = Some(selected.clone());
        self.save_config().context("Failed to save configuration")?;

        // Update active host mapping to point host -> selected account key
        if let Some(account) = self.config.accounts.get(&selected) {
            let key_path = self.ssh_dir.join(&account.key_file);
            self.upsert_active_mapping(&account.host, &key_path)?;
        }
        
        println!("✅ Switched to account '{selected}'");
        if let Some(account) = self.config.accounts.get(&selected) {
            println!(
                "   Host {} now uses {} in ~/.ssh/config (legacy global mapping).",
                account.host,
                account.key_file
            );
            ui::hint("This is global. For one repo only, use gam-cli attach.");
        }
        Ok(())
    }
    
    fn show_status(&self) -> Result<()> {
        println!();
        println!("📊 GAM status");
        ui::kv_indent("   ", "Config", ui::path_home(&self.config_path));
        ui::kv_indent("   ", "Accounts", self.config.accounts.len().to_string());

        match self.config.current_account.as_ref() {
            None => {
                println!();
                ui::empty_state(
                    "No global active account.",
                    &[
                        "gam-cli switch    Set the legacy global mapping",
                        "gam-cli attach    Preferred: identity for this repo only",
                    ],
                );
            }
            Some(current) => match self.config.accounts.get(current) {
                None => {
                    println!();
                    println!("❌ Current account '{current}' is missing from config.");
                    ui::next_steps(&["gam-cli list", "gam-cli switch"]);
                }
                Some(account) => {
                    println!();
                    println!("🟢 Active account: {current}");
                    self.print_account_card(current, account);
                    println!();
                    println!("🔄 SSH test (git@{})...", account.host);
                    self.print_ssh_test(account);
                    if !self.verbose {
                        ui::hint("gam-cli status -v    fingerprints and extra SSH config detail");
                    }
                }
            },
        }

        self.print_repo_status()?;
        Ok(())
    }

    fn print_ssh_test(&self, account: &SshAccount) {
        let key_path = self.ssh_dir.join(&account.key_file);
        if !key_path.exists() {
            println!("   ❌ Private key missing: {}", ui::path_home(&key_path));
            return;
        }
        let output = Command::new("ssh")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("StrictHostKeyChecking=accept-new")
            .arg("-o")
            .arg("ConnectTimeout=8")
            .arg("-T")
            .arg("-i")
            .arg(&key_path)
            .arg(format!("git@{}", account.host))
            .output();

        match output {
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                if stderr.contains("successfully authenticated") {
                    println!("   ✅ SSH authentication succeeded");
                } else if stderr.contains("Permission denied") {
                    println!(
                        "   ❌ Permission denied — add the public key on {}",
                        account.host
                    );
                    ui::hint("gam-cli list -v    copy the public key path");
                } else {
                    println!("   ℹ️  {}", stderr.trim());
                }
            }
            Err(e) => println!("   ⚠️  Could not run ssh: {e}"),
        }
    }

    fn print_repo_status(&self) -> Result<()> {
        println!();
        println!("📁 Current directory");
        match repo_context()? {
            None => {
                println!("   Not a git repository.");
                ui::hint("cd into a repo, then gam-cli attach");
            }
            Some(ctx) => {
                ui::kv_indent("   ", "Root", ui::path_home(&ctx.root));
                ui::kv_indent(
                    "   ",
                    "Origin",
                    ctx.origin.as_deref().unwrap_or("(no origin)"),
                );
                ui::kv_indent(
                    "   ",
                    "user.name",
                    ctx.user_name.as_deref().unwrap_or("(unset)"),
                );
                ui::kv_indent(
                    "   ",
                    "user.email",
                    ctx.user_email.as_deref().unwrap_or("(unset)"),
                );
                ui::kv_indent(
                    "   ",
                    "sshCommand",
                    ctx.ssh_command.as_deref().unwrap_or("(default ssh)"),
                );
                if ctx.user_name.is_none() && ctx.user_email.is_none() {
                    ui::hint("gam-cli attach    set this repo's Git identity");
                }
            }
        }
        Ok(())
    }
    
    fn remove_account(&mut self) -> Result<()> {
        if self.config.accounts.is_empty() {
            ui::empty_state(
                "No accounts found.",
                &["gam-cli add     Create your first SSH identity"],
            );
            return Ok(());
        }
        
        let account_names: Vec<String> = self.config.accounts.keys().cloned().collect();
        
        let selected = Select::new("Select account to remove:", account_names)
            .prompt()
            .context("Failed to get account selection")?;
        
        let confirm = Confirm::new(&format!("Are you sure you want to remove account '{}'?", selected))
            .with_default(false)
            .prompt()
            .context("Failed to get confirmation")?;
        
        if !confirm {
            println!("❌ Removal cancelled.");
            return Ok(());
        }
        
        if let Some(account) = self.config.accounts.remove(&selected) {
            // Remove key files if they exist
            let key_path = self.ssh_dir.join(&account.key_file);
            let pub_key_path = format!("{}.pub", key_path.display());
            
            let _ = fs::remove_file(&key_path);
            let _ = fs::remove_file(&pub_key_path);

            // Remove from ssh-agent if loaded
            self.remove_from_ssh_agent(&key_path);

            // Remove this account's alias block from ~/.ssh/config
            let _ = self.remove_ssh_config_for_account(&account);
            
            // Remove from current account if it was active
            if Some(&selected) == self.config.current_account.as_ref() {
                self.config.current_account = None;
                // Clear active mapping for this host
                let _ = self.clear_active_mapping_for_host(&account.host);
            }
            
            self.save_config().context("Failed to save configuration")?;
            
            println!("✅ Account '{}' removed successfully!", selected);
            println!("ℹ️  Note: SSH config entries need to be manually removed if desired.");
        }
        
        Ok(())
    }
    
    fn reset_application(&mut self) -> Result<()> {
        if self.config.accounts.is_empty() {
            println!("📭 No accounts found. Nothing to reset.");
            return Ok(());
        }

        println!("\n⚠️  WARNING: You are about to DELETE ALL accounts and SSH keys managed by gam.");
        println!("⚠️  This will also remove all gam entries from your ~/.ssh/config.");
        println!("⚠️  This action CANNOT be undone.\n");

        let confirm = Confirm::new("Are you sure you want to reset EVERYTHING?")
            .with_default(false)
            .prompt()
            .context("Failed to get confirmation")?;

        if !confirm {
            println!("❌ Reset cancelled.");
            return Ok(());
        }

        println!("\n🗑️  Deleting accounts and keys...");

        // Iterate over all accounts to remove keys and config entries
        // We clone the keys to iterate while modifying
        let account_names: Vec<String> = self.config.accounts.keys().cloned().collect();

        for name in account_names {
            if let Some(account) = self.config.accounts.get(&name) {
                // Remove keys
                let key_path = self.ssh_dir.join(&account.key_file);
                let pub_key_path = format!("{}.pub", key_path.display());
                
                if key_path.exists() {
                    let _ = fs::remove_file(&key_path);
                    println!("   Deleted key: {}", key_path.display());
                }
                if PathBuf::from(&pub_key_path).exists() {
                     let _ = fs::remove_file(&pub_key_path);
                }

                // Remove from agent
                self.remove_from_ssh_agent(&key_path);

                // Remove from ssh config (Host block)
                let _ = self.remove_ssh_config_for_account(&account);

                // Remove from active mapping (Active block)
                 let _ = self.clear_active_mapping_for_host(&account.host);
            }
        }

        // Clear config
        self.config.accounts.clear();
        self.config.current_account = None;
        self.save_config().context("Failed to save empty configuration")?;

        println!("\n✅ Application reset successfully. All accounts and keys have been removed.");
        Ok(())
    }

    fn view_ssh_config(&self) -> Result<()> {
        let ssh_config_path = self.ssh_dir.join("config");
        println!("\n📄 SSH config path: {}\n", ssh_config_path.display());
        if !ssh_config_path.exists() {
            println!("📭 No SSH config file found.");
            return Ok(());
        }
        let content = fs::read_to_string(&ssh_config_path)
            .context("Failed to read SSH config")?;
        println!("──────── BEGIN ~/.ssh/config ────────");
        print!("{}", content);
        if !content.ends_with('\n') {
            println!();
        }
        println!("────────  END ~/.ssh/config  ────────");
        Ok(())
    }
    
    fn attach_repo(&self) -> Result<()> {
        match repo_context()? {
            None => {
                println!("❌ Current directory is not a git repository.");
                ui::next_steps(&[
                    "cd /path/to/your/repo",
                    "gam-cli attach",
                ]);
                return Ok(());
            }
            Some(ctx) => {
                if self.config.accounts.is_empty() {
                    ui::empty_state(
                        "No accounts found.",
                        &["gam-cli add     Create an identity first"],
                    );
                    return Ok(());
                }

                println!();
                println!("📁 Repository {}", ui::path_home(&ctx.root));
                ui::kv_indent(
                    "   ",
                    "Origin",
                    ctx.origin.as_deref().unwrap_or("(no origin)"),
                );
                ui::kv_indent(
                    "   ",
                    "Now",
                    format!(
                        "{} <{}>",
                        ctx.user_name.as_deref().unwrap_or("unset"),
                        ctx.user_email.as_deref().unwrap_or("unset")
                    ),
                );

                let mut account_names: Vec<String> = self.config.accounts.keys().cloned().collect();
                account_names.sort();

                let selected = Select::new("Select account to attach to this repo:", account_names)
                    .prompt()
                    .context("Failed to get account selection")?;

                let Some(account) = self.config.accounts.get(&selected) else {
                    return Ok(());
                };

                println!();
                println!("🔗 Attaching '{selected}'…");

                if account.git_user_name.is_none() && account.git_user_email.is_none() {
                    println!("   ⚠️  This account has no git user.name/email. Only SSH will be set.");
                    ui::hint("Edit ~/.ssh/gam_config.json or re-add the account to store Git identity.");
                }

                if let Some(name) = &account.git_user_name {
                    Command::new("git")
                        .args(["config", "--local", "user.name", name])
                        .status()
                        .context("Failed to set user.name")?;
                    println!("   user.name        = {name}");
                }

                if let Some(email) = &account.git_user_email {
                    Command::new("git")
                        .args(["config", "--local", "user.email", email])
                        .status()
                        .context("Failed to set user.email")?;
                    println!("   user.email       = {email}");
                }

                let key_path = self.ssh_dir.join(&account.key_file);
                if !key_path.exists() {
                    println!(
                        "   ⚠️  Key {} is missing. SSH will fail until the file exists.",
                        ui::path_home(&key_path)
                    );
                }
                let ssh_command = format!("ssh -i {} -o IdentitiesOnly=yes", key_path.display());
                Command::new("git")
                    .args(["config", "--local", "core.sshCommand", &ssh_command])
                    .status()
                    .context("Failed to set core.sshCommand")?;
                println!("   core.sshCommand  = {}", account.key_file);

                let alias = Self::alias_for(account);
                println!();
                println!("✅ Repository uses account '{selected}'.");
                let mut steps = vec![
                    "git commit / git fetch to verify the identity",
                ];
                if ctx.origin.is_some() {
                    steps.push("optional: git remote set-url origin git@ALIAS:org/repo.git");
                }
                ui::next_steps(&steps);
                println!("   Replace ALIAS with: {alias}");
            }
        }

        Ok(())
    }

    fn doctor(&self) -> Result<()> {
        println!();
        println!("🩺 GAM doctor");
        let mut issues = 0usize;

        println!();
        println!("Environment");
        match Command::new("git").arg("--version").output() {
            Ok(o) if o.status.success() => {
                println!(
                    "   ✅ git  {}",
                    String::from_utf8_lossy(&o.stdout).trim()
                );
            }
            _ => {
                println!("   ❌ git not found in PATH");
                issues += 1;
            }
        }
        match Command::new("ssh").arg("-V").output() {
            Ok(o) => {
                let msg = if o.stderr.is_empty() {
                    String::from_utf8_lossy(&o.stdout)
                } else {
                    String::from_utf8_lossy(&o.stderr)
                };
                println!("   ✅ ssh  {}", msg.trim());
            }
            _ => {
                println!("   ❌ ssh not found in PATH");
                issues += 1;
            }
        }
        if self.config_path.exists() {
            println!("   ✅ config {}", ui::path_home(&self.config_path));
        } else {
            println!(
                "   ⚠️  no config at {} (will be created on first add)",
                ui::path_home(&self.config_path)
            );
        }

        println!();
        println!("Accounts ({})", self.config.accounts.len());
        if self.config.accounts.is_empty() {
            ui::hint("gam-cli add");
        }
        let mut names: Vec<String> = self.config.accounts.keys().cloned().collect();
        names.sort();
        for name in &names {
            let account = self.config.accounts.get(name).unwrap();
            let key_path = self.ssh_dir.join(&account.key_file);
            let pub_path = key_path.with_extension("pub");
            let mut flags = Vec::new();
            if key_path.exists() {
                flags.push("key ✓");
            } else {
                flags.push("key MISSING");
                issues += 1;
            }
            if pub_path.exists() {
                flags.push("pub ✓");
            } else {
                flags.push("pub MISSING");
                issues += 1;
            }
            if account.git_user_email.is_none() {
                flags.push("no git email");
            }
            println!("   • {name}  {}", flags.join(" · "));
            if self.verbose {
                self.print_account_card(name, account);
                println!();
            }
        }

        println!();
        self.print_repo_status()?;

        println!();
        if self.config.accounts.is_empty() {
            println!("ℹ️  No accounts yet. That is OK — run gam-cli add when you are ready.");
        } else if issues == 0 {
            println!("✅ No problems found.");
        } else {
            println!("⚠️  {issues} issue(s) found. See lines marked ❌ or MISSING.");
            ui::next_steps(&["gam-cli list -v", "gam-cli add", "docs/how-to/usar-varias-cuentas.md"]);
        }
        Ok(())
    }

    fn interactive_menu(&mut self) -> Result<()> {
        loop {
            let options = vec![
                "📝 Add new account",
                "📋 List accounts",
                "🔄 Switch account",
                "🔗 Attach to current repo",
                "📊 Show status",
                "🩺 Doctor (diagnose setup)",
                "📄 View SSH config",
                "🗑️  Remove account",
                "⚠️  Reset application",
                "🚪 Exit",
            ];
            
            let selection = Select::new(
                "\n🔑 Git Account Manager (gam-cli) — what would you like to do?",
                options,
            )
                .prompt()
                .context("Failed to get menu selection")?;
            
            match selection {
                "📝 Add new account" => self.add_account()?,
                "📋 List accounts" => self.list_accounts()?,
                "🔄 Switch account" => self.switch_account()?,
                "🔗 Attach to current repo" => self.attach_repo()?,
                "📊 Show status" => self.show_status()?,
                "🩺 Doctor (diagnose setup)" => self.doctor()?,
                "📄 View SSH config" => self.view_ssh_config()?,
                "🗑️  Remove account" => self.remove_account()?,
                "⚠️  Reset application" => self.reset_application()?,
                "🚪 Exit" => {
                    println!("👋 Goodbye!");
                    break;
                }
                _ => unreachable!(),
            }
            
            // Pause before showing menu again
            println!("\nPress Enter to continue...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
        }
        
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut manager = SshManager::new(args.verbose)
        .context("Failed to initialize SSH manager")?;
    
    match args.command {
        Some(Commands::Add) => manager.add_account(),
        Some(Commands::List) => manager.list_accounts(),
        Some(Commands::Switch) => manager.switch_account(),
        Some(Commands::Remove) => manager.remove_account(),
        Some(Commands::Status) => manager.show_status(),
        Some(Commands::Reset) => manager.reset_application(),
        Some(Commands::Attach) => manager.attach_repo(),
        Some(Commands::Doctor) => manager.doctor(),
        None => manager.interactive_menu(),
    }
}

struct RepoContext {
    root: PathBuf,
    origin: Option<String>,
    user_name: Option<String>,
    user_email: Option<String>,
    ssh_command: Option<String>,
}

fn git_ok(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn repo_context() -> Result<Option<RepoContext>> {
    match Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
    {
        Err(_) => anyhow::bail!("Failed to run git. Is git installed?"),
        Ok(out) if !out.status.success() => Ok(None),
        Ok(_) => {
            let root = git_ok(&["rev-parse", "--show-toplevel"])
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            Ok(Some(RepoContext {
                root,
                origin: git_ok(&["config", "--get", "remote.origin.url"]),
                user_name: git_ok(&["config", "--local", "--get", "user.name"]),
                user_email: git_ok(&["config", "--local", "--get", "user.email"]),
                ssh_command: git_ok(&["config", "--local", "--get", "core.sshCommand"]),
            }))
        }
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
    use clap::CommandFactory;

    #[test]
    fn root_help_includes_examples_and_doctor() {
        let help = Args::command().render_long_help().to_string();
        assert!(help.contains("Examples"), "help should include examples:\n{help}");
        assert!(help.contains("doctor"), "help should mention doctor:\n{help}");
        assert!(help.contains("--verbose") || help.contains("-v"));
        assert!(help.contains("docs/"));
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
        assert_eq!(SshManager::alias_for(&account), "github-work");
    }
}
