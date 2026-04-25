mod formatter;
mod scanner;

use anyhow::Result;
use clap::{Parser, Subcommand};

// ─── Global Options ──────────────────────────────────────────────────────────
// These apply to every subcommand via the top-level Cli struct.

#[derive(Parser)]
#[command(name = "limma")]
#[command(version)]
#[command(about = "Limma — Security Reconnaissance CLI")]
#[command(
    long_about = "Limma — Security Reconnaissance CLI\n\n\
    Run security scans, compare results, and manage findings from the terminal.\n\n\
    Examples:\n  \
      limma scan https://example.com\n  \
      limma scan https://example.com --json\n  \
      limma scan https://example.com --md --out report.md\n  \
      limma result abc123-def456\n  \
      limma history https://example.com\n  \
      limma compare CURRENT_ID PREVIOUS_ID\n  \
      limma audit https://example.com\n  \
      limma rules"
)]
struct Cli {
    /// Backend API base URL
    #[arg(long, global = true, default_value = "http://localhost:8900", env = "LIMMA_API_URL")]
    api_url: String,

    /// API key for authenticated access
    #[arg(long, global = true, env = "LIMMA_API_KEY")]
    api_key: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

// ─── Format Flags ────────────────────────────────────────────────────────────
// Instead of --format json, use --json / --md / --sarif as boolean flags.
// Default is JSON when none specified.

#[derive(Parser, Clone)]
struct FormatFlags {
    /// Output as JSON (default)
    #[arg(long, group = "fmt")]
    json: bool,

    /// Output as Markdown
    #[arg(long, group = "fmt")]
    md: bool,

    /// Output as SARIF 2.1.0
    #[arg(long, group = "fmt")]
    sarif: bool,

    /// Write output to file instead of stdout
    #[arg(long, short)]
    out: Option<String>,
}

impl FormatFlags {
    fn resolve(&self) -> formatter::OutputFormat {
        if self.md {
            formatter::OutputFormat::Markdown
        } else if self.sarif {
            formatter::OutputFormat::Sarif
        } else {
            formatter::OutputFormat::Json
        }
    }
}

// ─── Commands ────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum Commands {
    /// Run a full security scan
    Scan {
        /// Target URL (e.g. https://example.com)
        url: String,

        /// Scan timeout in minutes
        #[arg(long, short, default_value = "10")]
        timeout: u64,

        /// CI mode — minimal stderr, structured output
        #[arg(long)]
        ci: bool,

        /// Enable cross-module correlation
        #[arg(long)]
        correlate: bool,

        /// CI metadata: repository name
        #[arg(long, hide = true)]
        meta_repo: Option<String>,

        /// CI metadata: commit SHA
        #[arg(long, hide = true)]
        meta_sha: Option<String>,

        /// CI metadata: git ref
        #[arg(long, hide = true)]
        meta_ref: Option<String>,

        /// CI metadata: run ID
        #[arg(long, hide = true)]
        meta_run_id: Option<String>,

        #[command(flatten)]
        fmt: FormatFlags,
    },

    /// Fetch results of a previous scan
    Result {
        /// Scan ID (UUID)
        scan_id: String,

        #[command(flatten)]
        fmt: FormatFlags,
    },

    /// List scan history for a target
    History {
        /// Target URL to filter (optional — shows all if omitted)
        url: Option<String>,

        /// Maximum number of results
        #[arg(long, short = 'n', default_value = "20")]
        limit: i64,
    },

    /// Compare two scans (delta report)
    Compare {
        /// Current scan ID (UUID)
        current: String,

        /// Previous scan ID (UUID)
        previous: String,

        /// Target URL (required for delta lookup)
        #[arg(long, short)]
        target: String,

        #[command(flatten)]
        fmt: FormatFlags,
    },

    /// Run website analyzer
    Analyze {
        /// Target URL
        url: String,
        #[command(flatten)]
        fmt: FormatFlags,
    },

    /// Run server investigator
    Investigate {
        /// Target URL
        url: String,
        #[command(flatten)]
        fmt: FormatFlags,
    },

    /// Run security audit
    Audit {
        /// Target URL
        url: String,
        #[command(flatten)]
        fmt: FormatFlags,
    },

    /// Run API discovery
    Discover {
        /// Target URL
        url: String,
        #[command(flatten)]
        fmt: FormatFlags,
    },

    /// Run service collector
    Services {
        /// Target URL
        url: String,
        #[command(flatten)]
        fmt: FormatFlags,
    },

    /// Run form mapper
    Forms {
        /// Target URL
        url: String,
        #[command(flatten)]
        fmt: FormatFlags,
    },

    /// Show Dynamic Rule Engine status
    Rules,

    /// Print version information
    Version,
}

