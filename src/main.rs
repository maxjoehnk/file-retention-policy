use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use clap::Parser;
pub(crate) use color_eyre::eyre::Result;

pub(crate) use crate::args::Args;
use crate::args::SubCommand;
use crate::config::Config;
use crate::file::RetentionFile;

mod args;
mod config;
mod policy;
mod file;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let args: Args = Args::parse();
    tracing::debug!(?args);

    let config = if let Some(config_path) = args.config.as_ref() {
        Config::read(config_path)
    } else {
        Config::read("config.toml")
    }?;
    tracing::debug!(?config);

    let context = ExecutionContext::new(args);

    for path in config.paths {
        let policy = path.retention.unwrap_or(config.retention);
        let files = context.read_files(&path.path)?;

        tracing::trace!(?policy, ?files);

        let (files, err_files) = files
            .into_iter()
            .map(|filename| RetentionFile::new(filename, &path.file_pattern))
            .partition::<Vec<_>, _>(|file| file.is_ok());

        let mut files: Vec<_> = files.into_iter()
            .map(|result| result.unwrap())
            .collect();

        files.sort_by_key(|file| file.date);
        files.reverse();

        for err in err_files {
            tracing::warn!(?err, "Unable to parse file");
        }

        let (keep, drop) = policy.retain(files);

        context.drop_files(&path.path, keep, drop)?;
    }

    Ok(())
}

enum ExecutionContext {
    Default,
    DryRun,
    Simulate {
        path: PathBuf,
        input: Option<PathBuf>,
    }
}

impl ExecutionContext {
    fn new(args: Args) -> Self {
        if let Some(SubCommand::Simulate { input, path }) = args.command {
            Self::Simulate {
                path,
                input
            }
        }else if args.dry_run {
            Self::DryRun
        } else {
            Self::Default
        }
    }

    fn read_files(&self, path: impl AsRef<Path>) -> Result<Vec<String>> {
        let path = path.as_ref();
        match self {
            Self::Default | Self::DryRun | Self::Simulate { path: _, input: None } => {
                let files = fs::read_dir(path)?
                    .flat_map(|dir| {
                        if let Err(err) = dir.as_ref() {
                            tracing::warn!("Error while reading directory {path:?}: {err:?}");
                        }

                        dir.ok()
                    })
                    .map(|dir| dir.file_name().to_string_lossy().to_string())
                    .collect();

                Ok(files)
            },
            Self::Simulate { path: target_path, input: Some(input) } if path == target_path => {
                let file = File::open(input)?;
                let reader = BufReader::new(file);

                let files = reader.lines()
                    .flat_map(|line| {
                        if let Err(err) = line.as_ref() {
                            tracing::warn!("Error while reading line from input file {input:?}: {err:?}");
                        }

                        line.ok()
                    })
                    .collect();

                Ok(files)
            }
            Self::Simulate { .. } => Ok(Default::default())
        }
    }

    fn drop_files(&self, path: impl AsRef<Path>, keep: Vec<RetentionFile>, drop: Vec<RetentionFile>) -> Result<()> {
        match self {
            Self::Default => self.delete_files(path, drop),
            Self::Simulate { .. } | Self::DryRun => self.file_report(keep, drop),
        }
    }

    fn delete_files(&self, path: impl AsRef<Path>, files: Vec<RetentionFile>) -> Result<()> {
        let path = path.as_ref();
        for file in files {
            let file_path = path.join(file.filename);
            if !file_path.exists() {
                continue;
            }
            if file_path.is_dir() {
                fs::remove_dir_all(file_path)?;
            } else {
                fs::remove_file(file_path)?;
            }
        }
        Ok(())
    }

    fn file_report(&self, keep: Vec<RetentionFile>, drop: Vec<RetentionFile>) -> Result<()> {
        let keep: Vec<_> = keep.into_iter().map(|file| file.filename).collect();
        let drop: Vec<_> = drop.into_iter().map(|file| file.filename).collect();
        tracing::info!(?keep, "Keeping files");
        tracing::info!(?drop, "Dropping files");

        Ok(())
    }
}
