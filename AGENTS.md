# AGENTS.md — deref

Replace symbolic links with real files / directories. Single-binary Rust CLI (~220 LOC in `src/main.rs`).

## Commands (use `just`)

```bash
just check        # cargo check (fast, no binary)
just build        # cargo build (debug)
just release      # cargo build --release
just test         # cargo test (currently 0 tests)
just fmt          # cargo fmt
just fmt-check    # cargo fmt --check
just lint         # cargo clippy
just lint-strict  # cargo clippy -- -D warnings
just fix          # cargo clippy --fix --allow-dirty
just ci           # full gate: fmt-check → lint-strict → test → release build
just install      # cargo build --release → /usr/local/bin/deref
```

Run with args: `just run -- --help` or `just run-release -- --help`.

## Toolchain

- **Rust edition 2024** → requires Rust **≥1.85**
- CI uses `dtolnay/rust-toolchain@stable` with `rustfmt` + `clippy` components
- No custom `rustfmt.toml` or `clippy.toml` — all tool defaults

## CI / Release

| Workflow | Trigger | What it does |
|---|---|---|
| `ci.yml` | push/PR to `main` | `fmt-check`, `lint-strict`, `cargo test` |
| `release.yml` | tag `v*` | calls `build.yml` (all targets), publishes GitHub Release, auto-updates AUR |
| `nightly.yml` | daily cron | same as release but tagged `nightly` (prerelease, rolling), auto-updates nightly AUR |

### Build matrix (release)

| Target | OS | Note |
|---|---|---|
| `x86_64-unknown-linux-gnu` | ubuntu-latest | native |
| `x86_64-unknown-linux-musl` | ubuntu-latest | uses `cross` tool, not cargo |
| `aarch64-unknown-linux-gnu` | ubuntu-24.04-arm | native |
| `x86_64-apple-darwin` | macos-latest | native |
| `aarch64-apple-darwin` | macos-latest | native |
| `x86_64-pc-windows-msvc` | windows-latest | native |

Artifact naming: `deref-{target}.exe` on Windows, `deref-{target}` elsewhere.

## Architecture

Single file `src/main.rs`:

- **`main()`**: parse args, iterate over paths, report aggregate errors
- **`process()`**: entry point — checks symlink metadata, dispatches to `deref()` or recurses into directories
- **`deref()`**: core logic — reads symlink target, resolves it (optionally following chains), dispatches to file or directory replacement
- **`resolve_target()`**: resolves symlink target relative to parent, optionally walks chains (capped at 40 hops)
- **`replace_symlink_with_file()`**: removes symlink, copies target content, preserves permissions
- **`replace_symlink_with_dir()`**: removes symlink, creates real directory, recursively copies target contents
- **`copy_dir()`**: recursive directory copy using `entry.metadata()` (follows symlinks inside source)
- **`backup_name()`**: generates collision-free backup filename (.bak, .bak.1, .bak.2, ...)
- **`report()`**: prints action in verbose or dry-run mode

Dependencies: `clap` (derive) — only dependency, everything else is std.

## AUR

项目维护两个 AUR 包：

| 包名 | 子目录 | 远程 remote | 发布触发 |
|---|---|---|---|
| `deref-bin` (稳定版) | `aur/` | `aur` → `deref-bin.git` | `release.yml` (tag `v*`) |
| `deref-nightly-bin` (每夜构建) | `aur-nightly/` | `aur-nightly` → `deref-nightly-bin.git` | `nightly.yml` (每日 cron) |

两者通过「clone AUR 仓库 → 复制 `PKGBUILD`/`.SRCINFO` → commit → push」维护（AUR 禁止 force push，clone 保证 push 可 fast-forward），共享同一个 AUR 账户的 SSH key。

```bash
# 稳定版
just aur-srcinfo                # 重新生成 aur/.SRCINFO
just aur-release VERSION        # 更新版本号 + 推送

# 每夜构建
just aur-nightly-srcinfo        # 重新生成 aur-nightly/.SRCINFO
just aur-nightly-release DATE   # 更新日期版本号（YYYYMMDD）+ 推送
```

### Automated publishing

`release.yml` 和 `nightly.yml` 各自的 `aur` job 会在发布后自动更新对应 AUR 包：

1. 更新 `PKGBUILD` 中的 `pkgver`（稳定版用 tag 版本号，nightly 用 `CARGO_VER+nightly+日期+g短SHA`）、重置 `pkgrel=1`
2. 用 Arch Linux Docker 容器运行 `makepkg --printsrcinfo` 生成 `.SRCINFO`
3. clone 对应 AUR 仓库，复制 `PKGBUILD` + `.SRCINFO`，commit 后 push 到 AUR 仓库

**前置条件**：GitHub 仓库需要配置 secret `AUR_SSH_PRIVATE_KEY`（对应已上传到 AUR 账户的公钥）。

## Conventions

- Commit messages: `type: 中文描述` (e.g., `feat: 添加递归目录支持`, `fix: 修复损坏链接处理`)
- PR branch → `main`; only release tags (`v*`) trigger publishing
- The `justfile` is the authoritative source for dev commands — prefer `just <cmd>` over raw cargo
