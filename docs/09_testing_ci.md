# 步语 BuYu — 测试与 CI

**版本：** 0.4  
**最后更新：** 2026-04-04

## 1. 当前自动化门禁

### 前端

```bash
pnpm check
pnpm test
pnpm build
```

### Rust

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked -j 1
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -j 1 -- -A clippy::result_large_err -A clippy::too_many_arguments -D warnings
```

### 版本一致性

```bash
node scripts/version.mjs check
```

## 2. package.json 中的统一入口

| 命令 | 作用 |
|------|------|
| `pnpm ci:frontend` | 执行前端类型检查、测试、生产构建 |
| `pnpm ci:rust` | 执行 Rust 测试和 `clippy` |
| `pnpm ci:version` | 检查三处 manifest 版本一致 |
| `pnpm verify` | 按顺序执行完整门禁 |

这些命令是本地和 GitHub Actions 共享的事实来源。

## 3. GitHub Actions 工作流

### `CI`

触发：

- `pull_request`
- push 到 `main`

职责：

1. `Version Check`
   校验 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`
2. `Frontend Checks`
   执行 `pnpm check`、`pnpm test`、`pnpm build`
3. `Rust Checks`
   执行单任务 `cargo test`、`cargo clippy`，降低 Windows runner 的内存峰值；当前显式豁免 `result_large_err` 和 `too_many_arguments` 两类既有技术债，其余 warning 继续阻断

### `Release`

触发：

- `workflow_dispatch`
- push `v*` tag

职责：

- 先执行 `Build Frontend`，同时完成 tag 版本校验、前端检查、测试和 `dist` 产物上传
- 手动触发时：
  进入按平台和架构展开的独立 job，并行构建：
  `windows-x64`、`windows-arm64`、`linux-x64`、`linux-arm64`、`macos-x64`、`macos-arm64`、`android-arm64`、`android-armv7`、`android-x86_64`、`android-x86`；`iOS` 仅在 Apple mobile 工程存在时启用
- tag 触发时：
  `Create Release` job 会统一下载各架构产物并上传到 GitHub Release
- 移动端：
  `Android` 为每个 ABI 单独执行 `pnpm tauri android init --ci` 与 APK 构建；`iOS` 通过前置检测决定是否进入发布链路

## 4. 版本控制规则

- 版本号必须同时存在且保持一致：
  `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`
- 发布 tag 统一使用 `v<semver>`
- 使用 `pnpm version:set -- <version>` 修改版本，不手工改三份文件

## 5. 本地发布前检查

```bash
pnpm version:check
pnpm verify
pnpm tauri build
```

## 6. 后续保留项

当前没有加入的内容：

- Tauri 窗口级 E2E
- 覆盖率门槛阻断
- 已签名的 iOS 安装包发布

说明：发布工作流现在采用 `Build Frontend -> Per-Arch Build Jobs -> Create Release` 结构；`iOS` 仍取决于 Apple 工程初始化与签名材料。
