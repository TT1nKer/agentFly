use agent_bridge::db;
use std::io::{self, Write};
use chrono::Utc;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "pair" => cmd_pair(),
        "devices" => {
            if args.len() < 3 {
                println!("Usage: agent-bridge devices <list|revoke> [device_id]");
                return;
            }
            match args[2].as_str() {
                "list" => cmd_devices_list(),
                "revoke" => {
                    if args.len() < 4 {
                        println!("Usage: agent-bridge devices revoke <device_id>");
                        return;
                    }
                    cmd_devices_revoke(&args[3]);
                }
                _ => println!("Unknown devices command: {}", args[2]),
            }
        }
        "run" => cmd_run(),
        _ => {
            println!("Unknown command: {}", args[1]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("agent-bridge - Agent Cockpit Bridge");
    println!();
    println!("Commands:");
    println!("  pair                  Start device pairing mode");
    println!("  devices list          List paired devices");
    println!("  devices revoke <id>   Revoke a paired device");
    println!("  run                   Run the bridge (connect to relay)");
}

fn cmd_pair() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let pairing_code: u32 = rng.gen_range(100000..999999);

    let db = db::BridgeDb::open("bridge.db").expect("Failed to open database");
    let expires_at = Utc::now().timestamp() + 600;
    db.set_pairing_code(pairing_code.to_string(), expires_at)
        .expect("Failed to save pairing code");

    println!("Pairing code: {}", pairing_code);
    println!("Expires in 10 minutes.");
    println!("Waiting for phone...");
    io::stdout().flush().ok();
}

fn cmd_devices_list() {
    let db = db::BridgeDb::open("bridge.db").expect("Failed to open database");
    match db.list_trusted_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("No paired devices.");
            } else {
                println!("Paired devices:");
                for d in devices {
                    println!("- {}  {}  {}  last_seen={}",
                        d.name, d.device_id, d.status, d.last_seen.unwrap_or_default());
                }
            }
        }
        Err(e) => println!("Error listing devices: {}", e),
    }
}

fn cmd_devices_revoke(device_id: &str) {
    let db = db::BridgeDb::open("bridge.db").expect("Failed to open database");
    match db.revoke_device(device_id) {
        Ok(_) => println!("Device {} revoked.", device_id),
        Err(e) => println!("Error revoking device: {}", e),
    }
}

fn cmd_run() {
    println!("Bridge running... (WebSocket client to relay, not yet implemented)");
}
