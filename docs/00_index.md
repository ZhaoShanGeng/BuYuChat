# 步语 BuYu — 文档体系

**版本：** 0.4
**最后更新：** 2026-04-08

目标：让仓库里的设计、实现、协作和发布信息只有一套事实来源。

## 当前文档分层

| 编号 | 文件 | 作用 | 状态 |
|------|------|------|------|
| 00 | `00_index.md` | 文档入口、分层说明、维护规则 | 当前 |
| 01 | `01_frontend_design.md` | 前端结构、UI 和交互设计 | 当前 |
| 02 | `02_architecture.md` | 运行时架构、目录结构、工程治理 | 当前 |
| 03 | `03_database.md` | SQLite schema、迁移与约束 | 当前 |
| 04 | `04_api_openapi.yaml` | 接口契约 | 当前 |
| 05 | `05_api_reference.md` | API 使用说明与错误码 | 当前 |
| 06 | `06_code_conventions.md` | 代码、测试、版本与仓库规范 | 当前 |
| 07 | `07_design_review.md` | 设计风险、问题与后续建议 | 当前 |
| 08 | `08_collaboration.md` | 分支、提交、PR、发布协作流程 | 当前 |
| 09 | `09_testing_ci.md` | 自动化测试、CI、打包与发布流程 | 当前 |
| 10 | `10_progress.md` | 项目进度与基础设施状态 | 当前 |
| 11 | `11_backend_mvp_baseline.md` | 后端基线快照 | 当前 |
| 12 | `12_software_specification.md` | 当前软件规格与功能边界 | 当前 |
| 13 | `13_api_design.md` | API 设计原则、资源划分与接口约束 | 当前 |

## 文档维护规则

| 变更类型 | 必须同步更新的文档 |
|----------|--------------------|
| 架构、目录、工程脚本调整 | `02_architecture.md` |
| 协作、分支、发布规则调整 | `08_collaboration.md` |
| 测试门禁、CI、打包流程调整 | `09_testing_ci.md` |
| 版本号、里程碑、基础设施状态变化 | `10_progress.md` |
| 产品能力与边界变化 | `12_software_specification.md` |
| API 设计方式或资源划分变化 | `13_api_design.md` |
| API 或数据库契约变化 | `03_database.md`、`04_api_openapi.yaml`、`05_api_reference.md` |

## 版本管理约定

- 文档与代码同提交，不接受“代码先落地、文档以后补”。
- 以仓库当前实现为准。
- 发布说明不另建 `CHANGELOG.md`，以 GitHub Release Notes 和 Git 历史为准。
