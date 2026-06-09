use std::process::Command;

pub fn create_tmux_session(name: &str, workspace: &str, command: &str) -> Result<(), String> {
    let output = Command::new("tmux")
        .args(["new-session", "-d", "-s", name, "-c", workspace, command])
        .output()
        .map_err(|e| format!("tmux new-session: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(())
}

pub fn send_keys(name: &str, input: &str) -> Result<(), String> {
    let output = Command::new("tmux")
        .args(["send-keys", "-t", name, input, "Enter"])
        .output()
        .map_err(|e| format!("tmux send-keys: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(())
}

pub fn capture_output(name: &str, lines: i32) -> Result<String, String> {
    let output = Command::new("tmux")
        .args(["capture-pane", "-pt", name, "-S", &format!("-{}", lines)])
        .output()
        .map_err(|e| format!("tmux capture-pane: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn kill_session(name: &str) -> Result<(), String> {
    let output = Command::new("tmux")
        .args(["kill-session", "-t", name])
        .output()
        .map_err(|e| format!("tmux kill-session: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(())
}

pub fn session_exists(name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
