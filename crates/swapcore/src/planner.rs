use crate::movefile::move_one_streaming;
use crate::scan::{max_file, scan_files, total_size, FileEntry};
use crate::space::free_bytes;
use anyhow::{bail, Result};
use bytesize::ByteSize;
use std::path::{PathBuf};

fn human(b: u64) -> String {
  ByteSize(b).to_string()
}

#[derive(Clone, Debug)]
pub struct SwapConfig {
  pub a_root: PathBuf,
  pub b_root: PathBuf,
  pub a_dest: PathBuf,
  pub b_dest: PathBuf,
  pub reserve_bytes: u64,
  pub buffer_bytes: usize,
  pub verbose: bool,
}

#[derive(Clone, Debug)]
pub struct FeasibilityInfo {
  pub ok: bool,
  pub summary: String,
  pub deficit_c: u64,
  pub deficit_d: u64,
}

#[derive(Clone, Debug)]
pub struct SwapReport {
  pub summary: String,
  pub moved_a_bytes: u64,
  pub moved_b_bytes: u64,
  pub moved_a_files: u64,
  pub moved_b_files: u64,
}

pub fn feasibility_check(cfg: &SwapConfig) -> Result<FeasibilityInfo> {
  let a_files = scan_files(&cfg.a_root)?;
  let b_files = scan_files(&cfg.b_root)?;

  let s_a = total_size(&a_files);
  let s_b = total_size(&b_files);
  let m_a = max_file(&a_files);
  let m_b = max_file(&b_files);

  let f_c0 = free_bytes(&cfg.b_dest)?;
  let f_d0 = free_bytes(&cfg.a_dest)?;

  let d_max = f_d0.saturating_add(s_b);
  let c_max = f_c0.saturating_add(s_a);

  let deficit_d = s_a.saturating_sub(d_max);
  let deficit_c = s_b.saturating_sub(c_max);

  let mut lines = Vec::new();
  lines.push("=== Feasibility ===".to_string());
  lines.push(format!("A_total: {}", human(s_a)));
  lines.push(format!("B_total: {}", human(s_b)));
  lines.push(format!("A_max_file: {}", human(m_a)));
  lines.push(format!("B_max_file: {}", human(m_b)));
  lines.push(format!("free(C for B_dest): {}", human(f_c0)));
  lines.push(format!("free(D for A_dest): {}", human(f_d0)));
  lines.push(format!("D_max_free = free_D + B_total = {}", human(d_max)));
  lines.push(format!("C_max_free = free_C + A_total = {}", human(c_max)));
  lines.push(format!("reserve: {}", human(cfg.reserve_bytes)));
  lines.push("".to_string());

  if deficit_d > 0 {
    lines.push(format!(
      "IMPOSSIBLE: D can never fit A (need +{} more free on D, or third storage).",
      human(deficit_d)
    ));
    return Ok(FeasibilityInfo {
      ok: false,
      summary: lines.join("\n"),
      deficit_c,
      deficit_d,
    });
  }
  if deficit_c > 0 {
    lines.push(format!(
      "IMPOSSIBLE: C can never fit B (need +{} more free on C, or third storage).",
      human(deficit_c)
    ));
    return Ok(FeasibilityInfo {
      ok: false,
      summary: lines.join("\n"),
      deficit_c,
      deficit_d,
    });
  }

  if m_a > d_max {
    lines.push(format!(
      "IMPOSSIBLE: A contains a file of {} that can never fit on D (D_max_free is {}).",
      human(m_a),
      human(d_max)
    ));
    return Ok(FeasibilityInfo {
      ok: false,
      summary: lines.join("\n"),
      deficit_c,
      deficit_d,
    });
  }
  if m_b > c_max {
    lines.push(format!(
      "IMPOSSIBLE: B contains a file of {} that can never fit on C (C_max_free is {}).",
      human(m_b),
      human(c_max)
    ));
    return Ok(FeasibilityInfo {
      ok: false,
      summary: lines.join("\n"),
      deficit_c,
      deficit_d,
    });
  }

  lines.push("OK: swap is feasible by capacity laws.".to_string());

  Ok(FeasibilityInfo {
    ok: true,
    summary: lines.join("\n"),
    deficit_c,
    deficit_d,
  })
}

fn pick_largest_that_fits(files: &[FileEntry], done: &[bool], free_eff: u64) -> Option<usize> {
  for (i, f) in files.iter().enumerate() {
    if !done[i] && f.size <= free_eff {
        return Some(i);
    }
  }
  None
}

