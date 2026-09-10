use std::process::Command;

/// Ask the human for an SSH key passphrase via a native OS dialog.
/// Never prints the secret. Cancel and "no GUI helper" are structured errors.
#[derive(Debug)]
pub enum PassphrasePromptError {
    Cancelled,
    Empty,
    Mismatch,
    Unavailable(String),
    Failed(String),
}

impl std::fmt::Display for PassphrasePromptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "Passphrase prompt was cancelled."),
            Self::Empty => write!(f, "Passphrase was empty."),
            Self::Mismatch => write!(f, "Passphrases did not match."),
            Self::Unavailable(msg) => write!(f, "{msg}"),
            Self::Failed(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for PassphrasePromptError {}

pub fn prompt_confirmed(account: &str) -> Result<String, PassphrasePromptError> {
    let first = prompt_once(&format!(
        "Enter passphrase for the new GAM SSH key ({account})"
    ))?;
    if first.is_empty() {
        return Err(PassphrasePromptError::Empty);
    }
    let second = prompt_once(&format!(
        "Confirm passphrase for the new GAM SSH key ({account})"
    ))?;
    if first != second {
        return Err(PassphrasePromptError::Mismatch);
    }
    Ok(first)
}

fn prompt_once(message: &str) -> Result<String, PassphrasePromptError> {
    #[cfg(target_os = "macos")]
    {
        return macos_prompt(message);
    }
    #[cfg(target_os = "windows")]
    {
        return windows_prompt(message);
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return linux_prompt(message);
    }
    #[allow(unreachable_code)]
    Err(PassphrasePromptError::Unavailable(
        "No native passphrase dialog on this OS.".into(),
    ))
}

#[cfg(target_os = "macos")]
fn macos_prompt(message: &str) -> Result<String, PassphrasePromptError> {
    let script = r#"
on run argv
  set prompt to item 1 of argv
  try
    return text returned of (display dialog prompt default answer "" with hidden answer with title "GAM" buttons {"Cancel", "OK"} default button "OK")
  on error
    error "cancelled" number 1
  end try
end run
"#;
    let mut child = Command::new("osascript")
        .arg("-")
        .arg(message)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| PassphrasePromptError::Failed(format!("osascript: {e}")))?;
    {
        use std::io::Write;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| PassphrasePromptError::Failed("osascript stdin".into()))?;
        stdin
            .write_all(script.as_bytes())
            .map_err(|e| PassphrasePromptError::Failed(format!("osascript: {e}")))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| PassphrasePromptError::Failed(format!("osascript: {e}")))?;

    if !output.status.success() {
        return Err(PassphrasePromptError::Cancelled);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim_end_matches(['\n', '\r']).to_string())
}

#[cfg(target_os = "windows")]
fn windows_prompt(message: &str) -> Result<String, PassphrasePromptError> {
    let script = r#"
$msg = $env:GAM_PROMPT_MSG
try {
  $cred = Get-Credential -Message $msg -UserName 'GAM SSH key'
} catch {
  exit 2
}
if ($null -eq $cred) { exit 1 }
$bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($cred.Password)
try {
  $plain = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr)
  [Console]::Out.Write($plain)
} finally {
  [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)
}
"#;
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-STA", "-Command", script])
        .env("GAM_PROMPT_MSG", message)
        .output()
        .map_err(|e| PassphrasePromptError::Failed(format!("powershell: {e}")))?;

    if !output.status.success() {
        return Err(PassphrasePromptError::Cancelled);
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn linux_prompt(message: &str) -> Result<String, PassphrasePromptError> {
    let has_gui = std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();
    if !has_gui {
        return Err(PassphrasePromptError::Unavailable(
            "No graphical session (DISPLAY/WAYLAND_DISPLAY unset). Use a desktop session with zenity/kdialog, or run `gam add` interactively in a TTY.".into(),
        ));
    }
    if command_exists("zenity") {
        let mut cmd = Command::new("zenity");
        cmd.args(["--password", "--title=GAM", "--text", message]);
        return run_capture(&mut cmd);
    }
    if command_exists("kdialog") {
        let mut cmd = Command::new("kdialog");
        cmd.args(["--title", "GAM", "--password", message]);
        return run_capture(&mut cmd);
    }
    if command_exists("yad") {
        let mut cmd = Command::new("yad");
        cmd.args(["--entry", "--hide-text", "--title=GAM", "--text", message]);
        return run_capture(&mut cmd);
    }
    let askpass = std::env::var("SSH_ASKPASS").ok().filter(|s| !s.is_empty());
    if let Some(bin) = askpass {
        let mut cmd = Command::new(bin);
        cmd.arg(message);
        return run_capture(&mut cmd);
    }
    for bin in ["ssh-askpass", "lxqt-openssh-askpass", "ksshaskpass"] {
        if command_exists(bin) {
            let mut cmd = Command::new(bin);
            cmd.arg(message);
            return run_capture(&mut cmd);
        }
    }
    Err(PassphrasePromptError::Unavailable(
        "No GUI password helper found. Install zenity (GNOME), kdialog (KDE), or yad, then retry."
            .into(),
    ))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .args(["-c", "command -v \"$1\" >/dev/null"])
        .arg("sh")
        .arg(name)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn run_capture(cmd: &mut Command) -> Result<String, PassphrasePromptError> {
    let output = cmd
        .output()
        .map_err(|e| PassphrasePromptError::Failed(e.to_string()))?;
    if !output.status.success() {
        return Err(PassphrasePromptError::Cancelled);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim_end_matches(['\n', '\r']).to_string())
}
