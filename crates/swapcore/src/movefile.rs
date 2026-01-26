use anyhow::{bail, Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

fn tmp_path(dst: &Path) -> PathBuf {
  let mut os = dst.as_os_str().to_owned();
  os.push(".__swap_tmp");
  PathBuf::from(os)
}

pub fn move_one_streaming(src: &Path, dst: &Path, expected_size: u64, buffer_bytes: usize) -> Result<()> {
  if let Some(parent) = dst.parent() {
    fs::create_dir_all(parent)
      .with_context(|| format!("create_dir_all failed: {}", parent.display()))?;
  }

  if dst.exists() {
    bail!("Destination already exists: {}", dst.display());
  }

  let tmp = tmp_path(dst);
  if tmp.exists() {
    bail!("Temp destination already exists (stale?): {}", tmp.display());
  }

  let mut src_f = File::open(src).with_context(|| format!("open src failed: {}", src.display()))?;

  let mut tmp_f = OpenOptions::new()
    .create_new(true)
    .write(true)
    .open(&tmp)
    .with_context(|| format!("create tmp failed: {}", tmp.display()))?;

  let mut buf = vec![0u8; buffer_bytes.max(64 * 1024)];

  loop {
    let n = src_f
      .read(&mut buf)
      .with_context(|| format!("read failed: {}", src.display()))?;

    if n == 0 {
      break;
    }

    tmp_f
      .write_all(&buf[..n])
      .with_context(|| format!("write failed: {}", tmp.display()))?;
  }

  tmp_f.sync_all().ok();

  let tmp_size = fs::metadata(&tmp)
    .with_context(|| format!("metadata failed: {}", tmp.display()))?
    .len();

  if tmp_size != expected_size {
    bail!(
      "Copy verification failed: {} -> {} (expected {}, got {})",
      src.display(),
      dst.display(),
      expected_size,
      tmp_size
    );
  }

  fs::rename(&tmp, dst)
    .with_context(|| format!("rename failed: {} -> {}", tmp.display(), dst.display()))?;

  fs::remove_file(src).with_context(|| format!("remove src failed: {}", src.display()))?;
  Ok(())
}
