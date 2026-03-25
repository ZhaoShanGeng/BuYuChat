# 步语 BuYu

AI 对话桌面客户端。Tauri v2 + Svelte 5 + Rust。

## 快速开始

```bash
# 安装依赖
pnpm install

# 启动开发模式（Vite + Tauri 窗口）
pnpm tauri dev

# 仅前端（不启动 Tauri，localhost:1420）
pnpm dev

# 类型检查
pnpm check

# Rust 检查
cd src-tauri && cargo check

# 生产构建
pnpm tauri build
```

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Svelte 5 + TypeScript + Tailwind CSS 4 + bits-ui |
| 桌面壳 | Tauri v2 |
| 后端 | Rust + SQLite (sqlx) + aisdk |
| 包管理 | pnpm 10 |

## 文档

所有设计文档在 [`docs/`](docs/00_index.md) 目录下：

- [文档体系](docs/00_index.md) — 目录总览
- [前端设计](docs/01_frontend_design.md) — UI 布局、组件、主题
- [架构设计](docs/02_architecture.md) — 分层、生成流水线
- [数据库设计](docs/03_database.md) — DDL、索引、迁移
- [OpenAPI 规范](docs/04_api_openapi.yaml) — 接口定义
- [API 参考](docs/05_api_reference.md) — 调用示例、错误码
- [代码规范](docs/06_code_conventions.md) — 命名、复杂度
- [设计评审](docs/07_design_review.md) — 已知问题与改进
- [协作指南](docs/08_collaboration.md) — Git 工作流、PR 流程
- [测试与 CI](docs/09_testing_ci.md) — 测试策略、覆盖率
- [进度追踪](docs/10_progress.md) — MVP 功能状态

## 当前状态

最小骨架已搭建（一个页面 + 一个 Rust greet 命令），设计文档完成，准备进入实现阶段。
