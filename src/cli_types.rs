// ⚠️ 此文件被 build.rs 通过 include! 引入，禁止添加 use crate:: 依赖。
// 如需 crate 级别引用，请在 cli.rs 中添加。

use std::path::PathBuf;

use clap::Parser;

/// Replace symbolic links with real files / directories.
#[derive(Parser)]
#[command(name = "deref", about, version)]
pub struct Cli {
    /// One or more symlink paths to dereference
    #[arg(required = true, value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Recursively walk directories, dereferencing all symlinks inside
    #[arg(short, long)]
    pub recursive: bool,

    /// Dry run — print what would be done without making changes
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Keep original symlink as a .bak backup before replacing
    #[arg(short, long)]
    pub backup: bool,

    /// Follow chains of symlinks to the final real target
    #[arg(short, long)]
    pub follow: bool,

    /// Maximum recursion depth (0 = unlimited)
    #[arg(short = 'D', long, default_value_t = 0, value_name = "N")]
    pub max_depth: usize,

    /// How to handle broken symlinks [default: error]
    #[arg(long, default_value = "error", value_name = "POLICY")]
    pub broken: String,

    /// Increase log verbosity (-v debug, -vv trace)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Quiet mode, only output errors
    #[arg(short = 'q', long = "quiet", action = clap::ArgAction::SetTrue, conflicts_with = "verbose")]
    pub quiet: bool,
}
