//! Library half of the `beanify` CLI: argument parsing plus the recursive
//! path-walking and file-processing logic, kept here so it can be unit tested.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use beanifier_core::{BeanConfig, Beanifier};
use clap::{ArgGroup, Args, Parser};
use walkdir::WalkDir;

/// Mr-Beanify any path (file or directory) recursively into Mr-Bean-speak.
#[derive(Parser, Debug)]
#[command(name = "beanify", version, about, long_about = None)]
pub struct Cli {
    /// Paths (files or directories) to beanify. Directories are walked
    /// recursively.
    #[arg(required = true, value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    #[command(flatten)]
    pub bean: BeanArgs,

    #[command(flatten)]
    pub output: OutputArgs,

    /// Skip files larger than this many bytes.
    #[arg(long, default_value_t = 5_000_000, value_name = "N")]
    pub max_bytes: u64,

    /// Follow symbolic links while walking directories.
    #[arg(long)]
    pub follow_symlinks: bool,

    /// Report what would happen without writing anything.
    #[arg(long)]
    pub dry_run: bool,
}

/// Knobs forwarded to the beanification engine.
#[derive(Args, Debug)]
pub struct BeanArgs {
    /// Deterministic seed; the same seed always yields the same Bean-speak.
    #[arg(long, default_value_t = BeanConfig::default().seed)]
    pub seed: u64,

    /// Probability (0.0..=1.0) that a word becomes a signature Bean-ism.
    #[arg(long, default_value_t = BeanConfig::default().signature_frequency)]
    pub signature_frequency: f64,

    /// Maximum syllables in a generated mumble.
    #[arg(long, default_value_t = BeanConfig::default().max_syllables)]
    pub max_syllables: usize,

    /// Do not reapply the source word's UPPER/Title/lower casing.
    #[arg(long)]
    pub no_preserve_case: bool,
}

impl BeanArgs {
    /// Build the engine configuration from parsed flags.
    pub fn to_config(&self) -> BeanConfig {
        BeanConfig {
            seed: self.seed,
            signature_frequency: self.signature_frequency,
            max_syllables: self.max_syllables.max(1),
            preserve_case: !self.no_preserve_case,
        }
    }
}

/// Where beanified output should go. The two flags are mutually exclusive;
/// with neither set, output is streamed to stdout.
#[derive(Args, Debug)]
#[command(group(ArgGroup::new("sink").args(["in_place", "output"])))]
pub struct OutputArgs {
    /// Rewrite each input file in place.
    #[arg(long)]
    pub in_place: bool,

    /// Mirror the input tree into this directory, writing beanified copies.
    #[arg(long, value_name = "DIR")]
    pub output: Option<PathBuf>,
}

/// Resolved destination for beanified content.
enum Sink {
    Stdout { headers: bool },
    InPlace,
    Dir(PathBuf),
}

/// Tally of a beanification run.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Summary {
    /// Files successfully beanified.
    pub processed: usize,
    /// Files skipped (too large, non-UTF-8, etc.).
    pub skipped: usize,
    /// Files that errored during processing.
    pub errors: usize,
}

/// Entry point used by `main`: parse-free execution given a [`Cli`].
///
/// `out` receives streamed content (stdout mode) and is otherwise unused.
pub fn run(cli: &Cli, out: &mut impl Write) -> Result<Summary> {
    let config = cli.bean.to_config();
    let beanifier = Beanifier::new(config);

    let sink = if cli.output.in_place {
        Sink::InPlace
    } else if let Some(dir) = &cli.output.output {
        Sink::Dir(dir.clone())
    } else {
        let has_dir = cli.paths.iter().any(|p| p.is_dir());
        Sink::Stdout {
            headers: has_dir || cli.paths.len() > 1,
        }
    };

    let mut summary = Summary::default();
    for path in &cli.paths {
        if !path.exists() {
            bail!("path does not exist: {}", path.display());
        }
        process_input(path, &beanifier, &sink, cli, out, &mut summary)?;
    }
    Ok(summary)
}

/// Process one top-level input path (file or directory).
fn process_input(
    path: &Path,
    beanifier: &Beanifier,
    sink: &Sink,
    cli: &Cli,
    out: &mut impl Write,
    summary: &mut Summary,
) -> Result<()> {
    if path.is_file() {
        // A lone file mirrors under its own name.
        let root = path.parent().unwrap_or_else(|| Path::new(""));
        process_file(path, root, beanifier, sink, cli, out, summary);
        return Ok(());
    }

    for entry in WalkDir::new(path)
        .follow_links(cli.follow_symlinks)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            process_file(entry.path(), path, beanifier, sink, cli, out, summary);
        }
    }
    Ok(())
}

