use anyhow::Context;
use clap::Parser;
use env_logger::Env;
use log::{debug, error, info};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use crate::cli::Cli;

mod cli;
mod cli_types;

fn main() {
    let cli = Cli::parse();

    let default_log_level = if cli.quiet {
        "error"
    } else {
        match cli.verbose {
            0 => "info",
            1 => "debug",
            _ => "trace",
        }
    };
    env_logger::Builder::from_env(Env::default().default_filter_or(default_log_level))
        .default_format()
        .format_level(true)
        .format_target(false)
        .format_module_path(false)
        .format_timestamp(None)
        .init();

    let broken = match cli.broken.as_str() {
        "skip" | "delete" | "error" => cli.broken.as_str(),
        _ => {
            error!("--broken must be 'skip', 'delete', or 'error'");
            process::exit(1);
        }
    };

    let mut has_error = false;
    for path in &cli.paths {
        if let Err(e) = process(path, 0, &cli, broken) {
            error!("{}: {}", path.display(), e);
            has_error = true;
        }
    }
    if has_error {
        process::exit(1);
    }
}

/// Process a single path. Errors are reported inline; returns Err to signal failure.
fn process(path: &Path, depth: usize, opts: &Cli, broken: &str) -> anyhow::Result<()> {
    if opts.max_depth > 0 && depth > opts.max_depth {
        return Ok(());
    }

    let meta = fs::symlink_metadata(path).with_context(|| format!("{}", path.display()))?;

    if meta.is_symlink() {
        if let Err(e) = deref(path, opts) {
            match broken {
                "skip" => {
                    info!("{}: broken symlink — skipped", path.display());
                }
                "delete" => {
                    let _ = report(path, "remove (broken symlink)", opts);
                    if let Err(rm_err) = fs::remove_file(path) {
                        return Err(
                            anyhow::Error::from(rm_err).context(format!("{}", path.display()))
                        );
                    }
                }
                _ => return Err(e).context(format!("{}", path.display())),
            }
        }
    } else if meta.is_dir() && opts.recursive {
        let entries: Vec<_> = fs::read_dir(path)
            .with_context(|| format!("{}", path.display()))?
            .map(|e| e.with_context(|| format!("{}", path.display())))
            .map(|r| r.map(|entry| entry.path()))
            .collect::<anyhow::Result<Vec<_>>>()?;
        for child in entries {
            if let Err(e) = process(&child, depth + 1, opts, broken) {
                error!("{}: {}", child.display(), e);
            }
        }
    } else {
        debug!("{}: not a symlink — skipped", path.display());
    }

    Ok(())
}

/// Resolve a symlink to its final real path.
/// When `follow` is false, resolves against the symlink's parent directory only.
/// When `follow` is true, walks through symlink chains (capped at 40 hops).
fn resolve_target(symlink: &Path, mut target: PathBuf, follow: bool) -> io::Result<PathBuf> {
    let parent = symlink.parent().unwrap_or(Path::new("."));
    let mut resolved = if target.is_absolute() {
        target
    } else {
        parent.join(&target)
    };

    if !follow {
        return Ok(resolved);
    }

    for _ in 0..40 {
        match fs::symlink_metadata(&resolved) {
            Ok(meta) if meta.is_symlink() => {
                target = fs::read_link(&resolved)?;
                let dir = resolved.parent().unwrap_or(Path::new("."));
                resolved = if target.is_absolute() {
                    target
                } else {
                    dir.join(&target)
                };
            }
            _ => break,
        }
    }

    Ok(resolved)
}

/// Find an available backup name by appending .bak, .bak.1, .bak.2, etc.
fn backup_name(path: &Path) -> PathBuf {
    let base = path.with_added_extension("bak");
    let mut candidate = base.clone();
    let mut n = 1u32;
    while candidate.exists() {
        candidate = base.with_added_extension(format!("{n}"));
        n += 1;
    }
    candidate
}

/// Core operation: replace a symlink at `path` with real content.
fn deref(path: &Path, opts: &Cli) -> anyhow::Result<()> {
    let link_target = fs::read_link(path)?;
    let real = resolve_target(path, link_target, opts.follow)
        .with_context(|| format!("broken symlink → cannot resolve target: {}", path.display()))?;

    if !real.exists() {
        anyhow::bail!("broken symlink → {} not found", real.display());
    }

    let real_meta = real.metadata()?;

    if real_meta.is_dir() {
        Ok(replace_symlink_with_dir(path, &real, opts)?)
    } else {
        Ok(replace_symlink_with_file(path, &real, opts)?)
    }
}

/// Replace a file-type symlink with a copy of the target file content.
fn replace_symlink_with_file(path: &Path, target: &Path, opts: &Cli) -> io::Result<()> {
    report(
        path,
        &format!("→ file (copy of {})", target.display()),
        opts,
    )?;

    if opts.dry_run {
        return Ok(());
    }

    if opts.backup {
        let bak = backup_name(path);
        fs::rename(path, &bak)?;
    } else {
        fs::remove_file(path)?;
    }

    // fs::copy preserves permissions on Unix
    fs::copy(target, path)?;

    Ok(())
}

/// Replace a directory-type symlink with a real directory containing copies of all contents.
fn replace_symlink_with_dir(path: &Path, target: &Path, opts: &Cli) -> io::Result<()> {
    report(
        path,
        &format!("→ dir  (copy of {})", target.display()),
        opts,
    )?;

    if opts.dry_run {
        return Ok(());
    }

    if opts.backup {
        let bak = backup_name(path);
        fs::rename(path, &bak)?;
    } else {
        // remove_file() works for symlinks even when they point to directories
        fs::remove_file(path)?;
    }

    fs::create_dir(path)?;
    copy_dir(target, path)?;

    Ok(())
}

/// Recursively copy directory contents (follows symlinks inside source).
fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let meta = entry.metadata()?; // follows symlinks

        if meta.is_dir() {
            fs::create_dir(&dst_path)?;
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Print action report.
fn report(path: &Path, action: &str, opts: &Cli) -> io::Result<()> {
    if opts.dry_run {
        println!("[dry-run] {}: {}", path.display(), action);
    } else {
        info!("{}: {}", path.display(), action);
    }
    Ok(())
}
