//! Command-line interface definitions

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rm-patcher")]
#[command(author, version, about = "reMarkable xochitl firmware patcher")]
#[command(long_about = "
Binary patcher for reMarkable xochitl firmware.

Enables hidden features, removes telemetry, and allows customization
of the reMarkable tablet firmware. Creates automatic backups and
supports rollback.

SAFETY: Always test patches on a development device first.
Patches are provided for educational purposes.
")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze a xochitl binary and show patch status
    Analyze {
        /// Path to xochitl binary
        #[arg(value_name = "BINARY")]
        binary: PathBuf,
    },

    /// Apply patches to a xochitl binary
    Patch {
        /// Path to xochitl binary
        #[arg(value_name = "BINARY")]
        binary: PathBuf,

        /// Output path (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Patches to apply (by name or ID)
        #[arg(short, long, value_name = "PATCH")]
        patches: Vec<String>,

        /// Apply all available patches
        #[arg(long)]
        all: bool,

        /// Skip creating backup
        #[arg(long)]
        no_backup: bool,
    },

    /// List all available patches
    List,

    /// Verify a xochitl binary
    Verify {
        /// Path to xochitl binary
        #[arg(value_name = "BINARY")]
        binary: PathBuf,
    },

    /// Restore from a backup
    Restore {
        /// Path to backup file
        #[arg(value_name = "BACKUP")]
        backup: PathBuf,
    },

    /// Extract embedded resources (QML, images, etc.)
    Extract {
        /// Path to xochitl binary
        #[arg(value_name = "BINARY")]
        binary: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "extracted")]
        output: PathBuf,
    },
}
