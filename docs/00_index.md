# 步语 BuYu — 文档体系

**版本：** 0.3

目标：任何新来的开发者，靠读文档就能理解系统、找到接口、上手开发。

---

## 文档目录

| 编号 | 文件 | 内容 | 受众 |
|------|------|------|------|
| 00 | `00_index.md` | 本文件：文档体系总览与编写规范 | 所有人 |
| SRS | `BuYu_SRS_v0.2.md` | 需求规格说明书：用户故事、验收标准、业务规则 | 产品/开发 |
| 01 | `01_frontend_design.md` | 前端设计：页面布局、组件树、主题、交互、错误展示、i18n 错误码表 | 前后端 |
| 02 | `02_architecture.md` | 系统架构：分层、生成流水线、目录结构、技术选型、日志策略 | 开发 |
| 03 | `03_database.md` | 数据库设计：DDL、索引、外键、核心查询、迁移策略 | 后端 |
| 04 | `04_api_openapi.yaml` | OpenAPI 3.0 规范：所有接口的结构化定义 | 前后端 |
| 05 | `05_api_reference.md` | API 参考：通用约定、错误码、调用示例 | 前后端 |
| 06 | `06_code_conventions.md` | 代码规范：命名、复杂度、测试、Git | 开发 |
| 07 | `07_design_review.md` | 设计评审：缺陷、瓶颈、改进建议 | 架构/开发 |
| 08 | `08_collaboration.md` | 协作指南：分支策略、PR 流程、Issue 规范 | 所有人 |
| 09 | `09_testing_ci.md` | 测试与 CI：策略、覆盖率、流水线 | 开发 |
| 10 | `10_progress.md` | 进度追踪：功能完成状态 | 所有人 |
| 11 | `11_backend_mvp_baseline.md` | 后端 MVP 基线：当前已落地能力、数据库基线、测试基线、已知边界 | 开发/产品 |

---

## 文档编写规范

### 格式

- 使用 Markdown
- 每个文档开头标注**版本号**和**阶段**
- 使用表格和代码块提升可读性
- 中文为主（代码/标识符保持英文）

### 版本管理

- 文档随代码一起 commit
- 文档标注版本号（如 v0.3），重大变更时递增
- 变更记录写在文档末尾或专门的"变更记录"节

### 什么时候更新

| 触发事件 | 需更新的文档 |
|----------|-------------|
| 新增/修改数据库表 | 03_database.md |
| 新增/修改 API 接口 | 04_api_openapi.yaml + 05_api_reference.md |
| 架构调整 | 02_architecture.md |
| 完成功能模块 | 10_progress.md |
| 完成一个可复用的阶段性基线 | 11_backend_mvp_baseline.md |
| 发现设计问题 | 07_design_review.md |
| 工具链/流程变更 | 08_collaboration.md / 09_testing_ci.md |

### 代码内文档

- **Rust**：每个源码文件顶部必须有 `//!` 文件说明；所有函数/方法（含私有 helper、测试 helper）必须写中文 `///` 或紧邻块注释；公开类型/Trait/enum 必须写 doc comment
- **TypeScript / Svelte**：每个源码文件顶部必须有 `/** ... */` 文件说明；导出函数、局部 handler、helper、导出类型都必须写中文 JSDoc
- 注释用于解释职责、约束、边界和非显然逻辑；禁止写“把值赋给变量”这类低信息量注释

### 不需要的文档

- 不写单独的 CHANGELOG（Git log 即 changelog）
- 不写 ADR（设计决策记录在 07_design_review.md 中）
- 不写独立的部署文档（`pnpm tauri build` 即全部）
