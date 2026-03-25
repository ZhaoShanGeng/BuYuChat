# 步语 BuYu — 协作指南

**版本：** 0.1

所有参与开发的人请先读完这篇。

---

## 1. 开发环境

### 必需工具

| 工具 | 版本 | 用途 |
|------|------|------|
| Node.js | ≥ 18 | 前端构建 |
| pnpm | 10.x | 包管理 |
| Rust | stable (≥ 1.75) | 后端编译 |
| Tauri CLI | 2.x | 桌面打包 |

### 环境搭建

```bash
# 1. 安装依赖
pnpm install

# 2. 启动开发模式（Vite + Tauri）
pnpm tauri dev

# 3. 仅前端（不启动 Tauri shell）
pnpm dev

# 4. Rust 编译检查
cd src-tauri && cargo check
```

---

## 2. 项目结构约定

详见 `02_architecture.md` 第 6 节（目录结构规划）。核心原则：

- **前端代码**在 `src/` 下，按 `lib/transport/`、`lib/stores/`、`components/`、`views/` 组织
- **后端代码**在 `src-tauri/src/` 下，按 `commands/`、`services/`、`repo/`、`ai/`、`models/` 分层
- **设计文档**在 `docs/` 下，编号排序
- **测试**：Rust 用 `#[cfg(test)]` 或 `tests/` 目录；前端用 `*.test.ts`

---

## 3. Git 工作流

### 分支策略

```
main ─────────────────────────────────────
  │
  ├── feat/channel-crud ──── PR ──── ←
  ├── feat/agent-management ─ PR ── ←
  ├── fix/order-key-collision ─ PR ─ ←
  └── ...
```

| 分支 | 用途 | 命名 |
|------|------|------|
| `main` | 稳定分支，永远可编译可测试 | — |
| `feat/*` | 功能开发 | `feat/channel-crud` |
| `fix/*` | Bug 修复 | `fix/order-key-collision` |
| `refactor/*` | 重构 | `refactor/split-message-service` |
| `docs/*` | 纯文档 | `docs/update-api-reference` |

### 分支规则

- 所有改动通过 PR 合入 `main`，不直接 push
- 分支从 `main` 最新 HEAD 创建
- 合并前 rebase 到最新 `main`
- 合并使用 squash merge（保持 main 历史清洁）

---

## 4. Commit 规范

格式：`<type>: <description>`

```
feat: add channel CRUD commands
fix: handle order_key collision with retry
refactor: extract generation engine from message service
test: add integration tests for channel repo
docs: update database schema to v0.3
chore: upgrade tauri to 2.10.3
```

| type | 用途 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `refactor` | 重构（不改变外部行为） |
| `test` | 测试 |
| `docs` | 文档 |
| `chore` | 构建/依赖/配置 |

### Commit 粒度

- 每个 commit 可编译、可测试
- 一个 feature 拆多个 commit：repo → service → command → 前端
- 不把多个不相关改动塞进一个 commit

---

## 5. Pull Request 流程

### 创建 PR

1. 确保本地测试通过（`cargo test` + `pnpm check`）
2. 创建 PR，填写模板：

```markdown
## 改了什么
- 简述改动内容

## 为什么改
- 关联的需求/issue

## 怎么测
- 测试步骤或自动化测试覆盖情况

## 影响范围
- 列出受影响的模块
```

### Review 标准

- [ ] 代码符合 `06_code_conventions.md` 规范
- [ ] 新功能有对应测试
- [ ] 无 `unwrap()` / `expect()` 在业务代码中
- [ ] API 变更同步更新了 OpenAPI 和 API 参考文档
- [ ] 数据库变更同步更新了 DDL 文档

### 合并

- 至少 1 人 approve
- CI 通过
- Squash merge 到 main

---

## 6. 代码风格

详见 `06_code_conventions.md`。关键指标：

| 指标 | 阈值 |
|------|------|
| 函数长度（service） | ≤ 50 行 |
| 圈复杂度 | ≤ 10 |
| 嵌套深度 | ≤ 4 层 |
| Svelte 组件 | ≤ 200 行 |
| Rust 模块文件 | ≤ 300 行 |

---

## 7. 文档维护

| 改动类型 | 需更新的文档 |
|----------|-------------|
| 数据库 schema 变更 | `03_database.md` |
| API 新增/修改 | `04_api_openapi.yaml` + `05_api_reference.md` |
| 架构/模块变更 | `02_architecture.md` |
| 新增/完成功能 | `10_progress.md` |
| 代码规范变更 | `06_code_conventions.md` |

文档与代码同步更新，不允许"先写代码后补文档"。

---

## 8. Issue 规范

### Bug Report

```markdown
**环境**: Windows 11, BuYu v0.1.0
**复现步骤**:
1. ...
2. ...
**期望行为**: ...
**实际行为**: ...
**截图/日志**: （如有）
```

### Feature Request

```markdown
**用户故事**: 作为...我希望...以便...
**验收标准**: ...
**优先级**: P0/P1/P2
```

---

## 9. 发布流程

1. 从 `main` 创建 release 分支（如 `release/0.1.0`）
2. 更新版本号：`package.json` + `Cargo.toml` + `tauri.conf.json`
3. 运行完整测试
4. `pnpm tauri build` 生成安装包
5. 打 tag：`git tag v0.1.0`
6. 合并回 main
