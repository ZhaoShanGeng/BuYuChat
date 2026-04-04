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
- push 到 `main`
- push `v*` tag

职责：

- 手动触发时可以通过输入项勾选 `Windows / Linux / macOS / Android / iOS`，并提供自定义 `release_tag`；若留空则默认取 `package.json` 当前版本生成 `v<version>`
- push 到 `main` 时：
  会自动全平台全架构构建，并把最新产物发布到滚动预发布 `edge-main`
- 当 `publish_release=true` 时，工作流会自动创建或更新对应的 GitHub Release
- 先执行 `Build Frontend`，同时完成 tag 版本校验、前端检查、测试和 `dist` 产物上传
- 手动触发时：
  进入按平台和架构展开的独立 job，并行构建：
  `windows-x64`、`windows-arm64`、`linux-x64`、`linux-arm64`、`macos-x64`、`macos-arm64`、`android-arm64`、`android-armv7`、`android-x86_64`、`android-x86`；`iOS` 仅在 Apple mobile 工程存在时启用
- `main` 触发时：
  `Create Release` job 会先把 `edge-main` tag 强制移动到当前提交，再统一上传最新产物，保持首页 Releases 始终有一份滚动预发布包
- tag 触发时：
  `Create Release` job 会统一下载各架构产物并上传到正式 GitHub Release
- 移动端：
  `Android` 会先执行一次 `Prepare Android project`，把初始化后的工程作为 artifact 分发给各 ABI job，避免每个 ABI 重复 `pnpm tauri android init --ci`；`iOS` 通过前置检测决定是否进入发布链路
- 移动端网络：
  Android 初始化与恢复工程后都会执行 `pnpm mobile:sync-network`，自动补 `INTERNET`、`usesCleartextTraffic` 和 `network_security_config`；iOS 通过 `src-tauri/Info.ios.plist` 合并 `NSAppTransportSecurity`，以支持 `http / https` 渠道
- 构建缓存：
  桌面端、Android 与 iOS job 都启用 `sccache` 和 `rust-cache`；Android 额外复用 `Gradle` 缓存
- Android 签名：
  `build-android` 会在构建后使用仓库 Secrets 中的 keystore 对 APK 进行 `zipalign + apksigner` 签名；必需的 secrets 为 `ANDROID_KEYSTORE_BASE64`、`ANDROID_KEYSTORE_PASSWORD`、`ANDROID_KEY_ALIAS`、`ANDROID_KEY_PASSWORD`

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
