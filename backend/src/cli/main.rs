mod formatter;
mod scanner;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "limma")]
#[command(version)]
#[command(about = "Limma Security Recon — CI/CD Scanner CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a security reconnaissance scan against a target URL
    Scan(ScanArgs),
    /// Fetch results of a previous scan by ID
    Results {
        /// Scan ID to retrieve
        #[arg(long)]
        scan_id: String,
    },
    /// Compare two scans to produce a delta report
    Delta {
        /// Current scan ID
        #[arg(long)]
        current: String,
        /// Previous scan ID
        #[arg(long)]
        previous: String,
    },
}

#[derive(Parser)]
struct ScanArgs {
    /// Target URL to scan (e.g., https://staging.example.com)
    #[arg(long, short)]
    target: String,

    /// Scan timeout in minutes
    #[arg(long, default_value = "10")]
    timeout: u64,

    /// Output format: json, sarif, markdown
    #[arg(long, value_enum, default_value = "json")]
    format: formatter::OutputFormat,

    /// Enable CI mode (minimal output, structured for automation)
    #[arg(long)]
    ci_mode: bool,

    /// Enable cross-module correlation analysis
    #[arg(long)]
    enable_correlation: bool,

    /// API key for authenticated scans (placeholder — not yet enforced)
    #[arg(long)]
    api_key: Option<String>,

    /// Write results to a file instead of stdout
    #[arg(long, short)]
    output: Option<String>,

    /// Backend API base URL (defaults to http://localhost:8900)
    #[arg(long, default_value = "http://localhost:8900")]
    api_url: Option<String>,

    /// Repository name metadata (injected by GitHub Action)
    #[arg(long)]
    metadata_repo: Option<String>,

    /// Commit SHA metadata
    #[arg(long)]
    metadata_sha: Option<String>,

    /// Git ref metadata
    #[arg(long)]
    metadata_ref: Option<String>,

    /// Run ID metadata
    #[arg(long)]
    metadata_run_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan(args) => run_scan(args).await,
        Commands::Results { scan_id } => get_results(scan_id).await,
        Commands::Delta { current, previous } => compare_scans(current, previous).await,
    }
}

async fn run_scan(args: ScanArgs) -> Result<()> {
    let base_url = args.api_url.as_deref().unwrap_or("http://localhost:8900");
    let client = scanner::LimmaClient::new(base_url, args.api_key.as_deref())?;

    eprintln!("🔍 Limma Security Recon — CI/CD Scanner");
    eprintln!("==========================================");
    eprintln!("Target:  {}", args.target);
    eprintln!("Timeout: {}m", args.timeout);
    eprintln!("Format:  {:?}", args.format);
    eprintln!();

    let scan_result = client
        .scan(
            &args.target,
            args.timeout,
            args.ci_mode,
            args.enable_correlation,
            scanner::ScanMetadata {
                repo: args.metadata_repo,
                sha: args.metadata_sha,
                git_ref: args.metadata_ref,
                run_id: args.metadata_run_id,
            },
        )
        .await?;

    // Format output
    let output = match args.format {
        formatter::OutputFormat::Json => formatter::format_json(&scan_result)?,
        formatter::OutputFormat::Sarif => formatter::format_sarif(&scan_result)?,
        formatter::OutputFormat::Markdown => formatter::format_markdown(&scan_result)?,
    };

    // Write to file or stdout
    if let Some(output_path) = args.output {
        std::fs::write(&output_path, &output)?;
        eprintln!("📄 Results written to: {}", output_path);
    } else {
        println!("{}", output);
    }

    // Print summary to stderr (so it's visible even when stdout is piped to a file)
    let findings_count = formatter::count_findings(&scan_result);
    let p1 = formatter::count_by_severity(&scan_result, "Critical");
    let p2 = formatter::count_by_severity(&scan_result, "High");
    let score = formatter::extract_score(&scan_result);

    eprintln!();
    eprintln!("📊 Scan Summary");
    eprintln!("================");
    eprintln!("Findings:     {}", findings_count);
    eprintln!("P1 (Critical): {}", p1);
    eprintln!("P2 (High):     {}", p2);
    eprintln!("Score:         {}/100", score);
    eprintln!();
    eprintln!("✅ Scan completed successfully");

    Ok(())
}

async fn get_results(scan_id: String) -> Result<()> {
    eprintln!("📥 Fetching results for scan: {}", scan_id);
    // Placeholder — will be implemented when persistent scan storage is ready
    eprintln!("⚠️  Scan result retrieval is not yet implemented. Coming soon.");
    Ok(())
}

async fn compare_scans(current: String, previous: String) -> Result<()> {
    eprintln!(
        "🔄 Comparing scans: {} (current) vs {} (previous)",
        current, previous
    );
    // Placeholder — will leverage DeltaEngine when scan persistence is ready
    eprintln!("⚠️  Delta comparison is not yet implemented. Coming soon.");
    Ok(())
}
