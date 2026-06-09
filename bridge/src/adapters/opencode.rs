use crate::adapters::tmux;

pub fn create_opencode_session(session_id: &str, workspace: &str) -> Result<(), String> {
    let tmux_name = tmux_session_name(session_id);
    if tmux::session_exists(&tmux_name) {
        tmux::kill_session(&tmux_name)?;
    }
    tmux::create_tmux_session(&tmux_name, workspace, "opencode")?;
    Ok(())
}

pub fn send_input(session_id: &str, input: &str) -> Result<(), String> {
    let tmux_name = tmux_session_name(session_id);
    tmux::send_keys(&tmux_name, input)?;
    Ok(())
}

pub fn capture_output(session_id: &str, lines: i32) -> Result<String, String> {
    let tmux_name = tmux_session_name(session_id);
    tmux::capture_output(&tmux_name, lines)
}

pub fn stop_session(session_id: &str) -> Result<(), String> {
    let tmux_name = tmux_session_name(session_id);
    if tmux::session_exists(&tmux_name) {
        tmux::kill_session(&tmux_name)?;
    }
    Ok(())
}

pub fn session_exists(session_id: &str) -> bool {
    tmux::session_exists(&tmux_session_name(session_id))
}

fn tmux_session_name(session_id: &str) -> String {
    format!("ac_op_{}", &session_id[..8.min(session_id.len())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_opencode_session_lifecycle() {
        let tmpdir = std::env::temp_dir().join("ac_test_opencode");
        let _ = std::fs::create_dir_all(&tmpdir);
        let sid = "sess_open_001";

        if session_exists(sid) {
            stop_session(sid).ok();
        }

        let result = create_opencode_session(sid, tmpdir.to_str().unwrap());
        assert!(result.is_ok(), "opencode session created: {:?}", result);
        assert!(session_exists(sid));

        thread::sleep(Duration::from_millis(500));

        assert!(stop_session(sid).is_ok());
        assert!(!session_exists(sid));

        let _ = std::fs::remove_dir_all(&tmpdir);
    }
}
