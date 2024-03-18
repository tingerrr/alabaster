use std::path::Path;

use clap::Parser;
use config::{ManifestConfig, Mapping};
use same_file::is_same_file;
use typst_project::manifest::Manifest;

mod cli;
mod config;

fn copy_entry_shallow(from: &Path, to: &Path) -> anyhow::Result<()> {
    eprintln!("{:<40} -> {}", from.display(), to.display());

    let metadata = from.metadata()?;

    if metadata.is_file() {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::copy(from, to)?;
    } else if metadata.is_dir() {
        std::fs::create_dir_all(to)?;
    } else {
        anyhow::bail!("unable to handle {:?} yet ('{}')", metadata, from.display());
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    let pwd = std::env::current_dir()?;
    let root = if let Some(root) = args.root {
        root
    } else {
        pwd.clone()
    };

    let manifest =
        Manifest::try_find(&root)?.ok_or_else(|| anyhow::anyhow!("no manifest found"))?;

    let config = manifest
        .tool
        .as_ref()
        .map(|t| t.get_section::<ManifestConfig>("alabaster"))
        .transpose()?
        .flatten();

    match args.cmd {
        cli::Command::Package { force, output } => {
            if output.try_exists()? {
                if force {
                    std::fs::remove_dir_all(&output)?;
                    std::fs::create_dir_all(&output)?;
                } else if std::fs::read_dir(&output)?.next().is_some() {
                    anyhow::bail!(
                        "output directory '{}' already exists and isn't empty",
                        output.display()
                    );
                }
            } else {
                std::fs::create_dir_all(&output)?;
            }

            // TODO: simply disallow that this is part of the search path, for now this is required
            // for the entry filter
            let output = &*Box::leak(output.into_boxed_path());

            if let Some(config) = config {
                if !config.map.is_empty() {
                    eprintln!("mappings:");
                }

                for Mapping { from, to } in config.map {
                    let from = root.join(from);
                    let to = output.join(to);

                    copy_entry_shallow(&from, &to)?;
                }
            }

            let walk = ignore::WalkBuilder::new(&root)
                .require_git(true)
                // NOTE: trying to simply have the same EXACT lines inside an exclude array in the
                // tool section and adding them as overrides after inverting them works for all but
                // paths containing slashes it seems
                //
                // since ignore files work as expected and can also be read on the fly we only
                // those for now
                .add_custom_ignore_filename(".alabasterignore")
                // TODO: add options for granualr control here, this specifically is annoying in
                // helix, which reads these if enabled but has no own ignorefile name, for a helix
                // user and ignore file should not be included, but other users may see this as a
                // better default
                .ignore(false)
                .filter_entry(move |e| !is_same_file(e.path(), output).unwrap())
                .build();

            eprintln!("copies:");
            for entry in walk {
                let entry = entry?;

                if entry.file_type().is_some_and(|t| t.is_dir()) {
                    continue;
                }

                let from = entry.path();
                let to = output.join(
                    entry
                        .path()
                        .strip_prefix(&root)
                        .expect("is canonical in root"),
                );

                copy_entry_shallow(&from, &to)?;
            }
        }
    }

    Ok(())
}
