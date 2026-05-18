use std::path::PathBuf;
use std::process::ExitCode;

use std::sync::Arc;

use clap::{Parser, Subcommand};
use stubhouse_core::{
    from_postman_v21, interpolate_compose, list_environments, load_environment,
    mock::{
        activate_scenario, list_scenarios, load_rules,
        server::{run_with_hot_reload, MockReload},
    },
    to_curl, Workspace,
};

#[derive(Parser)]
#[command(
    name = "stubhouse",
    version,
    about = "StubHouse — stub it, ship it. Local-first API client + mock server.",
    propagate_version = true
)]
struct Cli {
    /// Path to the workspace root (defaults to CWD)
    #[arg(short, long, global = true)]
    workspace: Option<PathBuf>,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Initialise a new workspace
    Init {
        /// Workspace name
        name: String,
    },
    /// Validate a workspace's YAML files
    Validate,
    /// List all requests in the workspace
    List,
    /// Show a single request as canonical YAML
    Show {
        /// Request id, e.g. collections/users/get-user.yaml
        id: String,
    },
    /// List environments in the workspace
    Envs,
    /// Import a foreign collection
    Import {
        #[command(subcommand)]
        format: ImportFmt,
    },
    /// Export a request as a code snippet
    Export {
        #[command(subcommand)]
        format: ExportFmt,
    },
    /// Run the embedded mock server (Phase 2 slice 0)
    Serve {
        /// Bind port (default 4000)
        #[arg(short, long, default_value_t = 4000)]
        port: u16,
        /// Bind address (default 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
    },
    /// Inspect or switch mock scenarios
    Scenario {
        #[command(subcommand)]
        action: ScenarioCmd,
    },
}

#[derive(Subcommand)]
enum ImportFmt {
    /// Import a Postman v2.1 collection JSON file
    Postman {
        /// Path to the Postman collection JSON
        file: PathBuf,
    },
}

#[derive(Subcommand)]
enum ExportFmt {
    /// Emit a cURL snippet for a request
    Curl {
        /// Request id, e.g. collections/users/get-user.yaml
        id: String,
        /// Apply this environment's variables before emitting
        #[arg(long)]
        env: Option<String>,
    },
}

