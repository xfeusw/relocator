use anyhow::{bail, Context, Result};
use fs2::available_space;
use std::path::Path;

pub fn free_bytes(path: &Path) -> Result<u64> {
  let mut cur = path;

  loop {
    if cur.exists() {
      return available_space(cur).context("available_space failed");
    }
    match cur.parent() {
      Some(p) => cur = p,
      None => break,
    }
  }

  bail!(
    "free_bytes: could not find an existing parent for path {}",
    path.display()
  )
}
