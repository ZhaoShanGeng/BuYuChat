# 步语 BuYu — 代码规范

**版本：** 0.3
**适用范围：** Rust 后端 + TypeScript/Svelte 前端

---

## 1. 命名规则

### 1.1 Rust

| 元素 | 风格 | 示例 |
|------|------|------|
| 模块/文件名 | snake_case | `channel_service.rs` |
| 结构体/枚举/Trait | PascalCase | `ChannelModel`, `AppError` |
| 枚举变体 | PascalCase | `AppError::NoChannel` |
| 函数/方法 | snake_case | `list_channels()` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_RETRY_COUNT` |
| 类型别名 | PascalCase | `type Result<T> = std::result::Result<T, AppError>` |
| Tauri command | snake_case（与函数名一致） | `#[tauri::command] fn create_channel()` |
| 数据库列名 | snake_case | `channel_model_id` |

### 1.2 TypeScript / Svelte

| 元素 | 风格 | 示例 |
|------|------|------|
| 文件名（组件） | PascalCase | `MessageBubble.svelte` |
| 文件名（非组件） | kebab-case 或 camelCase | `order-key.ts`, `conversations.svelte.ts` |
| 接口/类型 | PascalCase | `Channel`, `SendMessageInput` |
| 函数/变量 | camelCase | `handleGreet()`, `activeTab` |
| 常量 | SCREAMING_SNAKE_CASE 或 camelCase | `MAX_VERSIONS`（全局）, `defaultTitle`（模块内） |
| Svelte rune | `$` 前缀 | `$state`, `$derived`, `$effect` |
| 事件处理 | `on` + 动词 或 `handle` + 动词 | `onclick={handleSend}` |
| Store 文件 | `*.svelte.ts` | `conversations.svelte.ts` |

### 1.3 JSON 序列化

- Rust → 前端：使用 `#[serde(rename_all = "camelCase")]`
- Rust 内部 / 数据库 / OpenAPI 文档：snake_case
- 前端 TypeScript 类型定义：camelCase

---

## 2. 函数与复杂度限制

### 2.1 函数长度

| 层级 | 最大行数 | 超出时处理 |
|------|---------|-----------|
| Tauri command handler | 30 行 | 提取到 service 层 |
| Service 函数 | 50 行 | 拆分为私有辅助函数 |
| Repo 函数（SQL 查询） | 40 行 | SQL 超长时考虑拆分查询 |
| TypeScript 函数 | 40 行 | 提取辅助函数 |
| Svelte 组件 `<script>` | 80 行 | 提取逻辑到 `.svelte.ts` 或 `$lib/` |

### 2.2 圈复杂度 (Cyclomatic Complexity)

| 语言 | 阈值 | 工具 |
|------|------|------|
| Rust | ≤ 10 | `cargo clippy`（`cognitive-complexity` lint） |
| TypeScript | ≤ 10 | ESLint `complexity` rule |

超过阈值时必须重构：提取函数、使用 early return、用 match/map 替代嵌套 if-else。

### 2.3 嵌套深度

最大嵌套深度 **4 层**。超过时使用 early return 或提取函数。

```rust
// ❌ 不要这样
fn process() {
    if a {
        if b {
            if c {
                if d {  // 4 层，已到极限
                }
            }
        }
    }
}

// ✅ 使用 early return
fn process() {
    if !a { return; }
    if !b { return; }
    if !c { return; }
    handle_d();
}
```

---

## 3. Rust 编码规范

### 3.1 错误处理

- 使用统一的 `AppError` 结构化错误，不在业务代码中使用 `unwrap()` / `expect()`
- Tauri command handler 返回 `Result<T, AppError>`
- `AppError` 通过 serde 序列化为前端统一错误格式 `error_code + message`
- 数据库错误使用 `?` 传播，在 `AppError::Database` 中包装

### 3.2 数据库访问

- 使用 `sqlx + SqlitePool`，迁移统一放在 `src-tauri/migrations/`
- 优先使用 `sqlx::query!` / `sqlx::query_as!`；若当前未启用离线元数据，可临时使用 `query` / `query_as`，并由 repo 集成测试兜底
- 写操作必须在事务中（`sqlx::Transaction`）
- 读操作直接使用连接池（`&SqlitePool`）
- 禁止在循环中执行 SQL（N+1 问题）

### 3.3 异步

- 所有 Tauri command 使用 `async fn`
- 长时间运行的任务（AI 生成）使用 `tokio::spawn`
- 使用 `tokio_util::sync::CancellationToken` 支持取消

### 3.4 模块组织

- 每个模块文件 ≤ 300 行（不含测试）
- 超过时按职责拆分为子模块
- `mod.rs` 只做 re-export，不含业务逻辑

### 3.5 依赖注入

- Tauri command 通过 `State<AppState>` 注入共享状态
- Service 层接收 `&SqlitePool` 和其他依赖作为参数（不使用全局变量）
- 便于单元测试 mock

### 3.6 注释规范

- 每个 Rust 源码文件顶部必须有中文 `//!` 文件说明
- 每个 Rust 函数/方法都必须有中文 `///` 或紧邻的块注释，测试 helper 也不例外
- 每个 TypeScript / Svelte 源码文件顶部必须有中文 `/** ... */` 文件说明
- 每个 TypeScript / Svelte 函数、事件处理器、helper、导出类型都必须写中文 JSDoc
- 注释应说明职责、约束、边界和非显然原因，不写低价值的逐行翻译

---

## 4. TypeScript / Svelte 编码规范

### 4.1 类型安全

- 启用 `strict: true`
- 禁止 `any`（使用 `unknown` 后做类型收窄）
- API 响应类型与 OpenAPI schema 一一对应
- 使用 discriminated union 处理 GenerationEvent

