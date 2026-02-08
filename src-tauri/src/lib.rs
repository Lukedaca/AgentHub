use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::io::{BufRead, BufReader};
use tauri::{AppHandle, Emitter, Manager, State};
use wait_timeout::ChildExt;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Status of an agent process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Running,
    Stopped,
    Error(String),
}

/// Serialisable snapshot returned to the frontend by `get_agents_status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
}

/// Payload emitted to the frontend whenever an agent writes to stdout/stderr.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutputEvent {
    pub id: String,
    pub data: String,
    pub stream: String, // "stdout" | "stderr"
}

/// Payload emitted when an agent process exits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExitEvent {
    pub id: String,
    pub code: Option<i32>,
}

/// Internal bookkeeping for a single managed agent.
pub struct AgentProcess {
    pub id: String,
    pub name: String,
    pub status: AgentStatus,
    pub child: Option<Child>,
}

/// Shared state across all Tauri commands.
pub struct AgentManager {
    pub agents: Mutex<HashMap<String, AgentProcess>>,
}

impl Default for AgentManager {
    fn default() -> Self {
        Self {
            agents: Mutex::new(HashMap::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Spawn a new CLI agent as a child process.
///
/// * `id`      - unique identifier chosen by the frontend
/// * `command` - program to execute (e.g. `"node"`, `"python"`, `"claude"`)
/// * `args`    - arguments passed to the program
#[tauri::command]
fn spawn_agent(
    id: String,
    command: String,
    args: Vec<String>,
    state: State<'_, AgentManager>,
    app: AppHandle,
) -> Result<AgentInfo, String> {
    let mut agents = state.agents.lock().map_err(|e| e.to_string())?;

    // Auto-remove dead agents with same ID (allows restart)
    if let Some(existing) = agents.get(&id) {
        if existing.status != AgentStatus::Running {
            agents.remove(&id);
        } else {
            return Err(format!("Agent '{}' už běží", id));
        }
    }

    // Spawn child with piped stdin / stdout / stderr
    // On Windows, use cmd /c to handle .cmd/.bat wrappers (npm installs)
    #[cfg(target_os = "windows")]
    let mut child = Command::new("cmd")
        .arg("/c")
        .arg(&command)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Nepodařilo se spustit '{}': {}", command, e))?;

    #[cfg(not(target_os = "windows"))]
    let mut child = Command::new(&command)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Nepodařilo se spustit '{}': {}", command, e))?;

    // --- stdout reader thread ---
    let stdout = child.stdout.take();
    if let Some(stdout) = stdout {
        let app_handle = app.clone();
        let agent_id = id.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(text) => {
                        let payload = AgentOutputEvent {
                            id: agent_id.clone(),
                            data: text,
                            stream: "stdout".to_string(),
                        };
                        let _ = app_handle.emit("agent-output", &payload);
                    }
                    Err(_) => break,
                }
            }
            // stdout closed -> process likely exited
            let _ = app_handle.emit(
                "agent-exit",
                &AgentExitEvent {
                    id: agent_id.clone(),
                    code: None,
                },
            );
            // Try to update status in state
            if let Some(manager) = app_handle.try_state::<AgentManager>() {
                if let Ok(mut agents) = manager.agents.lock() {
                    if let Some(agent) = agents.get_mut(&agent_id) {
                        // Try to collect exit code
                        if let Some(ref mut child) = agent.child {
                            if let Ok(status) = child.try_wait() {
                                agent.status = AgentStatus::Stopped;
                                if let Some(exit) = status {
                                    let _ = app_handle.emit(
                                        "agent-exit",
                                        &AgentExitEvent {
                                            id: agent_id.clone(),
                                            code: exit.code(),
                                        },
                                    );
                                }
                            }
                        } else {
                            agent.status = AgentStatus::Stopped;
                        }
                    }
                }
            }
        });
    }

