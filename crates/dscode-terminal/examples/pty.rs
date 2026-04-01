//! PTY terminal example.
//!
//! Demonstrates creating a terminal manager, spawning a PTY, and handling output.

use dscode_terminal::{TerminalEventSender, TerminalManager};

struct PrintSender;

impl TerminalEventSender for PrintSender {
    fn send_output(&self, terminal_id: &str, data: &str) {
        print!("[{}] {}", terminal_id, data);
    }

    fn send_close(&self, terminal_id: &str) {
        println!("[{}] closed", terminal_id);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TerminalManager::new(Box::new(PrintSender));

    // Create a terminal with the default shell
    let id = manager.create_terminal(None, None, None)?;
    println!("Created terminal: {}", id);

    // Start reading PTY output
    manager.start_reading(&id)?;

    // Write a command
    manager.write_to_terminal(&id, "echo 'Hello from PTY'\n")?;

    // Resize
    manager.resize_terminal(&id, 120, 40)?;

    // Clean up
    manager.close_terminal(&id)?;

    Ok(())
}
