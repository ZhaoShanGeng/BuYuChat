# 步语 BuYu

基于 `Tauri v2 + Svelte 5 + Rust + SQLite` 的 AI 对话桌面客户端。
当前后端使用自建 `OpenAI-compatible` 适配层，通过 `reqwest` 对接聊天补全、模型拉取和流式 SSE；同时内置基础工具调用与 MCP 接入能力。

## 本地开发

```bash
pnpm install

# 前端开发服务器
pnpm dev

# 桌面开发模式
pnpm tauri dev
```

## 本地校验

```bash
# 前端类型检查 + 测试 + 构建
pnpm ci:frontend

# Rust 测试 + clippy
pnpm ci:rust

# 检查 package.json / Cargo.toml / tauri.conf.json 版本一致
pnpm version:check

# 全量门禁
pnpm verify
```

## 打包与发布

```bash
# 本地打包 Windows 安装包
pnpm tauri build

# 统一修改三个版本号文件
pnpm version:set -- 0.2.0
```

GitHub Actions 已落地两条主流程：

- `CI`：在 PR 和 `main` push 上执行版本一致性检查、前端测试/构建、Rust 测试与 `clippy`
- `Release`：先执行 `Build Frontend`，然后按平台与架构展开独立 job：`windows-x64`、`windows-arm64`、`linux-x64`、`linux-arm64`、`macos-x64`、`macos-arm64`、`android-arm64`、`android-armv7`、`android-x86_64`、`android-x86`；最后由 `Create Release` 统一上传 Release 资产，`iOS` 仅在 Apple mobile 工程存在时启用

移动端说明：

- `Android` job 现在会在 GitHub Actions 里自动执行 `pnpm tauri android init --ci` 后再打包 APK
- `iOS` 通过工作流里的检测 job 判断仓库是否已提交 `src-tauri/gen/apple`；只有存在 Apple mobile 工程时才启用，因为该链路必须在 macOS 上运行，并且正式分发还依赖 Apple 签名材料

发布约定：

1. 先执行 `pnpm version:set -- <version>`。
2. 提交版本变更并打 tag：`git tag v<version>`。
3. 推送 tag 后由 `Release` 工作流生成 GitHub Release，并自动附带 `Windows / Linux / macOS` 安装包。

## 许可证

本项目采用 `GNU AGPL-3.0`，完整文本见 [LICENSE](LICENSE)。

如果你修改本项目并通过网络向用户提供服务，需要按照 AGPL 的要求向这些用户提供对应源码。

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Svelte 5 + TypeScript + Tailwind CSS 4 + bits-ui |
| 桌面壳 | Tauri v2 |
| 后端 | Rust + SQLite (`sqlx`) |
| AI 接入 | 自建 `OpenAI-compatible` 适配层（`reqwest` + SSE 解析） |
| 工具能力 | 内置 `fetch` 工具 + MCP 模块 |
| 包管理 | pnpm 10 |

## 文档

文档入口见 [docs/00_index.md](docs/00_index.md)。

重点文档：

- [docs/02_architecture.md](docs/02_architecture.md)
- [docs/08_collaboration.md](docs/08_collaboration.md)
- [docs/09_testing_ci.md](docs/09_testing_ci.md)
- [docs/10_progress.md](docs/10_progress.md)