    // --- stderr reader thread ---
    let stderr = child.stderr.take();
    if let Some(stderr) = stderr {
        let app_handle = app.clone();
        let agent_id = id.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(text) => {
                        let payload = AgentOutputEvent {
                            id: agent_id.clone(),
                            data: text,
                            stream: "stderr".to_string(),
                        };
                        let _ = app_handle.emit("agent-output", &payload);
                    }
                    Err(_) => break,
                }
            }
        });
    }

    // Build the friendly display name
    let name = if args.is_empty() {
        command.clone()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    let info = AgentInfo {
        id: id.clone(),
        name: name.clone(),
        status: AgentStatus::Running,
    };

    agents.insert(
        id.clone(),
        AgentProcess {
            id,
            name,
            status: AgentStatus::Running,
            child: Some(child),
        },
    );

    Ok(info)
}

/// Write a line of text to the agent's stdin pipe.
#[tauri::command]
fn send_to_agent(
    id: String,
    input: String,
    state: State<'_, AgentManager>,
) -> Result<(), String> {
    let mut agents = state.agents.lock().map_err(|e| e.to_string())?;
    let agent = agents
        .get_mut(&id)
        .ok_or_else(|| format!("Agent '{}' not found", id))?;

    if agent.status != AgentStatus::Running {
        return Err(format!("Agent '{}' is not running", id));
    }

    let child = agent
        .child
        .as_mut()
        .ok_or_else(|| format!("Agent '{}' has no child process", id))?;

    let stdin = child
        .stdin
        .as_mut()
        .ok_or_else(|| format!("Agent '{}' stdin not available", id))?;

    // Write the input followed by a newline so the agent receives a complete line
    stdin
        .write_all(input.as_bytes())
        .map_err(|e| format!("Failed to write to stdin: {}", e))?;
    stdin
        .write_all(b"\n")
        .map_err(|e| format!("Failed to write newline: {}", e))?;
    stdin
        .flush()
        .map_err(|e| format!("Failed to flush stdin: {}", e))?;

    Ok(())
}

/// Kill the agent process and mark it as Stopped.
#[tauri::command]
fn stop_agent(
    id: String,
    state: State<'_, AgentManager>,
) -> Result<AgentInfo, String> {
    let mut agents = state.agents.lock().map_err(|e| e.to_string())?;
    let agent = agents
        .get_mut(&id)
        .ok_or_else(|| format!("Agent '{}' not found", id))?;

    if let Some(ref mut child) = agent.child {
        child
            .kill()
            .map_err(|e| format!("Failed to kill agent '{}': {}", id, e))?;
        // Wait to clean up zombie / handle
        let _ = child.wait();
    }

    agent.status = AgentStatus::Stopped;
    agent.child = None;

    Ok(AgentInfo {
        id: agent.id.clone(),
        name: agent.name.clone(),
        status: agent.status.clone(),
    })
}

/// Return a snapshot of every managed agent and its status.
#[tauri::command]
fn get_agents_status(
    state: State<'_, AgentManager>,
) -> Result<Vec<AgentInfo>, String> {
    let mut agents = state.agents.lock().map_err(|e| e.to_string())?;

    let mut result: Vec<AgentInfo> = Vec::new();

    for agent in agents.values_mut() {
        // Refresh status: check if the child is still alive
        if agent.status == AgentStatus::Running {
            if let Some(ref mut child) = agent.child {
                match child.try_wait() {
                    Ok(Some(exit_status)) => {
                        if exit_status.success() {
                            agent.status = AgentStatus::Stopped;
                        } else {
                            agent.status = AgentStatus::Error(format!(
                                "Exited with code {}",
                                exit_status.code().unwrap_or(-1)
                            ));
                        }
                    }
                    Ok(None) => {
                        // Still running
                    }
                    Err(e) => {
                        agent.status =
                            AgentStatus::Error(format!("Failed to poll status: {}", e));
                    }
                }
            } else {
                agent.status = AgentStatus::Stopped;
            }
        }

        result.push(AgentInfo {
            id: agent.id.clone(),
            name: agent.name.clone(),
            status: agent.status.clone(),
        });
    }

    Ok(result)
}

