//! Writes the OpenAPI document.
//!
//! The document is built from the routers themselves, with no database, no configuration and
//! no listening socket, so the frontend client can be regenerated from a plain checkout and
//! CI can check the committed spec is current.

use std::{fs, io::Write as _, path::PathBuf};

use anyhow::Context as _;

fn main() -> anyhow::Result<()> {
    let document = app::api::openapi()
        .to_pretty_json()
        .context("could not render the OpenAPI document")?;

    match std::env::args().nth(1) {
        Some(path) => {
            let path = PathBuf::from(path);

            if let Some(parent) = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                fs::create_dir_all(parent)
                    .with_context(|| format!("could not create {}", parent.display()))?;
            }

            // Trailing newline, so the file is well-formed text and does not show up in every
            // diff as a "no newline at end of file".
            fs::write(&path, format!("{document}\n"))
                .with_context(|| format!("could not write {}", path.display()))?;

            eprintln!("wrote {}", path.display());
        }
        None => {
            let mut stdout = std::io::stdout().lock();
            writeln!(stdout, "{document}").context("could not write to stdout")?;
        }
    }

    Ok(())
}