fn all_done(done: &[bool]) -> bool {
  done.iter().all(|&x| x)
}

fn effective_free(free: u64, reserve: u64) -> u64 {
  free.saturating_sub(reserve)
}

pub fn run_dry(cfg: &SwapConfig) -> Result<SwapReport> {
  let a_files = scan_files(&cfg.a_root)?;
  let b_files = scan_files(&cfg.b_root)?;
  let s_a = total_size(&a_files);
  let s_b = total_size(&b_files);

  let f_c0 = free_bytes(&cfg.b_dest)?;
  let f_d0 = free_bytes(&cfg.a_dest)?;

  let mut free_c = f_c0;
  let mut free_d = f_d0;

  let mut done_a = vec![false; a_files.len()];
  let mut done_b = vec![false; b_files.len()];

  let mut moved_a_bytes = 0u64;
  let mut moved_b_bytes = 0u64;
  let mut moved_a_files = 0u64;
  let mut moved_b_files = 0u64;

  loop {
    if all_done(&done_a) && all_done(&done_b) {
      break;
    }

    while !all_done(&done_b) {
      let free_c_eff = effective_free(free_c, cfg.reserve_bytes);
      let idx = pick_largest_that_fits(&b_files, &done_b, free_c_eff);
      let Some(i) = idx else { break };
      let f = &b_files[i];

      free_c = free_c.saturating_sub(f.size);
      free_d = free_d.saturating_add(f.size);

      done_b[i] = true;
      moved_b_bytes += f.size;
      moved_b_files += 1;

      if cfg.verbose {
        println!("DRY  B->C  {}  {}", human(f.size), f.rel.display());
      }
    }

    if all_done(&done_b) && !all_done(&done_a) {
      while !all_done(&done_a) {
        let free_d_eff = effective_free(free_d, cfg.reserve_bytes);
        let idx_a = pick_largest_that_fits(&a_files, &done_a, free_d_eff);
        let Some(i) = idx_a else {
          bail!(
            "DEADLOCK (dry-run): B finished but remaining A file doesn't fit on D. free_D_eff={}",
            human(free_d_eff)
          );
        };

        let g = &a_files[i];

        free_d = free_d.saturating_sub(g.size);
        free_c = free_c.saturating_add(g.size);

        done_a[i] = true;
        moved_a_bytes += g.size;
        moved_a_files += 1;

        if cfg.verbose {
          println!("DRY A->D {} {}", human(g.size), g.rel.display());
        }
      }
      break;
    }

    while !all_done(&done_b) {
      let free_c_eff = effective_free(free_c, cfg.reserve_bytes);
      if pick_largest_that_fits(&b_files, &done_b, free_c_eff).is_some() {
        break;
      }

      let free_d_eff = effective_free(free_d, cfg.reserve_bytes);
      let idx_a = pick_largest_that_fits(&a_files, &done_a, free_d_eff);
      let Some(i) = idx_a else { break };
      let g = &a_files[i];

      free_d = free_d.saturating_sub(g.size);
      free_c = free_c.saturating_add(g.size);

      done_a[i] = true;
      moved_a_bytes += g.size;
      moved_a_files += 1;

      if cfg.verbose {
        println!("DRY  A->D  {}  {}", human(g.size), g.rel.display());
      }
    }

    let free_c_eff = effective_free(free_c, cfg.reserve_bytes);
    let free_d_eff = effective_free(free_d, cfg.reserve_bytes);

    let blocked_b = !all_done(&done_b) && pick_largest_that_fits(&b_files, &done_b, free_c_eff).is_none();
    let blocked_a = !all_done(&done_a) && pick_largest_that_fits(&a_files, &done_a, free_d_eff).is_none();

    if blocked_a && blocked_b {
      bail!(
        "DEADLOCK (dry-run): no remaining file fits on either side. free_C_eff={} free_D_eff={}",
        human(free_c_eff),
        human(free_d_eff)
      );
    }
  }

  Ok(SwapReport {
    summary: format!(
        "DRY-RUN complete.\nMoved B->C: {} ({} files) / {}\nMoved A->D: {} ({} files) / {}",
        human(moved_b_bytes),
        moved_b_files,
        human(s_b),
        human(moved_a_bytes),
        moved_a_files,
        human(s_a),
    ),
    moved_a_bytes,
    moved_b_bytes,
    moved_a_files,
    moved_b_files,
  })
}

