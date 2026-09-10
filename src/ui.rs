use std::path::Path;

pub fn next_steps(steps: &[&str]) {
    if steps.is_empty() {
        return;
    }
    println!();
    println!("Next:");
    for (i, step) in steps.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }
}

pub fn hint(message: &str) {
    println!("  💡 {message}");
}

pub fn empty_state(title: &str, steps: &[&str]) {
    println!("📭 {title}");
    next_steps(steps);
}

pub fn kv(label: &str, value: impl AsRef<str>) {
    println!("      {:<12} {}", format!("{label}:"), value.as_ref());
}

pub fn kv_indent(indent: &str, label: &str, value: impl AsRef<str>) {
    println!("{indent}{:<12} {}", format!("{label}:"), value.as_ref());
}

pub fn path_home(path: &Path) -> String {
    let display = path.display().to_string();
    if let Some(home) = home::home_dir() {
        let home_s = home.display().to_string();
        if let Some(rest) = display.strip_prefix(&home_s) {
            return format!("~{rest}");
        }
    }
    display
}

pub fn clap_styles() -> clap::builder::Styles {
    use clap::builder::styling::{AnsiColor, Effects, Styles};
    Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::BrightBlack.on_default())
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
        .valid(AnsiColor::Green.on_default() | Effects::BOLD)
        .invalid(AnsiColor::Yellow.on_default() | Effects::BOLD)
}

pub const ROOT_AFTER_HELP: &str = "\
Examples:
  gam                 Open the interactive menu
  gam add             Create a new SSH identity
  gam list -v         Accounts with aliases, keys and fingerprints
  gam status -v       Active account plus this repo's git identity
  gam add --name work --email you@org.com --json
  gam attach          Bind the current git repo to an account
  gam ensure --json   Apply mapped identity (agents)
  gam doctor          Diagnose SSH keys, config and git identity
  gam mcp             MCP stdio server for AI agents
  gam help list       Detailed help for one command

Docs: https://lucasgio.github.io/gam-cli/
";

pub const ROOT_LONG_ABOUT: &str = "\
Git Account Manager (gam) keeps several Git SSH identities on one machine.

Each account gets its own key and a Host alias in ~/.ssh/config
(e.g. git@github-work:org/repo.git). Use attach to set user.name, user.email
and core.sshCommand for a single repository without changing other repos.

Run with no arguments for an interactive menu. Pass -v / --verbose on list,
status and doctor for extra diagnostics. Pass --json for agents.
See https://lucasgio.github.io/gam-cli/ for the full guide.
";
