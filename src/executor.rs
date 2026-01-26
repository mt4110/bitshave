use anyhow::{Result, Context};
use std::process::Command;

/// Runs an external command safely.
/// Returns Ok(true) if the command succeeded.
pub fn run_external(program: &str, args: &[&str]) -> Result<bool> {
    // Basic check if program exists (optional, but good for "PATH check")
    
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute tool: {}", program))?;

    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_echo() {
        let res = run_external("echo", &["hello"]).unwrap();
        assert!(res);
    }
}
