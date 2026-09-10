use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use regex::Regex;

use crate::config::SshAccount;

pub fn alias_for(account: &SshAccount) -> String {
    let host_prefix = account
        .host
        .split('.')
        .next()
        .unwrap_or(&account.host)
        .to_string();
    let name_part = account.name.replace(' ', "-");
    format!("{host_prefix}-{name_part}")
}

pub fn update_ssh_config(ssh_dir: &Path, account: &SshAccount) -> Result<bool> {
    let ssh_config_path = ssh_dir.join("config");
    let key_path = ssh_dir.join(&account.key_file);
    let alias = alias_for(account);
    let host_config = format!(
        "\n# {} - {}\nHost {}\n    HostName {}\n    User git\n    IdentityFile {}\n    AddKeysToAgent yes\n    UseKeychain yes\n    IdentitiesOnly yes\n",
        account.name,
        account.description.as_deref().unwrap_or(&account.email),
        alias,
        account.host,
        key_path.display()
    );

    let current_config = if ssh_config_path.exists() {
        fs::read_to_string(&ssh_config_path).context("Failed to read SSH config")?
    } else {
        String::new()
    };

    let host_marker = format!("# {} - ", account.name);
    if current_config.contains(&host_marker) {
        return Ok(false);
    }

    fs::write(&ssh_config_path, current_config + &host_config)
        .context("Failed to write SSH config")?;
    Ok(true)
}

pub fn upsert_active_mapping(ssh_dir: &Path, host: &str, key_path: &Path) -> Result<()> {
    let ssh_config_path = ssh_dir.join("config");
    let mut current_config = if ssh_config_path.exists() {
        fs::read_to_string(&ssh_config_path).context("Failed to read SSH config")?
    } else {
        String::new()
    };

    let start_marker_new = format!("# gam ACTIVE START [{host}]\n");
    let end_marker_new = format!("# gam ACTIVE END [{host}]\n");
    let start_marker_old = format!("# ssh-manager ACTIVE START [{host}]\n");
    let end_marker_old = format!("# ssh-manager ACTIVE END [{host}]\n");

    let mut block = String::new();
    block.push_str(&start_marker_new);
    block.push_str(&format!(
        "Host {host}\n    HostName {host}\n    User git\n    IdentityFile {}\n    AddKeysToAgent yes\n    UseKeychain yes\n    IdentitiesOnly yes\n",
        key_path.display()
    ));
    block.push_str(&end_marker_new);

    let (start_marker, end_marker) = if current_config.contains(&start_marker_new) {
        (start_marker_new.clone(), end_marker_new.clone())
    } else {
        (start_marker_old.clone(), end_marker_old.clone())
    };
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
    Ok(())
}

pub fn clear_active_mapping_for_host(ssh_dir: &Path, host: &str) -> Result<()> {
    let ssh_config_path = ssh_dir.join("config");
    if !ssh_config_path.exists() {
        return Ok(());
    }

    let mut current_config =
        fs::read_to_string(&ssh_config_path).context("Failed to read SSH config")?;
    let markers = [
        (
            format!("# gam ACTIVE START [{host}]\n"),
            format!("# gam ACTIVE END [{host}]\n"),
        ),
        (
            format!("# ssh-manager ACTIVE START [{host}]\n"),
            format!("# ssh-manager ACTIVE END [{host}]\n"),
        ),
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
    Ok(())
}

pub fn remove_from_ssh_agent(key_path: &Path) {
    let _ = Command::new("ssh-add").arg("-d").arg(key_path).status();
}

pub fn remove_ssh_config_for_account(ssh_dir: &Path, account: &SshAccount) -> Result<bool> {
    let ssh_config_path = ssh_dir.join("config");
    if !ssh_config_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&ssh_config_path).context("Failed to read SSH config")?;
    let alias = alias_for(account);
    let name_escaped = regex::escape(&account.name);
    let alias_escaped = regex::escape(&alias);

    let header_pattern = format!(
        r"(?m)\n?# {}\s-\s[^\n]*\nHost {}\n(?:[ \t].*\n)*",
        name_escaped, alias_escaped
    );
    let re_header = Regex::new(&header_pattern).context("Failed to compile regex")?;
    let after_header = re_header.replace_all(&content, "");

    let host_pattern = format!(r"(?m)^\n?Host {}\n(?:[ \t].*\n)*", alias_escaped);
    let re_host = Regex::new(&host_pattern).context("Failed to compile regex")?;
    let new_content = re_host.replace_all(&after_header, "");

    if new_content != content {
        fs::write(&ssh_config_path, new_content.as_ref()).context("Failed to write SSH config")?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn read_ssh_config(ssh_dir: &Path) -> Result<Option<String>> {
    let ssh_config_path = ssh_dir.join("config");
    if !ssh_config_path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&ssh_config_path).context("Failed to read SSH config")?;
    Ok(Some(content))
}

pub fn key_path(ssh_dir: &Path, account: &SshAccount) -> PathBuf {
    ssh_dir.join(&account.key_file)
}
