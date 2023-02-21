use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
};

use anyhow::{anyhow, Result};

pub fn format_code(code: &str) -> Result<String> {
    let mut clang_format = Command::new("clang-format")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = clang_format
        .stdin
        .take()
        .ok_or(anyhow!("Couldn't take clang-format stdin"))?;

    let code = code.to_owned();
    thread::spawn(move || stdin.write_all(code.as_bytes()));

    let output = clang_format.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}