```typescript
type GenerationEvent =
  | { type: "chunk"; delta: string; /* ... */ }
  | { type: "completed"; promptTokens: number; /* ... */ }
  | { type: "failed"; error: string; /* ... */ }
  | { type: "cancelled"; /* ... */ }
  | { type: "empty_rollback"; nodeDeleted: boolean; /* ... */ };
```

### 4.2 Svelte 5 Runes

- 使用 `$state` 替代 `let`（响应式变量）
- 使用 `$derived` 替代 `$:` 响应式声明
- 使用 `$effect` 替代 `afterUpdate` / `onMount`（副作用）
- 不使用 Svelte 4 的 store（`writable` / `readable`）

#### 4.2.1 Svelte 5 使用边界

- `$state` 只用于**会驱动 UI 的可变状态**，例如加载中、当前选中项、表单草稿、消息列表缓存
- `const` 继续用于常量、依赖注入对象、纯函数引用
- 普通 `let` 只允许用于**非响应式哨兵或局部临时变量**，例如 `initialized`、上一次草稿值、循环内中间值
- 纯函数 helper 保持在普通 `.ts`；只有持有 runes 状态的模块才使用 `*.svelte.ts`
- 当某个 `.svelte` 文件接近 200 行，或 `<script>` 接近 80 行时，必须把页面级状态和异步逻辑下沉到 `*.svelte.ts`

```typescript
/**
 * 页面级状态应收敛到 `.svelte.ts`
 */
export function createWorkspaceState() {
  let initialized = false; // 非响应式哨兵，允许使用普通 let
  const state = $state({
    loading: true,
    query: "",
    items: [] as string[]
  });
  const itemCount = $derived(state.items.length);

  $effect(() => {
    if (initialized) {
      return;
    }

    initialized = true;
    void loadInitialData();
  });

  async function loadInitialData() {
    state.loading = true;
    state.items = ["a", "b"];
    state.loading = false;
  }

  return {
    state,
    get itemCount() {
      return itemCount;
    }
  };
}
```

### 4.3 组件设计

- 每个 `.svelte` 文件 ≤ 200 行（含模板）
- 逻辑超长时提取到 `.svelte.ts` 文件
- Props 使用 `$props()` 声明
- 事件使用 callback props（不使用 `createEventDispatcher`）

### 4.4 Transport 层

- 所有 `invoke()` 调用集中在 `src/lib/transport/` 下
- 组件/store 不直接调用 `invoke()`
- Transport 函数负责：调用 invoke、类型转换、Channel 创建与事件绑定

---

## 5. 测试规范（TDD）

### 5.1 Rust 测试

| 层级 | 测试方式 | 命名 |
|------|---------|------|
| Repo | 集成测试（内存 SQLite） | `tests/repo_*.rs` |
| Service | 单元测试（mock repo） | `#[cfg(test)] mod tests` |
| Command | 集成测试（真实 AppState + async command impl） | `tests/cmd_*.rs` |
| AI Client | 单元测试（mock HTTP） | `#[cfg(test)] mod tests` |

**运行：**
```bash
# 全部测试
cd src-tauri && cargo test

# 单个测试
cargo test test_create_channel

# 特定模块
cargo test --lib services::channel_service
```

### 5.2 前端测试

| 层级 | 测试方式 | 工具 |
|------|---------|------|
| Transport | 单元测试（mock invoke） | vitest |
| Store | 单元测试 | vitest |
| Component | 组件测试（可选，Svelte 5 栈稳定时开启） | vitest + @testing-library/svelte |

**运行：**
```bash
pnpm test          # 全部
pnpm test -- -t "test name"  # 单个
```

### 5.3 TDD 流程

1. 写失败的测试（Red）
2. 写最少代码让测试通过（Green）
3. 重构（Refactor）
4. 对每个 API endpoint，先写集成测试覆盖正常路径和错误路径

---

## 6. Git 规范

### 6.1 分支

- `main`：稳定分支，所有 PR 合入
- `feat/*`：功能分支
- `fix/*`：修复分支

### 6.2 Commit Message

格式：`<type>: <description>`

| type | 用途 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `refactor` | 重构（不改变行为） |
| `test` | 测试 |
| `docs` | 文档 |
| `chore` | 构建/依赖/配置 |

示例：`feat: add channel CRUD commands`

### 6.3 Commit 粒度

- 每个 commit 是一个可编译、可测试的原子变更
- 一个 feature 可拆成多个 commit：先 repo 层 → service 层 → command 层 → 前端

---

## 7. 工程脚本与版本规范

### 7.1 必须使用的脚本

| 命令 | 用途 |
|------|------|
| `pnpm ci:frontend` | 前端类型检查、测试和生产构建 |
| `pnpm ci:rust` | Rust 测试和 `clippy` |
| `pnpm verify` | 本地完整门禁，与 Release 工作流对齐 |
| `pnpm version:check` | 检查 `package.json`、`Cargo.toml`、`tauri.conf.json` 版本一致 |
| `pnpm version:set -- <version>` | 一次性更新三处版本号 |

### 7.2 版本控制规则

- 发布版本只认 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 三处。
- 发布 tag 统一使用 `v<semver>`，例如 `v0.2.0`。
- tag 与三处 manifest 版本不一致时，CI / Release 必须失败。

### 7.3 不应提交到仓库的内容

- 本地数据库：`src-tauri/buyu.db*`
- 本地调试目录：`target-codex-*`、`src-tauri/tmp/`
- 生成型配置产物：`vite.config.js`、`vite.config.d.ts`
