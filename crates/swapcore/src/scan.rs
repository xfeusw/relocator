use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct FileEntry {
  pub abs: PathBuf,
  pub rel: PathBuf,
  pub size: u64
}

pub fn scan_files(root: &Path) -> Result<Vec<FileEntry>> {
  let mut out = Vec::new();

  for entry in WalkDir::new(root).follow_links(false) {
    let entry = entry.context("walkdir entry failed")?;

    if !entry.file_type().is_file() {
      continue;
    }

    let abs = entry.path().to_path_buf();
    let rel = abs
      .strip_prefix(root)
      .with_context(|| format!("strip_prefix failed for {}", abs.display()))?
      .to_path_buf();

    let md = entry
      .metadata()
      .with_context(|| format!("metadata failed for {}", abs.display()))?;

    out.push(FileEntry {
      abs,
      rel,
      size: md.len(),
    });
  }

  // Biggest-first
  out.sort_by(|a, b| b.size.cmp(&a.size));
  Ok(out)
}

pub fn total_size(files: &[FileEntry]) -> u64 {
  files.iter().map(|f| f.size).sum()
}

pub fn max_file(files: &[FileEntry]) -> u64 {
  files.first().map(|f| f.size).unwrap_or(0)
}
