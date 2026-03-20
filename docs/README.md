# BuYu 文档索引

当前只保留两套有效文档：

## 1. 数据库结构设计

- [11-schema-design/00-索引.md](./11-schema-design/00-索引.md)
- 作用：
  - 作为数据库表设计的唯一入口
  - 每张表单独一个文件
  - 先定结构，再落 migration

## 2. 后端实现设计

- [12-backend-design/README.md](./12-backend-design/README.md)
- 作用：
  - 作为后端实现契约的唯一入口
  - 明确目录、类型、仓储、服务、Provider、命令、实现顺序
  - 直接供 AI 按步骤实现

## 3. 当前后端阅读文档

- [13-backend-current/README.md](./13-backend-current/README.md)
- 作用：
  - 作为当前后端代码阅读入口
  - 说明当前真实模块分布、关键类型、功能调用链、命令与测试
  - 用于快速读懂已经实现的后端

## 4. 前端设计文档

- [14-frontend-design/README.md](./14-frontend-design/README.md)
- 作用：
  - 作为前端重构与开发入口
  - 明确布局、视觉、状态流、组件拆分、文件结构、实施顺序
  - 直接指导 `Svelte 5 + Tauri 2 + Tailwind` 前端实现

## 当前规则

- 旧架构草稿、旧模块文档、旧实现指南已移除
- 现在不要再参考任何已删除的旧文档
- 数据库相关只看 `11-schema-design`
- 后端实现相关只看 `12-backend-design`
- 读当前代码现状时看 `13-backend-current`
- 前端重构相关只看 `14-frontend-design`
