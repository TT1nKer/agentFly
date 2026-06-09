use crate::adapters::tmux;

pub fn create_shell_session(session_id: &str, workspace: &str) -> Result<(), String> {
    let tmux_name = format!("ac_{}", &session_id[..8.min(session_id.len())]);
    tmux::create_tmux_session(&tmux_name, workspace, "/bin/bash")?;
    Ok(())
}

pub fn send_shell_input(session_id: &str, input: &str) -> Result<(), String> {
    let tmux_name = format!("ac_{}", &session_id[..8.min(session_id.len())]);
    if !tmux::session_exists(&tmux_name) {
        return Err("Session not found".to_string());
    }
    tmux::send_keys(&tmux_name, input)?;
    Ok(())
}

pub fn capture_shell_output(session_id: &str, lines: i32) -> Result<String, String> {
    let tmux_name = format!("ac_{}", &session_id[..8.min(session_id.len())]);
    if !tmux::session_exists(&tmux_name) {
        return Err("Session not found".to_string());
    }
    tmux::capture_output(&tmux_name, lines)
}

pub fn stop_shell_session(session_id: &str) -> Result<(), String> {
    let tmux_name = format!("ac_{}", &session_id[..8.min(session_id.len())]);
    if !tmux::session_exists(&tmux_name) {
        return Err("Session not found".to_string());
    }
    tmux::kill_session(&tmux_name)?;
    Ok(())
}
