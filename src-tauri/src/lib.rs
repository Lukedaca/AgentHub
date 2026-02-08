use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::io::{BufRead, BufReader, Read as IoRead};
use tauri::{AppHandle, Emitter, Manager, State};
use wait_timeout::ChildExt;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutputEvent {
    pub id: String,
    pub data: String,
    pub stream: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDoneEvent {
    pub id: String,
    pub code: Option<i32>,
}

/// Tracks which agents are currently processing a message.
pub struct AgentState {
    pub busy: Mutex<std::collections::HashSet<String>>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            busy: Mutex::new(std::collections::HashSet::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// One-shot agent execution
// ---------------------------------------------------------------------------

/// Run a one-shot message through a CLI agent.
/// Spawns `<command> -p "<message>"`, streams output back via events.
#[tauri::command]
fn run_agent(
    id: String,
    command: String,
    message: String,
    state: State<'_, AgentState>,
    app: AppHandle,
) -> Result<(), String> {
    // Check if already processing
    {
        let mut busy = state.busy.lock().map_err(|e| e.to_string())?;
        if busy.contains(&id) {
            return Err(format!("Agent '{}' právě zpracovává zprávu", id));
        }
        busy.insert(id.clone());
    }

    let app_handle = app.clone();
    let agent_id = id.clone();

    std::thread::spawn(move || {
        // Build command: cmd /c <command> -p "<message>"
        #[cfg(target_os = "windows")]
        let child = Command::new("cmd")
            .args(["/c", &command, "-p", &message])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        #[cfg(not(target_os = "windows"))]
        let child = Command::new(&command)
            .args(["-p", &message])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match child {
            Ok(mut child) => {
                // Read stdout line by line and stream to frontend
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(text) = line {
                            let _ = app_handle.emit("agent-output", &AgentOutputEvent {
                                id: agent_id.clone(),
                                data: text,
                                stream: "stdout".to_string(),
                            });
                        }
                    }
                }

                // Collect stderr
                if let Some(mut stderr) = child.stderr.take() {
                    let mut err_text = String::new();
                    let _ = stderr.read_to_string(&mut err_text);
                    if !err_text.trim().is_empty() {
                        let _ = app_handle.emit("agent-output", &AgentOutputEvent {
                            id: agent_id.clone(),
                            data: err_text.trim().to_string(),
                            stream: "stderr".to_string(),
                        });
                    }
                }

                // Wait for exit
                let code = child.wait().ok().and_then(|s| s.code());
                let _ = app_handle.emit("agent-done", &AgentDoneEvent {
                    id: agent_id.clone(),
                    code,
                });
            }
            Err(e) => {
                let _ = app_handle.emit("agent-output", &AgentOutputEvent {
                    id: agent_id.clone(),
                    data: format!("Chyba při spouštění: {}", e),
                    stream: "stderr".to_string(),
                });
                let _ = app_handle.emit("agent-done", &AgentDoneEvent {
                    id: agent_id.clone(),
                    code: Some(-1),
                });
            }
        }

        // Mark as no longer busy
        {
            let state_ref = app_handle.state::<AgentState>();
            let mut busy = state_ref.busy.lock().unwrap();
            busy.remove(&agent_id);
        }
    });

    Ok(())
}

/// Check if an agent is currently busy processing.
#[tauri::command]
fn is_agent_busy(
    id: String,
    state: State<'_, AgentState>,
) -> Result<bool, String> {
    let busy = state.busy.lock().map_err(|e| e.to_string())?;
    Ok(busy.contains(&id))
}

// ---------------------------------------------------------------------------
// Agent discovery - real system scan
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredAgent {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub command: String,
    pub path: String,
    pub color: String,
    pub version: String,
    pub available: bool,
}

struct AgentSignature {
    command: &'static str,
    name: &'static str,
    short_name: &'static str,
    color: &'static str,
    npm_package: &'static str,
}

fn agent_signatures() -> Vec<AgentSignature> {
    vec![
        AgentSignature { command: "claude",   name: "Claude Code",  short_name: "CC", color: "#00FF64", npm_package: "@anthropic-ai/claude-code" },
        AgentSignature { command: "codex",    name: "Codex CLI",    short_name: "CX", color: "#3B82F6", npm_package: "@openai/codex" },
        AgentSignature { command: "gemini",   name: "Gemini CLI",   short_name: "GM", color: "#FFB800", npm_package: "" },
        AgentSignature { command: "aider",    name: "Aider",        short_name: "AI", color: "#9333EA", npm_package: "" },
        AgentSignature { command: "cody",     name: "Cody CLI",     short_name: "CD", color: "#FF5733", npm_package: "" },
        AgentSignature { command: "cursor",   name: "Cursor Agent", short_name: "CR", color: "#7C3AED", npm_package: "" },
        AgentSignature { command: "amp",      name: "Amp",          short_name: "AM", color: "#F59E0B", npm_package: "" },
    ]
}

fn find_on_path(cmd: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    let output = Command::new("where")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("which")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let path = String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if path.is_empty() { None } else { Some(path) }
        }
        _ => None,
    }
}

fn get_version(cmd: &str) -> String {
    use std::time::Duration;

    #[cfg(target_os = "windows")]
    let child = Command::new("cmd")
        .args(["/c", cmd, "--version"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    #[cfg(not(target_os = "windows"))]
    let child = Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(mut child) => {
            match child.wait_timeout(Duration::from_secs(5)) {
                Ok(Some(_)) => {
                    let stdout = child.stdout.take()
                        .map(|s| {
                            let mut buf = String::new();
                            BufReader::new(s).read_line(&mut buf).ok();
                            buf.trim().to_string()
                        })
                        .unwrap_or_default();
                    if stdout.is_empty() {
                        child.stderr.take()
                            .map(|s| {
                                let mut buf = String::new();
                                BufReader::new(s).read_line(&mut buf).ok();
                                buf.trim().to_string()
                            })
                            .unwrap_or_default()
                    } else {
                        stdout
                    }
                }
                _ => {
                    let _ = child.kill();
                    String::new()
                }
            }
        }
        Err(_) => String::new(),
    }
}

fn scan_npm_global() -> Vec<(String, String)> {
    let output = Command::new("npm")
        .args(["list", "-g", "--depth=0", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let mut found = Vec::new();
    if let Ok(o) = output {
        let text = String::from_utf8_lossy(&o.stdout);
        for sig in agent_signatures() {
            if !sig.npm_package.is_empty() && text.contains(sig.npm_package) {
                found.push((sig.npm_package.to_string(), sig.command.to_string()));
            }
        }
    }
    found
}

#[tauri::command]
fn discover_agents() -> Vec<DiscoveredAgent> {
    let signatures = agent_signatures();
    let mut found: Vec<DiscoveredAgent> = Vec::new();

    // Scan npm global
    let _npm_agents = scan_npm_global();

    // Scan PATH + verify version
    for sig in &signatures {
        if let Some(path) = find_on_path(sig.command) {
            let version = get_version(sig.command);

            found.push(DiscoveredAgent {
                id: sig.command.to_string(),
                name: sig.name.to_string(),
                short_name: sig.short_name.to_string(),
                command: sig.command.to_string(),
                path,
                color: sig.color.to_string(),
                version,
                available: true,
            });
        }
    }

    found
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AgentState::default())
        .invoke_handler(tauri::generate_handler![
            run_agent,
            is_agent_busy,
            discover_agents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