// ─── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = scanner::LimmaClient::new(&cli.api_url, cli.api_key.as_deref())?;

    match cli.command {
        Commands::Scan {
            url,
            timeout,
            ci,
            correlate,
            meta_repo,
            meta_sha,
            meta_ref,
            meta_run_id,
            fmt,
        } => {
            cmd_scan(
                &client, &url, timeout, ci, correlate, meta_repo, meta_sha, meta_ref, meta_run_id,
                &fmt,
            )
            .await
        }
        Commands::Result { scan_id, fmt } => cmd_result(&client, &scan_id, &fmt).await,
        Commands::History { url, limit } => cmd_history(&client, url.as_deref(), limit).await,
        Commands::Compare {
            current,
            previous,
            target,
            fmt,
        } => cmd_compare(&client, &target, &current, &previous, &fmt).await,
        Commands::Analyze { url, fmt } => cmd_module(&client, "analyze", &url, &fmt).await,
        Commands::Investigate { url, fmt } => cmd_module(&client, "investigate", &url, &fmt).await,
        Commands::Audit { url, fmt } => cmd_module(&client, "audit-security", &url, &fmt).await,
        Commands::Discover { url, fmt } => cmd_module(&client, "discover-apis", &url, &fmt).await,
        Commands::Services { url, fmt } => cmd_module(&client, "collect-services", &url, &fmt).await,
        Commands::Forms { url, fmt } => cmd_module(&client, "map-forms", &url, &fmt).await,
        Commands::Rules => cmd_rules(&client).await,
        Commands::Version => {
            println!("limma {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Command Implementations
// ═══════════════════════════════════════════════════════════════════════════════

// ── scan ─────────────────────────────────────────────────────────────────────

async fn cmd_scan(
    client: &scanner::LimmaClient,
    url: &str,
    timeout: u64,
    ci: bool,
    correlate: bool,
    meta_repo: Option<String>,
    meta_sha: Option<String>,
    meta_ref: Option<String>,
    meta_run_id: Option<String>,
    fmt: &FormatFlags,
) -> Result<()> {
    if !ci {
        eprintln!("🔍 Limma Security Scan");
        eprintln!("======================");
        eprintln!("Target:  {}", url);
        eprintln!("Timeout: {}m", timeout);
        eprintln!();
    }

    let result = client
        .scan(
            url,
            timeout,
            ci,
            correlate,
            scanner::ScanMetadata {
                repo: meta_repo,
                sha: meta_sha,
                git_ref: meta_ref,
                run_id: meta_run_id,
            },
        )
        .await?;

    let output = match fmt.resolve() {
        formatter::OutputFormat::Json => formatter::format_json(&result)?,
        formatter::OutputFormat::Sarif => formatter::format_sarif(&result)?,
        formatter::OutputFormat::Markdown => formatter::format_markdown(&result)?,
    };

    write_output(&output, fmt.out.as_deref())?;

    // Summary to stderr
    if !ci {
        let findings = formatter::count_findings(&result);
        let p1 = formatter::count_by_severity(&result, "Critical");
        let p2 = formatter::count_by_severity(&result, "High");
        let score = formatter::extract_score(&result);

        eprintln!();
        eprintln!("📊 Summary: {}/100 score, {} findings ({}🔴 {}🟠)", score, findings, p1, p2);
        eprintln!("✅ Done");
    }

    Ok(())
}

// ── result ───────────────────────────────────────────────────────────────────

async fn cmd_result(
    client: &scanner::LimmaClient,
    scan_id: &str,
    fmt: &FormatFlags,
) -> Result<()> {
    eprintln!("📥 Fetching scan {}...", scan_id);

    let result = client.get_scan_by_id(scan_id).await?;

    let formatted = match fmt.resolve() {
        formatter::OutputFormat::Json => serde_json::to_string_pretty(&result)?,
        formatter::OutputFormat::Markdown => format_scan_detail_md(&result),
        formatter::OutputFormat::Sarif => serde_json::to_string_pretty(&result)?,
    };

    write_output(&formatted, fmt.out.as_deref())?;

    let score = result["score"].as_f64().unwrap_or(0.0);
    let endpoints = result["total_endpoints"].as_i64().unwrap_or(0);
    let findings = result["total_findings"].as_i64().unwrap_or(0);
    let target = result["target_url"].as_str().unwrap_or("—");

    eprintln!("📊 {} — {:.0}/100, {} endpoints, {} findings", target, score, endpoints, findings);

    Ok(())
}

fn format_scan_detail_md(result: &serde_json::Value) -> String {
    let target = result["target_url"].as_str().unwrap_or("unknown");
    let score = result["score"].as_f64().unwrap_or(0.0);

    let mut md = format!(
        "# 📋 Scan Results\n\n**Target:** `{}`  \n**Score:** {:.0}/100\n\n",
        target, score
    );

    if let Some(endpoints) = result["endpoints"].as_array() {
        if !endpoints.is_empty() {
            md.push_str("## Endpoints\n\n| Method | URL |\n|--------|-----|\n");
            for ep in endpoints {
                md.push_str(&format!(
                    "| {} | {} |\n",
                    ep["method"].as_str().unwrap_or("GET"),
                    ep["url"].as_str().unwrap_or("")
                ));
            }
            md.push('\n');
        }
    }

    if let Some(findings) = result["findings"].as_array() {
        if !findings.is_empty() {
            md.push_str("## Findings\n\n| Sev | Name | URL | Status |\n|-----|------|-----|--------|\n");
            for f in findings {
                let sev = f["severity"].as_str().unwrap_or("?");
                let icon = match sev {
                    "Critical" => "🔴", "High" => "🟠", "Medium" => "🟡", "Low" => "🟢", _ => "⚪"
                };
                md.push_str(&format!(
                    "| {}{} | {} | {} | {} |\n",
                    icon, sev,
                    f["name"].as_str().unwrap_or("—"),
                    f["url"].as_str().unwrap_or(""),
                    f["status"].as_str().unwrap_or("Open"),
                ));
            }
            md.push('\n');
        }
    }

    md
}

// ── history ──────────────────────────────────────────────────────────────────

async fn cmd_history(
    client: &scanner::LimmaClient,
    target_url: Option<&str>,
    limit: i64,
) -> Result<()> {
    eprintln!("📜 Scan history{}", target_url.map(|t| format!(" for {}", t)).unwrap_or_default());

    let scans = client.list_scans(target_url, Some(limit)).await?;
    let arr = scans.as_array().cloned().unwrap_or_default();

    if arr.is_empty() {
        println!("No scans found.");
        return Ok(());
    }

    println!(
        "{:<38} {:<12} {:>6} {:>10} {:>10}",
        "SCAN ID", "DATE", "SCORE", "ENDPOINTS", "FINDINGS"
    );
    println!("{}", "─".repeat(80));

    for s in &arr {
        let id = s["scan_id"].as_str().unwrap_or("—");
        let ts = s["timestamp_sec"].as_i64().unwrap_or(0);
        let date = chrono::DateTime::from_timestamp(ts, 0)
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "—".to_string());
        let score = s["score"].as_f64().unwrap_or(0.0);
        let ep = s["total_endpoints"].as_i64().unwrap_or(0);
        let fd = s["total_findings"].as_i64().unwrap_or(0);

        println!("{:<38} {:<12} {:>6.0} {:>10} {:>10}", id, date, score, ep, fd);
    }

    eprintln!("\n{} scans total", arr.len());
    Ok(())
}

// ── compare ──────────────────────────────────────────────────────────────────

async fn cmd_compare(
    client: &scanner::LimmaClient,
    target: &str,
    current: &str,
    previous: &str,
    fmt: &FormatFlags,
) -> Result<()> {
    eprintln!("🔄 {} vs {}", &current[..8.min(current.len())], &previous[..8.min(previous.len())]);

    let delta = client.get_delta(target, current, previous).await?;

    let formatted = match fmt.resolve() {
        formatter::OutputFormat::Json => serde_json::to_string_pretty(&delta)?,
        formatter::OutputFormat::Markdown => format_delta_md(&delta),
        formatter::OutputFormat::Sarif => serde_json::to_string_pretty(&delta)?,
    };

    write_output(&formatted, fmt.out.as_deref())?;

    let new_ep = delta["new_endpoints"].as_array().map(|a| a.len()).unwrap_or(0);
    let new_fd = delta["new_findings"].as_array().map(|a| a.len()).unwrap_or(0);
    let resolved = delta["resolved_findings"].as_array().map(|a| a.len()).unwrap_or(0);

    eprintln!("📊 +{} endpoints, +{} findings, -{} resolved", new_ep, new_fd, resolved);
    Ok(())
}

fn format_delta_md(delta: &serde_json::Value) -> String {
    let mut md = "# 🔄 Delta Report\n\n".to_string();

    for (title, icon, key) in [
        ("New Endpoints", "🆕", "new_endpoints"),
        ("New Findings", "🔺", "new_findings"),
        ("Resolved Findings", "✅", "resolved_findings"),
    ] {
        if let Some(arr) = delta[key].as_array() {
            md.push_str(&format!("## {} {} ({})\n\n", icon, title, arr.len()));
            if arr.is_empty() {
                md.push_str("*None*\n\n");
            } else if key == "new_endpoints" {
                md.push_str("| Method | URL |\n|--------|-----|\n");
                for item in arr {
                    md.push_str(&format!(
                        "| {} | {} |\n",
                        item["method"].as_str().unwrap_or("GET"),
                        item["url"].as_str().unwrap_or("")
                    ));
                }
                md.push('\n');
            } else {
                md.push_str("| Severity | Name | URL |\n|----------|------|-----|\n");
                for item in arr {
                    md.push_str(&format!(
                        "| {} | {} | {} |\n",
                        item["severity"].as_str().unwrap_or("?"),
                        item["name"].as_str().unwrap_or("—"),
                        item["url"].as_str().unwrap_or("")
                    ));
                }
                md.push('\n');
            }
        }
    }

    md
}

// ── module commands (analyze, investigate, audit, discover, services, forms) ─

async fn cmd_module(
    client: &scanner::LimmaClient,
    endpoint: &str,
    url: &str,
    fmt: &FormatFlags,
) -> Result<()> {
    let label = match endpoint {
        "analyze" => "Analyzer",
        "investigate" => "Investigator",
        "audit-security" => "Security Audit",
        "discover-apis" => "API Discovery",
        "collect-services" => "Service Collector",
        "map-forms" => "Form Mapper",
        _ => endpoint,
    };

    eprintln!("🔍 {} → {}", label, url);

    let result = match endpoint {
        "analyze" => client.analyze(url).await?,
        "investigate" => client.investigate(url).await?,
        "audit-security" => client.audit_security(url).await?,
        "discover-apis" => client.discover_apis(url).await?,
        "collect-services" => client.collect_services(url).await?,
        "map-forms" => client.map_forms(url).await?,
        _ => anyhow::bail!("Unknown module: {}", endpoint),
    };

    let formatted = match fmt.resolve() {
        formatter::OutputFormat::Json => serde_json::to_string_pretty(&result)?,
        formatter::OutputFormat::Markdown => format!(
            "# {} Report\n\n**Target:** `{}`\n\n```json\n{}\n```\n",
            label, url, serde_json::to_string_pretty(&result).unwrap_or_default()
        ),
        formatter::OutputFormat::Sarif => serde_json::to_string_pretty(&result)?,
    };

    write_output(&formatted, fmt.out.as_deref())?;
    eprintln!("✅ {}", label);
    Ok(())
}

// ── rules ────────────────────────────────────────────────────────────────────

async fn cmd_rules(client: &scanner::LimmaClient) -> Result<()> {
    let status = client.get_rule_engine_status().await?;

    let total = status["total_rules"].as_i64().unwrap_or(0);
    let active = status["active_rules"].as_array().map(|a| a.len()).unwrap_or(0);
    let load_err = status["load_errors"].as_array().map(|a| a.len()).unwrap_or(0);
    let val_err = status["validation_errors"].as_array().map(|a| a.len()).unwrap_or(0);

    println!("📏 Rule Engine — {} total, {} active", total, active);
    println!();

    if let Some(rules) = status["active_rules"].as_array() {
        if !rules.is_empty() {
            println!(
                "{:<28} {:<22} {:<14} {}",
                "ID", "NAME", "CATEGORY", "STATUS"
            );
            println!("{}", "─".repeat(72));
            for r in rules {
                let active = r["is_active"].as_bool().unwrap_or(false);
                println!(
                    "{:<28} {:<22} {:<14} {}",
                    trunc(r["id"].as_str().unwrap_or("—"), 26),
                    trunc(r["name"].as_str().unwrap_or("—"), 20),
                    r["category"].as_str().unwrap_or("—"),
                    if active { "✅" } else { "❌" }
                );
            }
        }
    }

    if let Some(stats) = status["feedback_stats"].as_object() {
        if !stats.is_empty() {
            println!("\n📊 Feedback:");
            for (id, s) in stats {
                println!(
                    "  {} — {}tp {}fp {}ign (rep: {:.2})",
                    trunc(id, 24),
                    s["confirmed"].as_i64().unwrap_or(0),
                    s["false_positives"].as_i64().unwrap_or(0),
                    s["ignored"].as_i64().unwrap_or(0),
                    s["reputation_score"].as_f64().unwrap_or(0.0)
                );
            }
        }
    }

    if load_err > 0 || val_err > 0 {
        println!();
        for (label, key) in [("Load errors", "load_errors"), ("Validation errors", "validation_errors")] {
            if let Some(errs) = status[key].as_array() {
                if !errs.is_empty() {
                    println!("⚠️  {}:", label);
                    for e in errs {
                        println!("  • {}", e.as_str().unwrap_or("?"));
                    }
                }
            }
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Utilities
// ═══════════════════════════════════════════════════════════════════════════════

fn write_output(content: &str, path: Option<&str>) -> Result<()> {
    if let Some(p) = path {
        std::fs::write(p, content)?;
        eprintln!("📄 Written to {}", p);
    } else {
        println!("{}", content);
    }
    Ok(())
}

fn trunc(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