#[derive(Subcommand)]
enum ScenarioCmd {
    /// List mock scenarios found under collections/*/mocks/*.yaml
    List,
    /// Activate a scenario name across every rule that defines it
    Activate {
        /// Scenario name to activate
        name: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let root = cli
        .workspace
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    match cli.command {
        Cmd::Init { name } => init(&root, &name),
        Cmd::Validate => validate(&root),
        Cmd::List => list(&root),
        Cmd::Show { id } => show(&root, &id),
        Cmd::Envs => envs(&root),
        Cmd::Import {
            format: ImportFmt::Postman { file },
        } => import_postman(&root, &file),
        Cmd::Export {
            format: ExportFmt::Curl { id, env },
        } => export_curl(&root, &id, env.as_deref()),
        Cmd::Serve { port, bind } => serve(&root, &bind, port),
        Cmd::Scenario { action } => scenario(&root, action),
    }
}

fn scenario(root: &std::path::Path, action: ScenarioCmd) -> Result<(), String> {
    let _ws = Workspace::open(root).map_err(|e| e.to_string())?;
    match action {
        ScenarioCmd::List => {
            let scenarios = list_scenarios(root).map_err(|e| e.to_string())?;
            if scenarios.is_empty() {
                println!("(no mock scenarios)");
                return Ok(());
            }
            for scenario in scenarios {
                let active = if scenario.active_rules > 0 { "*" } else { " " };
                println!(
                    "{active} {}  ({}/{} active)",
                    scenario.name, scenario.active_rules, scenario.rules
                );
            }
            Ok(())
        }
        ScenarioCmd::Activate { name } => {
            let activation = activate_scenario(root, &name).map_err(|e| e.to_string())?;
            println!(
                "activated scenario '{}' in {} rule file(s)",
                activation.scenario, activation.files_changed
            );
            Ok(())
        }
    }
}

fn serve(root: &std::path::Path, bind: &str, port: u16) -> Result<(), String> {
    let _ws = Workspace::open(root).map_err(|e| e.to_string())?;
    let rules = load_rules(root).map_err(|e| e.to_string())?;
    let addr: std::net::SocketAddr = format!("{bind}:{port}")
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;

    eprintln!(
        "stubhouse mock server: loaded {} rule(s) — listening on http://{addr}",
        rules.len()
    );
    for r in &rules {
        eprintln!("  {:?} {} → {}", r.method, r.path, r.response.status);
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async move {
        let log_fn: Arc<dyn Fn(stubhouse_core::mock::server::MockLog) + Send + Sync> =
            Arc::new(|log| {
                eprintln!(
                    "{} {} → {} {}",
                    log.method,
                    log.path,
                    log.status,
                    log.matched_rule.as_deref().unwrap_or("(no match)"),
                );
            });
        let reload_fn: Arc<dyn Fn(MockReload) + Send + Sync> = Arc::new(|reload| {
            if let Some(error) = reload.error {
                eprintln!(
                    "stubhouse mock reload failed; keeping {} rule(s): {error}",
                    reload.rules
                );
            } else {
                eprintln!("stubhouse mock reloaded: {} rule(s)", reload.rules);
            }
        });

        let (tx, rx) = tokio::sync::oneshot::channel();
        let root = root.to_path_buf();
        let server = tokio::spawn(async move {
            run_with_hot_reload(root, addr, Some(rx), Some(log_fn), Some(reload_fn)).await
        });

        // Graceful shutdown on Ctrl-C.
        if tokio::signal::ctrl_c().await.is_ok() {
            eprintln!("\nshutting down…");
        }
        let _ = tx.send(());
        let _ = server.await;
        Ok::<(), String>(())
    })
}

fn init(root: &std::path::Path, name: &str) -> Result<(), String> {
    Workspace::init(root, name).map_err(|e| e.to_string())?;
    println!("initialised workspace '{name}' at {}", root.display());
    Ok(())
}

fn validate(root: &std::path::Path) -> Result<(), String> {
    let ws = Workspace::open(root).map_err(|e| e.to_string())?;
    let entries = ws.list_requests().map_err(|e| e.to_string())?;
    let mut errors = 0usize;
    for entry in &entries {
        if let Err(e) = ws.load_request(&entry.id) {
            errors += 1;
            eprintln!("✗ {}: {e}", entry.id);
        }
    }
    let envs = list_environments(root).map_err(|e| e.to_string())?;
    for e in &envs {
        if let Err(err) = load_environment(root, &e.name) {
            errors += 1;
            eprintln!("✗ environments/{}: {err}", e.file);
        }
    }
    if errors > 0 {
        return Err(format!("{errors} file(s) failed validation"));
    }
    println!(
        "ok — {} request(s), {} environment(s) parse cleanly",
        entries.len(),
        envs.len()
    );
    Ok(())
}

fn list(root: &std::path::Path) -> Result<(), String> {
    let ws = Workspace::open(root).map_err(|e| e.to_string())?;
    let entries = ws.list_requests().map_err(|e| e.to_string())?;
    if entries.is_empty() {
        println!("(no requests)");
        return Ok(());
    }
    let mut current = "";
    for entry in &entries {
        if entry.collection != current {
            println!("{}/", entry.collection);
            current = &entry.collection;
        }
        println!("  {}  ·  {}", entry.id, entry.name);
    }
    Ok(())
}

fn show(root: &std::path::Path, id: &str) -> Result<(), String> {
    let ws = Workspace::open(root).map_err(|e| e.to_string())?;
    let def = ws.load_request(id).map_err(|e| e.to_string())?;
    let yaml = serde_yaml::to_string(&def).map_err(|e| e.to_string())?;
    print!("{yaml}");
    Ok(())
}

fn envs(root: &std::path::Path) -> Result<(), String> {
    let envs = list_environments(root).map_err(|e| e.to_string())?;
    if envs.is_empty() {
        println!("(no environments — add files under .stubhouse/environments/)");
        return Ok(());
    }
    for e in &envs {
        println!("{}  ({})", e.name, e.file);
    }
    Ok(())
}

fn import_postman(root: &std::path::Path, file: &std::path::Path) -> Result<(), String> {
    let ws = match Workspace::open(root) {
        Ok(ws) => ws,
        Err(_) => {
            let name = root
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("workspace")
                .to_string();
            Workspace::init(root, &name).map_err(|e| e.to_string())?
        }
    };
    let json =
        std::fs::read_to_string(file).map_err(|e| format!("read {}: {e}", file.display()))?;
    let items = from_postman_v21(&json).map_err(|e| e.to_string())?;
    for item in &items {
        ws.save_request(&item.collection, &item.slug, &item.def)
            .map_err(|e| e.to_string())?;
    }
    println!(
        "imported {} request(s) from {}",
        items.len(),
        file.display()
    );
    Ok(())
}

fn export_curl(root: &std::path::Path, id: &str, env_name: Option<&str>) -> Result<(), String> {
    let ws = Workspace::open(root).map_err(|e| e.to_string())?;
    let def = ws.load_request(id).map_err(|e| e.to_string())?;
    let compose = match env_name {
        Some(name) => {
            let env = load_environment(root, name).map_err(|e| e.to_string())?;
            interpolate_compose(&def.compose, &env.variables)
        }
        None => def.compose,
    };
    let snippet = to_curl(&compose).map_err(|e| e.to_string())?;
    println!("{snippet}");
    Ok(())
}
