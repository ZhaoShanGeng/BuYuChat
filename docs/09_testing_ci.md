# 步语 BuYu — 测试与 CI 规范

**版本：** 0.3

所有 PR 必须满足这里的要求才能合并。

---

## 1. 测试策略总览

```
┌─────────────────────────────────┐
│     E2E Tests (P1)              │  ← Tauri 窗口级，MVP 不做
├─────────────────────────────────┤
│     Integration Tests           │  ← Rust: 真实 SQLite; 前端: mock invoke
├─────────────────────────────────┤
│     Unit Tests                  │  ← 纯逻辑测试，无 IO
└─────────────────────────────────┘
```

MVP 聚焦 **单元测试 + 集成测试**，E2E 测试 P1 再引入。

---

## 2. 测试分层与职责

### 2.1 Rust 后端

| 层级 | 测试类型 | 什么要测 | 怎么测 |
|------|---------|---------|--------|
| Repo | 集成测试 | SQL 正确性、约束、级联 | 内存 SQLite（`:memory:`） |
| Service | 单元测试 | 业务逻辑、状态流转、校验 | mock Repo（trait 抽象） |
| Command | 集成测试 | 命令编排、错误转换、真实副作用 | 真实 `AppState` + async command impl |
| AI Adapter | 单元测试 | 请求构建、响应解析、SSE 处理 | mock HTTP (wiremock) |
| Generation Engine | 集成测试 | 流水线完整流程、取消机制 | 内存 SQLite + mock AI |

### 2.2 TypeScript 前端

| 层级 | 测试类型 | 什么要测 | 怎么测 |
|------|---------|---------|--------|
| Transport | 单元测试 | invoke 封装、参数映射 | mock `@tauri-apps/api` |
| State Factory / `.svelte.ts` | 单元测试 | `$state` / `$derived` / `$effect` 状态流转与副作用 | vitest |
| Component | 组件测试 | 渲染、交互、事件处理 | @testing-library/svelte |
| Utils | 单元测试 | 纯函数（order-key 生成等） | vitest |

---

## 3. 覆盖率指标

| 层级 | 行覆盖率目标 | 分支覆盖率目标 |
|------|-------------|---------------|
| Rust Repo | ≥ 90% | ≥ 80% |
| Rust Service | ≥ 85% | ≥ 75% |
| Rust Command | ≥ 70% | — |
| TypeScript | ≥ 80% | ≥ 70% |

MVP 初期不强制阻断，但 PR 不应降低现有覆盖率。

---

## 4. 测试工具链

### Rust

| 工具 | 用途 |
|------|------|
| `cargo test` | 测试运行器 |
| `sqlx` | 内存 SQLite 用于 repo 测试 |
| `wiremock` | HTTP mock（AI 客户端测试） |
| `tokio::test` | 异步测试 |
| `cargo-llvm-cov` | 覆盖率报告（可选） |

### TypeScript

| 工具 | 用途 |
|------|------|
| `vitest` | 测试运行器 + 断言 |
| `@testing-library/svelte` | Svelte 组件测试 |
| `vi.mock()` | mock Tauri invoke |

---

## 5. 测试文件组织

### Rust

```
src-tauri/
├── src/
│   ├── services/
│   │   └── channel_service/     # 内含 #[cfg(test)] mod tests
│   ├── ai/
│   │   └── adapter.rs           # 内含 #[cfg(test)] mod tests
│   └── ...
└── tests/                       # 集成测试（跨模块）
    ├── repo_channels_test.rs
    ├── cmd_channels_test.rs
    ├── cmd_messages_test.rs
    └── helpers/
        └── mod.rs               # 测试 helpers（创建测试 DB 等）
```

### TypeScript

```
src/
├── lib/
│   ├── transport/
│   │   ├── channels.ts
│   │   └── channels.test.ts     # 与源文件同目录
│   │   ├── messages.ts
│   │   └── messages.test.ts
│   └── utils/
│       ├── order-key.ts
│       └── order-key.test.ts
└── components/
    ├── workspace-shell.svelte.ts
    ├── workspace-shell.svelte.test.ts
    ├── channel-settings.svelte.ts
    └── channel-settings-state.test.ts
```

---

## 6. 编写规范

### 6.1 测试命名

```rust
// Rust: test_{被测行为}_{场景}_{预期结果}
#[test]
fn test_create_channel_with_invalid_url_returns_validation_error() { ... }

#[test]
fn test_send_message_without_agent_returns_no_agent_error() { ... }
```

```typescript
// TypeScript: describe/it 风格
describe("createChannel", () => {
  it("returns validation error for invalid URL", async () => { ... });
  it("creates channel with default channel_type", async () => { ... });
});
```

