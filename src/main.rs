mod commands;
mod formats;
mod inspect;
mod scrub;
mod util;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io;
use std::path::PathBuf;
use util::config::AppConfig;
use util::output::OutputOpts;
use util::xdg::XdgPaths;

#[derive(Parser, Debug)]
#[command(
    name = "metaclean",
    author = "r3dg0d",
    version,
    about = "Metadata inspector and scrubber for images, audio, video, PDF, and Office documents",
    long_about = "Inspect and scrub privacy-sensitive metadata (EXIF GPS, timestamps, authors, \
ID3, PDF Info, OOXML core props, …).\n\n\
Default behaviour never overwrites originals — writes a scrubbed copy with a suffix, or to \
--output. Use --in-place (with automatic backup under ~/.cache/metaclean/backups/) to replace."
)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, short = 'v', global = true)]
    verbose: bool,
    #[arg(long, short = 'q', global = true)]
    quiet: bool,
    #[arg(long, global = true, env = "METACLEAN_CONFIG")]
    config: Option<PathBuf>,
    #[arg(long, global = true)]
    dry_run: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Inspect metadata in a file
    Inspect {
        file: PathBuf,
    },
    /// Scrub metadata (writes copy by default)
    Scrub {
        file: PathBuf,
        /// Recurse into directory
        #[arg(long, short = 'r')]
        recursive: bool,
        /// Overwrite original (backs up to XDG cache)
        #[arg(long)]
        in_place: bool,
        /// Explicit output path (single file mode)
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
    },
    /// Verify remaining sensitive metadata after scrubbing
    Verify {
        file: PathBuf,
    },
    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn init_tracing(verbose: bool, quiet: bool) {
    let level = if quiet {
        "error"
    } else if verbose {
        "debug"
    } else {
        "info"
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
        )
        .with_writer(std::io::stderr)
        .try_init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose, cli.quiet);
    let out = OutputOpts {
        json: cli.json,
        quiet: cli.quiet,
        verbose: cli.verbose,
    };

    if let Commands::Completions { shell } = &cli.command {
        let mut cmd = Cli::command();
        generate(*shell, &mut cmd, "metaclean", &mut io::stdout());
        return Ok(());
    }

    let paths = XdgPaths::new()?;
    let cfg_path = cli
        .config
        .clone()
        .unwrap_or_else(|| paths.default_config_file());
    let cfg = AppConfig::load(Some(&cfg_path))?;

    match cli.command {
        Commands::Inspect { file } => commands::inspect::run(&out, &cfg, &file)?,
        Commands::Scrub {
            file,
            recursive,
            in_place,
            output,
        } => commands::scrub::run(
            &out,
            &cfg,
            &paths,
            &file,
            recursive,
            in_place,
            output.as_deref(),
            cli.dry_run,
        )?,
        Commands::Verify { file } => commands::verify::run(&out, &cfg, &file)?,
        Commands::Completions { .. } => unreachable!(),
    }
    Ok(())
}