/// Beanify a single file and route the result to the sink.
///
/// Errors are recorded in the summary rather than aborting the whole run —
/// one unreadable file should not doom an entire tree.
fn process_file(
    file: &Path,
    root: &Path,
    beanifier: &Beanifier,
    sink: &Sink,
    cli: &Cli,
    out: &mut impl Write,
    summary: &mut Summary,
) {
    match process_file_inner(file, root, beanifier, sink, cli, out) {
        Ok(Outcome::Processed) => summary.processed += 1,
        Ok(Outcome::Skipped) => summary.skipped += 1,
        Err(err) => {
            summary.errors += 1;
            eprintln!("beanify: {}: {err:#}", file.display());
        }
    }
}

enum Outcome {
    Processed,
    Skipped,
}

fn process_file_inner(
    file: &Path,
    root: &Path,
    beanifier: &Beanifier,
    sink: &Sink,
    cli: &Cli,
    out: &mut impl Write,
) -> Result<Outcome> {
    let bytes = fs::read(file).with_context(|| format!("reading {}", file.display()))?;

    if bytes.len() as u64 > cli.max_bytes {
        return Ok(Outcome::Skipped);
    }

    // Only UTF-8 text can be beanified. Non-text files are mirrored verbatim in
    // directory mode and skipped otherwise.
    let content = match std::str::from_utf8(&bytes) {
        Ok(text) => text,
        Err(_) => {
            if let Sink::Dir(dir) = sink {
                if !cli.dry_run {
                    let dest = mirror_path(dir, root, file)?;
                    ensure_parent(&dest)?;
                    fs::write(&dest, &bytes)
                        .with_context(|| format!("copying {}", dest.display()))?;
                }
            }
            return Ok(Outcome::Skipped);
        }
    };

    let beanified = beanifier.beanify_text(content);

    if cli.dry_run {
        eprintln!("beanify: would beanify {}", file.display());
        return Ok(Outcome::Processed);
    }

    match sink {
        Sink::Stdout { headers } => {
            if *headers {
                writeln!(out, "==> {} <==", file.display())?;
            }
            out.write_all(beanified.as_bytes())?;
            if !beanified.ends_with('\n') {
                writeln!(out)?;
            }
        }
        Sink::InPlace => {
            fs::write(file, beanified.as_bytes())
                .with_context(|| format!("writing {}", file.display()))?;
        }
        Sink::Dir(dir) => {
            let dest = mirror_path(dir, root, file)?;
            ensure_parent(&dest)?;
            fs::write(&dest, beanified.as_bytes())
                .with_context(|| format!("writing {}", dest.display()))?;
        }
    }

    Ok(Outcome::Processed)
}

/// Map a source file to its destination under `dir`, preserving the tree shape
/// relative to `root`.
fn mirror_path(dir: &Path, root: &Path, file: &Path) -> Result<PathBuf> {
    let rel = file.strip_prefix(root).unwrap_or_else(|_| {
        // Falls back to the bare file name when `file` is not under `root`.
        Path::new(file.file_name().unwrap_or(file.as_os_str()))
    });
    Ok(dir.join(rel))
}

/// Create the parent directory of `path` if needed.
fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    Ok(())
}

