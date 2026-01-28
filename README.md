# Relocator

**Relocator** is a safe, deterministic directory-swapping tool designed for situations where you **don’t have enough free disk space to copy everything at once**.

It swaps two directories (typically on different disks) by **moving files one at a time**, always choosing the **largest file that fits**, dynamically freeing space until the swap is complete.

This tool exists for real problems like:

- Swapping large games between `C:` and `D:` on Windows
- Reorganizing data across disks with limited free space
- Avoiding temporary third storage

---

## Key Features

- **Largest-first greedy algorithm** (minimizes deadlocks)
- **True swap** without requiring full free space
- **Feasibility check** before touching your files
- **Dry-run mode** (see exactly what will happen)
- **Recursive directory handling** (files only)
- **Reserve space protection** (never fully fill a disk)
- **CLI-first**, scriptable, predictable
- **Windows-friendly** (native `.exe` releases)

---

## How the Algorithm Works (Short Version)

1. Scan both directories recursively
2. Sort files by size (largest → smallest)
3. Move the largest file from B → A **that fits**
4. If blocked, move files from A → B to free space
5. Repeat until all files are swapped
6. Abort safely if a deadlock is mathematically unavoidable

This is **not copying**. It is a controlled relocation.

---

## Installation

### Windows (recommended)

Download the latest release from GitHub:

https://github.com/xfeusw/relocator/releases

Unzip and use `swapdirs.exe`.

### Linux / NixOS (from source)

```bash
git clone https://github.com/xfeusw/relocator
cd relocator
nix develop
cargo build -p swapdirs --release
```

---

## Usage

### Dry run (strongly recommended)

```bash
swapdirs \
  --a-root "C:\games\game1" \
  --b-root "D:\games\game2" \
  --a-dest "D:\games\game1.._swapped" \
  --b-dest "C:\games\game2.._swapped" \
  --dry-run \
  --verbose
```

### Real run

```bash
swapdirs \
  --a-root "C:\games\game1" \
  --b-root "D:\games\game2" \
  --a-dest "D:\games\game1.._swapped" \
  --b-dest "C:\games\game2.._swapped" \
  --reserve-mib 1024 \
  --verbose
```

### Important flags

- `--dry-run`   No file movement, simulation only
- `--reserve-mib` Guaranteed free space left on each disk
- `--verbose`   Show every move

---

## Safety Notes (Read This)

- **Always run `--dry-run` first**
- Destination directories must be **empty or non-existent**
- Files are moved using a **streaming copy + atomic rename**
- No journaling yet (planned)
- If a deadlock is detected, the tool **aborts safely**

---

## Project Structure

```text
relocator/
├─ crates/
│  ├─ swapcore/   # core algorithm (no CLI)
│  └─ swapdirs/   # CLI frontend
├─ shell.nix
├─ flake.nix
├─ README.md
└─ CHANGELOG.md
```

---

## License

MIT © xfeusw