### 6.2 测试结构

使用 **Arrange-Act-Assert** 模式：

```rust
#[tokio::test]
async fn test_create_channel_success() {
    // Arrange
    let pool = setup_test_db().await;
    let input = CreateChannelInput { name: "Test".into(), base_url: "https://api.openai.com".into(), .. };

    // Act
    let result = channel_service::create(&pool, input).await;

    // Assert
    assert!(result.is_ok());
    let channel = result.unwrap();
    assert_eq!(channel.name, "Test");
    assert!(channel.enabled);
}
```

### 6.3 测试隔离

- 每个 Rust 集成测试创建独立的内存 SQLite
- 不共享测试状态
- 不依赖测试执行顺序
- 前端测试在每个 describe 前 reset mock
- Windows 环境下若 `tauri::test` 的 mock webview 运行时不稳定，可退化为“真实 `AppState` + async command impl”模式，并在文档中注明

### 6.4 什么必须测

| 场景 | 必须测 |
|------|--------|
| CRUD 的成功路径 | ✅ |
| 错误码（每个 error_code） | ✅ |
| 级联删除效果 | ✅ |
| 状态机流转 | ✅ |
| 空内容回滚 | ✅ |
| 并发取消 | ✅ |
| 边界值（空字符串、超长内容） | ✅ |

---

## 7. CI 流水线

### GitHub Actions（计划）

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  rust:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cd src-tauri && cargo check
      - run: cd src-tauri && cargo test
      - run: cd src-tauri && cargo clippy -- -D warnings

  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
      - run: pnpm install
      - run: pnpm check
      - run: pnpm test
```

### 合并门禁

- [ ] `cargo test` 全部通过
- [ ] `cargo clippy` 无 warning
- [ ] `pnpm check` 无类型错误
- [ ] `pnpm test` 全部通过
- [ ] 覆盖率不低于现有水平

---

## 8. 本地开发怎么跑测试

```bash
# Rust 全部测试（当前 Windows 开发环境建议串行编译，避免 pagefile / mmap 问题）
cd src-tauri && CARGO_BUILD_JOBS=1 cargo test -j 1

# Rust 单个测试
cargo test test_create_channel

# Rust 特定模块
cargo test --lib services::channel_service

# Rust clippy 检查
CARGO_BUILD_JOBS=1 cargo clippy -- -D warnings

# 桌面壳运行（当前二进制挂在 feature 下）
cargo run --features desktop-shell

# 前端全部测试
pnpm test

# 前端单个测试
pnpm test -- -t "createChannel"

# 前端类型检查
pnpm check
```

### 8.1 当前高频回归命令

最近流式生成、前端状态层和 transport 映射改动较多，本地联调时建议优先补跑这些高频回归：

```bash
# Rust：消息流式与空消息回滚
cd src-tauri && cargo test --test cmd_messages_test
cd src-tauri && cargo test --test repo_messages_test
cd src-tauri && cargo clippy -- -D warnings

# Frontend：transport + 工作台状态层
pnpm check
pnpm test
```

说明：

1. `cmd_messages_test` 已覆盖“流式 delta 正常到达，但 provider 终态文本为空”时不应误触发 `empty_rollback` 的场景。
2. `pnpm test` 当前覆盖 transport 序列化、Channel 事件映射以及工作台状态层的流式更新回归。

---

## 9. 常见问题

### Q: Repo 测试用内存 SQLite，和生产 SQLite 有差异吗？

A: 功能上一致。唯一注意点是 WAL 模式在内存 DB 中不生效（内存 DB 无文件），但不影响查询逻辑测试。

### Q: 怎么测试 Tauri command？

A: 当前 MVP 基线是“真实 `AppState` + async command impl”的命令集成测试。若平台上的 `tauri::test` mock webview 稳定，可再补一层 IPC 级测试。

### Q: AI 客户端怎么测，不想真的调 API？

A: 使用 `wiremock` 启动本地 mock server，返回预设的 SSE 流。aisdk adapter 层对外暴露 trait，测试时可直接 mock。

### Q: 为什么现在 `cargo test` 建议加 `CARGO_BUILD_JOBS=1`？

A: 当前 Windows 开发环境下，并行编译 Tauri / WebView 相关依赖时更容易触发 pagefile / `mmap` 资源问题。后端测试本身已经和桌面壳解耦，但本地验证仍建议使用串行编译，确保结果稳定可复现。

### Q: 为什么运行桌面壳要显式带 `desktop-shell` feature？

A: 当前仓库把桌面壳二进制挂到了 `desktop-shell` feature 下，目的是让后端测试和 `cargo clippy` 不被桌面壳编译链拖住。后端基线验证只关心库代码和命令层，桌面壳单独按需编译。