/// Convenience wrapper that streams to real stdout.
pub fn run_stdout(cli: &Cli) -> Result<Summary> {
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    let summary = run(cli, &mut lock)?;
    lock.flush().ok();
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn cli_for(paths: Vec<PathBuf>) -> Cli {
        Cli {
            paths,
            bean: BeanArgs {
                seed: BeanConfig::default().seed,
                signature_frequency: BeanConfig::default().signature_frequency,
                max_syllables: BeanConfig::default().max_syllables,
                no_preserve_case: false,
            },
            output: OutputArgs {
                in_place: false,
                output: None,
            },
            max_bytes: 5_000_000,
            follow_symlinks: false,
            dry_run: false,
        }
    }

    #[test]
    fn cli_parses() {
        use clap::Parser;
        let cli =
            Cli::try_parse_from(["beanify", "--seed", "7", "--in-place", "some/path"]).unwrap();
        assert_eq!(cli.bean.seed, 7);
        assert!(cli.output.in_place);
        assert_eq!(cli.paths, vec![PathBuf::from("some/path")]);
    }

    #[test]
    fn in_place_and_output_conflict() {
        use clap::Parser;
        let res = Cli::try_parse_from(["beanify", "--in-place", "--output", "o", "p"]);
        assert!(res.is_err());
    }

    #[test]
    fn single_file_to_stdout() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("hello.txt");
        fs::write(&file, "hello world").unwrap();

        let cli = cli_for(vec![file]);
        let mut buf = Vec::new();
        let summary = run(&cli, &mut buf).unwrap();

        assert_eq!(summary.processed, 1);
        let out = String::from_utf8(buf).unwrap();
        assert!(!out.contains("hello"));
        assert!(!out.starts_with("==>")); // no header for a single file
    }

    #[test]
    fn recurses_into_directories_in_place() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("a.txt"), "alpha beta").unwrap();
        fs::write(dir.path().join("sub/b.txt"), "gamma delta").unwrap();

        let mut cli = cli_for(vec![dir.path().to_path_buf()]);
        cli.output.in_place = true;
        let mut buf = Vec::new();
        let summary = run(&cli, &mut buf).unwrap();

        assert_eq!(summary.processed, 2);
        let a = fs::read_to_string(dir.path().join("a.txt")).unwrap();
        let b = fs::read_to_string(dir.path().join("sub/b.txt")).unwrap();
        assert!(!a.contains("alpha"));
        assert!(!b.contains("gamma"));
        // Whitespace structure preserved: single space between two words.
        assert_eq!(a.matches(' ').count(), 1);
    }

    #[test]
    fn output_dir_mirrors_tree() {
        let src = tempdir().unwrap();
        let dst = tempdir().unwrap();
        fs::create_dir_all(src.path().join("nested")).unwrap();
        fs::write(src.path().join("nested/x.txt"), "one two three").unwrap();

        let mut cli = cli_for(vec![src.path().to_path_buf()]);
        cli.output.output = Some(dst.path().to_path_buf());
        let mut buf = Vec::new();
        let summary = run(&cli, &mut buf).unwrap();

        assert_eq!(summary.processed, 1);
        let mirrored = dst.path().join("nested/x.txt");
        assert!(mirrored.exists());
        assert!(!fs::read_to_string(&mirrored).unwrap().contains("one"));
        // Source untouched.
        assert_eq!(
            fs::read_to_string(src.path().join("nested/x.txt")).unwrap(),
            "one two three"
        );
    }

    #[test]
    fn skips_oversized_files() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("big.txt");
        fs::write(&file, "way too big").unwrap();

        let mut cli = cli_for(vec![file]);
        cli.max_bytes = 3;
        let mut buf = Vec::new();
        let summary = run(&cli, &mut buf).unwrap();
        assert_eq!(summary.processed, 0);
        assert_eq!(summary.skipped, 1);
    }

    #[test]
    fn non_utf8_is_copied_verbatim_in_dir_mode() {
        let src = tempdir().unwrap();
        let dst = tempdir().unwrap();
        let bin = src.path().join("data.bin");
        fs::write(&bin, [0xff, 0xfe, 0x00, 0x01]).unwrap();

        let mut cli = cli_for(vec![src.path().to_path_buf()]);
        cli.output.output = Some(dst.path().to_path_buf());
        let mut buf = Vec::new();
        let summary = run(&cli, &mut buf).unwrap();

        assert_eq!(summary.skipped, 1);
        let mirrored = dst.path().join("data.bin");
        assert_eq!(fs::read(&mirrored).unwrap(), [0xff, 0xfe, 0x00, 0x01]);
    }

    #[test]
    fn dry_run_writes_nothing() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("c.txt");
        fs::write(&file, "keep me original").unwrap();

        let mut cli = cli_for(vec![file.clone()]);
        cli.output.in_place = true;
        cli.dry_run = true;
        let mut buf = Vec::new();
        let summary = run(&cli, &mut buf).unwrap();

        assert_eq!(summary.processed, 1);
        assert_eq!(fs::read_to_string(&file).unwrap(), "keep me original");
    }

    #[test]
    fn missing_path_errors() {
        let cli = cli_for(vec![PathBuf::from("/definitely/not/here/xyz")]);
        let mut buf = Vec::new();
        assert!(run(&cli, &mut buf).is_err());
    }

    #[test]
    fn multiple_files_get_headers_on_stdout() {
        let dir = tempdir().unwrap();
        let f1 = dir.path().join("1.txt");
        let f2 = dir.path().join("2.txt");
        fs::write(&f1, "aaa").unwrap();
        fs::write(&f2, "bbb").unwrap();

        let cli = cli_for(vec![f1, f2]);
        let mut buf = Vec::new();
        run(&cli, &mut buf).unwrap();
        assert!(String::from_utf8(buf).unwrap().contains("==>"));
    }
}
