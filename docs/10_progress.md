# 步语 BuYu — 进度追踪

**最后更新：** 2026-04-04

## 总体状态

| 领域 | 状态 | 说明 |
|------|------|------|
| 前端工作台 | 🔨 进行中 | 主路径已接通，仍在做体验和状态层收敛 |
| Rust 后端 | ✅ 可用 | 渠道、模型、Agent、会话、消息、生成与 Reroll 已接通 |
| 测试基线 | ✅ 可用 | `pnpm test`、`cargo test`、`cargo clippy` 已纳入统一脚本 |
| CI | ✅ 已落地 | GitHub Actions 已覆盖版本校验、前端门禁、Rust 门禁 |
| 发布 | ✅ 已落地 | 发布链路已收敛为 `Build Frontend -> Prepare Android project -> Per-Arch Build Jobs -> Create Release`；桌面端与 Android 均按架构拆分为独立 job，并启用构建缓存 |
| 版本治理 | ✅ 已落地 | 三处 manifest 版本统一由脚本校验/修改 |

## 当前已完成的基础设施

| 项目 | 状态 | 备注 |
|------|------|------|
| `scripts/version.mjs` | ✅ | 统一检查/更新版本号 |
| `pnpm verify` | ✅ | 本地与 CI 共享一套门禁入口 |
| `.github/workflows/ci.yml` | ✅ | PR 与 `main` 常规校验 |
| `.github/workflows/release.yml` | ✅ | `Build Frontend` 产出前端构建物，随后按 `windows/linux/macos/android` 的具体架构拆分独立 job，并由 `Create Release` 统一收口；iOS 继续按 Apple 工程条件启用 |
| 仓库清理规则 | ✅ | 移除/忽略生成型配置与本地调试残留 |

## 仍在推进的功能面

| 模块 | 状态 | 下一步 |
|------|------|--------|
| 流式生成体验 | 🔨 | 继续压缩竞态与切会话边界问题 |
| 前端状态层测试 | 🔨 | 补更多工作台和消息版本回归用例 |
| 安全与运维项 | 🔲 | API Key 管理、SSRF 防护、日志策略 |

## 近期重点

1. 继续收口前端联调细节，减少“功能能跑但边界不稳”的状态。
2. 在 CI 已稳定的前提下，补更细的回归用例，而不是继续依赖人工验证。
3. 补 Apple mobile 工程与签名材料，接通 iOS 自动打包与发布。

## 状态图例

| 符号 | 含义 |
|------|------|
| 🔲 | 未开始 |
| 🔨 | 进行中 |
| ✅ | 完成 |
