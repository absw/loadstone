use std::{io, path::Path, process::Command};

pub fn prettify_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    Command::new("rustfmt")
        .arg(path.as_ref())
        .spawn()?
        .wait()?;
    Ok(())
}