/// Remove a stopped/errored agent from the manager entirely.
#[tauri::command]
fn remove_agent(
    id: String,
    state: State<'_, AgentManager>,
) -> Result<(), String> {
    let mut agents = state.agents.lock().map_err(|e| e.to_string())?;
    let agent = agents
        .get(&id)
        .ok_or_else(|| format!("Agent '{}' not found", id))?;

    if agent.status == AgentStatus::Running {
        return Err(format!(
            "Agent '{}' is still running. Stop it first.",
            id
        ));
    }

    agents.remove(&id);
    Ok(())
}

// ---------------------------------------------------------------------------
// Agent discovery - real system scan
// ---------------------------------------------------------------------------

/// Information about a discovered CLI agent on the system.
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

/// Agent signature database - maps command names to metadata.
/// Used to IDENTIFY agents found on the system, not to define what to show.
struct AgentSignature {
    command: &'static str,
    name: &'static str,
    short_name: &'static str,
    color: &'static str,
    npm_package: &'static str,
}

fn agent_signatures() -> Vec<AgentSignature> {
    vec![
        AgentSignature { command: "claude",   name: "Claude Code",    short_name: "CC", color: "#00FF64", npm_package: "@anthropic-ai/claude-code" },
        AgentSignature { command: "codex",    name: "Codex CLI",      short_name: "CX", color: "#3B82F6", npm_package: "@openai/codex" },
        AgentSignature { command: "gemini",   name: "Gemini CLI",     short_name: "GM", color: "#FFB800", npm_package: "@anthropic-ai/gemini-cli" },
        AgentSignature { command: "aider",    name: "Aider",          short_name: "AI", color: "#9333EA", npm_package: "" },
        AgentSignature { command: "cody",     name: "Cody CLI",       short_name: "CD", color: "#FF5733", npm_package: "" },
        AgentSignature { command: "continue", name: "Continue",       short_name: "CN", color: "#1389FD", npm_package: "" },
        AgentSignature { command: "cursor",   name: "Cursor Agent",   short_name: "CR", color: "#7C3AED", npm_package: "" },
        AgentSignature { command: "amp",      name: "Amp",            short_name: "AM", color: "#F59E0B", npm_package: "" },
    ]
}

/// Find command on PATH using `where` (Windows) / `which` (Unix).
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

/// Get version string from a CLI agent (with timeout).
fn get_version(cmd: &str) -> String {
    use std::time::Duration;

    let child = Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(mut child) => {
            // Wait max 3 seconds
            match child.wait_timeout(Duration::from_secs(3)) {
                Ok(Some(status)) if status.success() => {
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

/// Scan npm global packages for installed agents.
fn scan_npm_global() -> Vec<(String, String)> {
    // Returns vec of (package_name, command_name)
    let output = Command::new("npm")
        .args(["list", "-g", "--depth=0", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let mut found = Vec::new();

    if let Ok(o) = output {
        let text = String::from_utf8_lossy(&o.stdout);
        // Simple JSON parsing - look for known package names
        for sig in agent_signatures() {
            if !sig.npm_package.is_empty() && text.contains(sig.npm_package) {
                found.push((sig.npm_package.to_string(), sig.command.to_string()));
            }
        }
    }

    found
}

/// Discover CLI agents by scanning the system.
/// 1) Scan npm global packages
/// 2) Scan PATH for known agent executables
/// 3) Verify with --version
/// Only returns agents that are ACTUALLY installed.
#[tauri::command]
fn discover_agents() -> Vec<DiscoveredAgent> {
    let signatures = agent_signatures();
    let mut found: Vec<DiscoveredAgent> = Vec::new();
    let mut seen_commands: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Phase 1: Scan npm global
    let npm_agents = scan_npm_global();
    for (_, cmd) in &npm_agents {
        seen_commands.insert(cmd.clone());
    }

    // Phase 2: Scan PATH for all known agent commands
    for sig in &signatures {
        if let Some(path) = find_on_path(sig.command) {
            seen_commands.insert(sig.command.to_string());

            // Phase 3: Verify with --version
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
        .manage(AgentManager::default())
        .invoke_handler(tauri::generate_handler![
            spawn_agent,
            send_to_agent,
            stop_agent,
            get_agents_status,
            remove_agent,
            discover_agents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
