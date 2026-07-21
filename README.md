# deref

Replace symbolic links with real files and directories. Single binary, zero runtime dependencies beyond the OS.

## Install

### Pre-built binaries

Download from [GitHub Releases](https://github.com/someoneonsmile/deref/releases) (latest stable release).

| Platform                    | Asset                                                                                                                           |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Linux x86_64 (glibc)        | [`deref-x86_64-linux-gnu`](https://github.com/someoneonsmile/deref/releases/latest/download/deref-x86_64-linux-gnu)     |
| Linux x86_64 (musl, static) | [`deref-x86_64-linux-musl`](https://github.com/someoneonsmile/deref/releases/latest/download/deref-x86_64-linux-musl)    |
| Linux ARM64                 | [`deref-aarch64-linux-gnu`](https://github.com/someoneonsmile/deref/releases/latest/download/deref-aarch64-linux-gnu)    |
| macOS Intel                 | [`deref-x86_64-macos`](https://github.com/someoneonsmile/deref/releases/latest/download/deref-x86_64-macos)             |
| macOS Apple Silicon         | [`deref-aarch64-macos`](https://github.com/someoneonsmile/deref/releases/latest/download/deref-aarch64-macos)           |
| Windows x86_64              | [`deref-x86_64-windows.exe`](https://github.com/someoneonsmile/deref/releases/latest/download/deref-x86_64-windows.exe) |

```bash
# Linux / macOS example
curl -L -o /usr/local/bin/deref \
  https://github.com/someoneonsmile/deref/releases/latest/download/deref-x86_64-linux-gnu
chmod +x /usr/local/bin/deref
```

### From source

```bash
cargo install --path .
```

Requires Rust toolchain (1.85+ for edition 2024).

### AUR (Arch Linux)

```bash
# 稳定版
yay -S deref-bin
# 或
paru -S deref-bin

# 每夜构建（最新 commit）
yay -S deref-nightly-bin
# 或
paru -S deref-nightly-bin
```

## Usage

```bash
# Replace a single symlink with a real file
deref my-symlink

# Replace multiple symlinks
deref link1 link2 link3

# Recursively walk a directory and dereference all symlinks inside
deref -r ./project

# Dry run — see what would be done without making changes
deref -rn ./project

# Create .bak backups of original symlinks before replacing
deref -rb ./project

# Follow chains of symlinks to the final real target
deref -f chain-of-links

# Verbose output
deref -rv ./project

# Skip broken symlinks instead of erroring
deref -r --broken skip ./project

# Delete broken symlinks
deref -r --broken delete ./project

# Limit recursion depth
deref -rD 3 ./project
```

### CLI reference

```
$ deref --help
Replace symbolic links with real files / directories.

Usage: deref [OPTIONS] <PATH>...

Arguments:
  <PATH>...  One or more symlink paths to dereference

Options:
  -r, --recursive          Recursively walk directories, dereferencing all symlinks inside
  -n, --dry-run            Dry run — print what would be done without making changes
  -b, --backup             Keep original symlink as a .bak backup before replacing
  -f, --follow             Follow chains of symlinks to the final real target
  -D, --max-depth <N>      Maximum recursion depth (0 = unlimited) [default: 0]
      --broken <POLICY>    How to handle broken symlinks [default: error] [possible values: skip, delete, error]
  -v, --verbose            Verbose output (report every action)
  -h, --help               Print help
  -V, --version            Print version
```

## How it works

### File symlink

```
$ ls -l config.json
config.json -> ../templates/default.json

$ deref config.json       # or: deref -v config.json
config.json: → file (copy of ../templates/default.json)

$ ls -l config.json       # now a real file
-rw-r--r-- 1 user user 1234 Jul 21 10:00 config.json
```

### Directory symlink

```
$ ls -l assets
assets -> ../shared/assets/

$ deref assets

$ ls -l assets            # now a real directory
drwxr-xr-x 2 user user 4096 Jul 21 10:00 assets/
```

### Recursive mode

Walks a directory tree and dereferences every symlink found inside. Non-symlink entries are left untouched.

```
deref -rv my-project/
my-project/config.json: → file (copy of ../templates/default.json)
my-project/assets: → dir  (copy of ../shared/assets/)
my-project/README.md: not a symlink — skipped
```

### Backup mode (`-b`)

Renames the original symlink to `<name>.bak` (or `<name>.bak.1` if `.bak` exists) before creating the real file. Useful for reversible operations.