pub fn run_real(cfg: &SwapConfig) -> Result<SwapReport> {
  let a_files = scan_files(&cfg.a_root)?;
  let b_files = scan_files(&cfg.b_root)?;

  let s_a = total_size(&a_files);
  let s_b = total_size(&b_files);

  let mut done_a = vec![false; a_files.len()];
  let mut done_b = vec![false; b_files.len()];

  let mut moved_a_bytes = 0u64;
  let mut moved_b_bytes = 0u64;
  let mut moved_a_files = 0u64;
  let mut moved_b_files = 0u64;

  loop {
    if all_done(&done_a) && all_done(&done_b) {
      break;
    }

    while !all_done(&done_b) {
        let free_c = free_bytes(&cfg.b_dest)?;
        let free_c_eff = effective_free(free_c, cfg.reserve_bytes);

        let idx = pick_largest_that_fits(&b_files, &done_b, free_c_eff);
        let Some(i) = idx else { break };

        let f = &b_files[i];
        let dst = cfg.b_dest.join(&f.rel);

        if cfg.verbose {
            println!("MOVE B->C  {}  {}", human(f.size), f.rel.display());
        }
        move_one_streaming(&f.abs, &dst, f.size, cfg.buffer_bytes)?;

        done_b[i] = true;
        moved_b_bytes += f.size;
        moved_b_files += 1;
    }

    if all_done(&done_a) && all_done(&done_b) {
      break;
    }

    while !all_done(&done_b) {
        let free_c = free_bytes(&cfg.b_dest)?;
        let free_c_eff = effective_free(free_c, cfg.reserve_bytes);
        if pick_largest_that_fits(&b_files, &done_b, free_c_eff).is_some() {
            break;
        }

        let free_d = free_bytes(&cfg.a_dest)?;
        let free_d_eff = effective_free(free_d, cfg.reserve_bytes);

        let idx_a = pick_largest_that_fits(&a_files, &done_a, free_d_eff);
        let Some(i) = idx_a else { break };

        let g = &a_files[i];
        let dst = cfg.a_dest.join(&g.rel);

        if cfg.verbose {
          println!("MOVE A->D  {}  {}", human(g.size), g.rel.display());
        }
        move_one_streaming(&g.abs, &dst, g.size, cfg.buffer_bytes)?;

        done_a[i] = true;
        moved_a_bytes += g.size;
        moved_a_files += 1;
    }

    if all_done(&done_b) && !all_done(&done_a) {
      while !all_done(&done_a) {
        let free_d = free_bytes(&cfg.a_dest)?;
        let free_d_eff = effective_free(free_d, cfg.reserve_bytes);

        let idx_a = pick_largest_that_fits(&a_files, &done_a, free_d_eff);
        let Some(i) = idx_a else {
          bail!(
            "DEADLOCK: B finished but remaining A file doesn't fit on D. free_D_eff={}",
            human(free_d_eff)
          );
        };

        let g = &a_files[i];
        let dst = cfg.a_dest.join(&g.rel);

        if cfg.verbose {
          println!("MOVE A->D {} {}", human(g.size), g.rel.display());
        }
        move_one_streaming(&g.abs, &dst, g.size, cfg.buffer_bytes)?;

        done_a[i] = true;
        moved_a_bytes += g.size;
        moved_a_files += 1;
      }
      break;
    }

    let free_c = free_bytes(&cfg.b_dest)?;
    let free_d = free_bytes(&cfg.a_dest)?;
    let free_c_eff = effective_free(free_c, cfg.reserve_bytes);
    let free_d_eff = effective_free(free_d, cfg.reserve_bytes);

    let blocked_b = !all_done(&done_b) && pick_largest_that_fits(&b_files, &done_b, free_c_eff).is_none();
    let blocked_a = !all_done(&done_a) && pick_largest_that_fits(&a_files, &done_a, free_d_eff).is_none();

    if blocked_a && blocked_b {
        bail!(
            "DEADLOCK: neither side has a file that fits.\nfree_C_eff={} free_D_eff={}\nTip: free space or use third storage.",
            human(free_c_eff),
            human(free_d_eff)
        );
    }
  }

  Ok(SwapReport {
    summary: format!(
        "REAL-RUN complete.\nMoved B->C: {} ({} files) / {}\nMoved A->D: {} ({} files) / {}",
        human(moved_b_bytes),
        moved_b_files,
        human(s_b),
        human(moved_a_bytes),
        moved_a_files,
        human(s_a),
    ),
    moved_a_bytes,
    moved_b_bytes,
    moved_a_files,
    moved_b_files,
  })
}
