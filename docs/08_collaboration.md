# 步语 BuYu — 协作指南

**版本：** 0.2
**最后更新：** 2026-04-08

## 1. 开发环境

| 工具 | 要求 |
|------|------|
| Node.js | 22.x |
| pnpm | 10.x |
| Rust | stable |
| Tauri CLI | 2.x |

基础命令：

```bash
pnpm install
pnpm dev
pnpm tauri dev
pnpm verify
```

## 2. 分支与提交

| 类型 | 命名示例 |
|------|----------|
| 功能 | `feat/channel-bindings` |
| 修复 | `fix/message-empty-rollback` |
| 重构 | `refactor/model-service-validation` |
| 文档 | `docs/update-release-flow` |
| 构建/依赖 | `chore/upgrade-tauri-cli` |

提交格式：

```text
<type>: <description>
```

例如：

```text
feat: add version sync script
fix: keep release tag aligned with manifests
docs: refresh testing and release docs
```

## 3. Pull Request 要求

合并前至少满足：

- `pnpm verify` 通过
- 相关 API / 数据库 / 流程文档已同步
- 没有把本地数据库、构建目录或临时产物带进提交
- PR 描述说明了改动范围、验证方式和影响面

建议 PR 模板包含：

```markdown
## 改动
- ...

## 验证
- `pnpm verify`
- ...

## 影响
- ...
```

## 4. 版本与发布协作

发布前统一使用：

```bash
pnpm version:set -- 0.2.0
pnpm version:check
pnpm verify
```

发布流程：

1. 日常合入 `main` 后，`Release` 工作流会自动产出滚动预发布 `edge-main`。
2. 正式发版前，执行 `pnpm version:set -- <version>`、`pnpm version:check`、`pnpm verify`。
3. 提交版本变更。
4. 创建并推送正式 tag：`git tag v<version>`、`git push origin v<version>`。
5. GitHub `Release` 工作流在 tag 触发时校验 manifest 版本一致后，自动打包并发布正式产物。
6. 如需临时打包或只构建部分平台，可手动触发 `workflow_dispatch`。

## 5. GitHub Actions 约定

| 工作流 | 说明 |
|--------|------|
| `CI` | PR 和 `main` push 的常规门禁 |
| `Release` | 手动打包、`main` 滚动预发布、`v*` tag 正式发布 |

原则：

- 不维护“本地一套、CI 一套”的命令。
- 本地 `pnpm verify` 失败时不要推 PR 期待 CI 帮你找问题。
- 任何会影响发布的变更，都必须先保证 `pnpm version:check` 和 `pnpm verify` 可本地复现。
