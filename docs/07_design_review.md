# 步语 BuYu — 设计评审

**版本：** 0.3
**评审范围：** SRS v0.2 + 数据库设计 v0.3 + API 设计 v0.3 + 架构设计 v0.2

---

## 1. 设计缺陷与改进建议

### D1: content 存储策略 ✅ 已解决

**问题：** `message_versions.content` 无大小限制，大量版本累积后 SQLite 文件膨胀。

**解决方案：** 内容与版本元数据分离。

1. **`message_versions` 表不再存 content**，只存元数据（status、model_name、tokens 等）
2. **新增 `message_contents` 表**，按 `version_id` 分块存储内容：
   - 每个 chunk ≤ 64KB，顺序编号
   - 流式生成时逐 chunk 追加（INSERT），不 UPDATE 已有行
   - 为未来多模态内容（图片/文件引用）预留 `content_type` 字段
3. 应用层限制单条消息总内容 ≤ 512KB
4. P1 可引入大文件外部存储，DB 只存引用路径

---

### D2: 版本数量无上限

**问题：** 单个 node 下 reroll 可无限产生新版本。

**风险等级：** 中

**解决方案：**
- 应用层设置单 node 版本上限 50，超过后最旧的非 active 版本自动删除
- MVP 可不实现硬限制，但代码中预留版本清理接口
- **配合 P1 优化**：`list_messages` 非 active 版本只返回元数据（不含 content），版本膨胀对传输量影响降至最低

---

### D3: order_key 冲突重试 ✅ 采纳

重试上限 3 次，超出返回 `INTERNAL_ERROR`。实际冲突概率极低（UUID 随机后缀）。

---

### D4: 外键引用校验 ✅ 采纳

Service 层显式校验 `agent_id` / `channel_id` / `channel_model_id` 存在且 enabled，不依赖 DEFERRABLE 外键做业务校验。

---

### D5: dry_run 暴露 system_prompt ✅ 预期行为

`dry_run` 是调试接口，暴露完整 prompt（含 system_prompt）是设计目标。无需额外处理。

---

## 2. 遗漏与模糊点

### M1: chunk 写库策略 ✅ 采纳

明确规范：
- 计数单位：UTF-8 字节数，阈值 2048 bytes
- 实现方式：`tokio::select!` 同时监听 chunk 到达 + 2 秒 interval
- 写库失败：log error，不中断生成，下次 flush 包含全部累积内容
- **配合 D1**：chunk 直接 INSERT 到 `message_contents`，不 UPDATE

---

### M2: 前端重启后的状态恢复 ✅ 采纳

- 前端启动时调用 `list_messages` 获取最新状态
- `status=failed` 的版本直接展示终态
- `generatingVersions` Set 启动时为空，仅从新的 send/reroll 响应中填充

---

### M3: channel_type 扩展机制 ✅ 采纳

- 定义 `ChannelTypeConfig` trait/struct
- MVP 用简单 match，代码结构便于扩展
- **补充**：AI 请求层使用 `aisdk` + `aisdk-macros` 库，该库已内置多 provider 支持（OpenAI-compatible），无需从头实现 HTTP 客户端

---

### M4: 并发生成限制 ✅ 采纳

`tokio::Semaphore` 限制最大并发生成数 ≤ 5，超出排队不拒绝。

---

### M5: 归档会话搜索范围 ✅ 采纳

P1 搜索设计时需考虑 `archived` 过滤选项。

---

### M6: api_key 更新生效时机 ✅ 采纳

已明确：生成开始时一次性读取，正在进行中的请求不受影响。属于预期行为。

---

### M7: AI 请求库选型（新增）

**决定：** 使用 `aisdk` + `aisdk-macros` crate，不自建 HTTP 客户端。

**理由：**
- 已内置 OpenAI-compatible API 的请求/响应类型
- 支持 SSE 流式解析
- 宏简化 provider 配置
- 减少重复造轮子

**影响：**
- 架构文档中的 `ai/client.rs` 改为 aisdk 的 adapter 层
- `Cargo.toml` 新增 `aisdk`、`aisdk-macros` 依赖

---

## 3. 性能瓶颈

### P1: 消息列表传输优化 ✅ 已解决

**方案：分层加载**

1. `list_messages` 返回所有 node + 所有 version **元数据**，但 **只返回 active version 的 content**
2. 非 active version 的 content 通过单独接口按需加载：`GET /versions/{versionId}/content`
3. P1 实现游标分页（`before_order_key` + `limit`）
4. 前端虚拟滚动（只渲染可见区域）

**效果：** 即使一个 node 有 50 个版本，传输量仅为 1 个 content + 50 条元数据（每条 ~200 bytes）

---

### P2: conversations.updated_at 写入频率

**状态：** ✅ 已解决（仅终态时更新）

---

### P3: 版本切换器写库频率

**结论：** 保持"立即写库"，SQLite 单条 UPDATE ~0.1ms，无需优化。

---

## 4. 安全隐患

> S1-S4 已评估，MVP 阶段风险可接受，暂不处理。详见 v0.2 评审存档。

| 编号 | 问题 | 风险 | 处理 |
|------|------|------|------|
| S1 | API Key 明文存储 | 中 | P1 迁移 Keychain |
| S2 | base_url SSRF | 低 | P2 上云时处理 |
| S3 | 无请求频率限制 | 低 | Semaphore + UI 防抖 |
| S4 | Channel 无鉴权 | 无 | 非问题（进程内通信） |

---

## 5. 改进建议汇总

| 编号 | 优先级 | 建议 | 阶段 | 状态 |
|------|--------|------|------|------|
| D1 | 高 | content 与 version 分离，分块存储 | MVP | ✅ 采纳 |
| D2 | 低 | 版本数上限 50 | P1 | ✅ 采纳 |
| D3 | 低 | order_key 冲突重试上限 3 次 | MVP | ✅ 采纳 |
| D4 | 高 | service 层显式校验外键引用 | MVP | ✅ 采纳 |
| D5 | — | dry_run 暴露 system_prompt | — | ✅ 预期行为 |
| M1 | 中 | chunk 刷盘策略：2048B / 2s / INSERT | MVP | ✅ 采纳 |
| M4 | 中 | Semaphore 限制并发生成 ≤ 5 | MVP | ✅ 采纳 |
| M7 | 高 | 使用 aisdk + aisdk-macros | MVP | ✅ 采纳 |
| P1 | 高 | list_messages 仅返回 active content | MVP | ✅ 采纳 |

---

## 6. 与上一版评审对比（v0.2 → v0.3 变更记录）

| 变更项 | 处理结果 |
|--------|---------|
| D1 content 限制 100KB | **升级为**分块存储方案（`message_contents` 表） |
| D5 system_prompt 暴露 | **确认为**预期行为，不再标记为缺陷 |
| P1 消息列表全量加载 | **解决**：非 active version 只返回元数据 |
| AI 客户端自建 | **改为**使用 `aisdk` + `aisdk-macros` 库 |
| S1-S4 安全项 | **统一延后**，MVP 不处理 |
