#[cfg(test)]
mod tmux_tests {
    use agent_bridge::adapters::tmux;
    use agent_bridge::db::BridgeDb;
    use agent_bridge::event_log::store::EventLog;
    use agent_bridge::event_log::model::EventType;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_tmux_shell_session_full_lifecycle() {
        let tmpdir = std::env::temp_dir().join("agent_cockpit_test");
        let _ = std::fs::create_dir_all(&tmpdir);

        let tmux_name = "ac_test_phase5";

        if tmux::session_exists(tmux_name) {
            tmux::kill_session(tmux_name).ok();
        }

        let result = tmux::create_tmux_session(tmux_name, tmpdir.to_str().unwrap(), "/bin/bash");
        assert!(result.is_ok(), "Session created: {:?}", result);
        assert!(tmux::session_exists(tmux_name));

        thread::sleep(Duration::from_millis(300));

        assert!(tmux::send_keys(tmux_name, "echo hello").is_ok());

        thread::sleep(Duration::from_millis(300));

        let output = tmux::capture_output(tmux_name, 10).unwrap();
        assert!(output.contains("hello"), "Output should contain 'hello', got: {}", output);

        let db_path = tmpdir.join("test_phase5.db");
        let db = BridgeDb::open(db_path.to_str().unwrap()).unwrap();
        let log = EventLog::new_db(db);

        let event = log.record(
            Some("sess_phase5"),
            EventType::UserInput,
            Some("echo hello"),
            None,
        ).unwrap();
        assert_eq!(event.seq, 1);

        log.record(
            Some("sess_phase5"),
            EventType::AgentOutput,
            Some(&output),
            None,
        ).unwrap();

        let events = log.fetch_after(Some("sess_phase5"), 0).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, EventType::UserInput);
        assert_eq!(events[1].event_type, EventType::AgentOutput);

        assert!(tmux::kill_session(tmux_name).is_ok());
        assert!(!tmux::session_exists(tmux_name));

        let _ = std::fs::remove_dir_all(&tmpdir);
    }
}
