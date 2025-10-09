use std::path::PathBuf;
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    // show the relative path of OUT_DIR.
    // this is relative so as to accomodate foundry access control.
    // this wouldn't work in a release build, but that's not the use case here.
    let cwd = std::env::current_dir()?;
    let dir = PathBuf::from_str(env!("OUT_DIR"))?;
    let dir = dir.strip_prefix(cwd)?;
    println!("{}", dir.to_string_lossy());
    Ok(())
}
