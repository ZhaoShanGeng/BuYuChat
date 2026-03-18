# 消息版本系统

> 路径：`src-tauri/src/services/versioning.rs` 和 `src-tauri/src/db/message.rs`

毕业设计版本只做**线性版本**，不做分支对话。

---

## 数据模型

```
对话历史（线性展示，每次只显示 is_active=true 的消息）：

msg-A (user,  version_group_id="vg-A", version_index=1, is_active=true)
  └─ "帮我写个快排"

msg-B (assistant, version_group_id="vg-B", version_index=1, is_active=false)
msg-B (assistant, version_group_id="vg-B", version_index=2, is_active=false)
msg-B (assistant, version_group_id="vg-B", version_index=3, is_active=true)
     └─ 三次生成，当前展示第 3 次

前端展示：< 1/3 ▶ 导航条（只有 assistant 消息有多版本时展示）
```

---

## 版本服务接口

```rust
// src-tauri/src/services/versioning.rs
pub struct VersioningService {
    db: SqlitePool,
}

impl VersioningService {
    /// 切换到指定版本
    /// 1. 验证 version_index 在 [1, max_version_index] 范围内
    /// 2. 调用 db::message::set_active_version(version_group_id, version_index)
    /// 3. 返回目标版本的 MessageRow（供前端更新展示）
    pub async fn switch_version(
        &self,
        version_group_id: &str,
        target_index: i64,
    ) -> Result<MessageRow>;
}
```

---

## 核心操作详解

### 操作 1：重新生成（Regenerate）

**触发**：用户点击 assistant 消息的「重新生成」按钮

**流程（由 ChatService.regenerate 执行）：**

```
1. 查找对话中最后一条 is_active=true AND role='assistant' 的消息
   → 获取其 version_group_id、created_at

2. BEGIN TRANSACTION
   UPDATE messages SET is_active=0 WHERE version_group_id=?
   （不删除旧版本，仍可切换回来）

3. 查 MAX(version_index) WHERE version_group_id=?
   → new_index = max + 1

4. 组装历史（排除最后一条 assistant 消息，从该 user 消息截止）
   → 历史包含 [user, assistant, ..., user_最新]，不含被重新生成的那条 assistant

5. 发起新一轮生成（与 send_message 步骤 7-11 相同）
   → 生成完成后 INSERT 新 MessageRow：
     version_group_id = 原来的 vg-B
     version_index    = new_index
     is_active        = true

6. COMMIT
```

---

### 操作 2：切换版本（Switch Version）

**触发**：用户点击 `◀` 或 `▶`

**流程：**
```
1. 前端先调用 `get_message_versions(version_group_id)` 得到全部版本
2. 用户点击 `◀` 或 `▶` 后，传入 `(version_group_id, target_index)`
3. 后端执行 `switch_version`
4. 前端收到新的 `MessageRow` 后替换当前展示内容
```

> 注意：切换版本后再发送新消息，上下文使用当前活跃版本（即切换后的版本）组装历史。

---

### 操作 3：编辑用户消息（Edit User Message）

**触发**：用户点击自己的消息「编辑」，修改内容后提交

**流程（由 ChatService 内部处理）：**

```
1. 获取被编辑的 user 消息的 created_at（记为 cutoff）

2. BEGIN TRANSACTION
   DELETE FROM messages
   WHERE conversation_id=? AND created_at >= cutoff
   （删除被编辑消息及其后所有消息，包括它本身的旧版本）

3. INSERT 新 user 消息：
   - 新的 id、version_group_id（全新 UUID）
   - version_index = 1, is_active = true
   - content = 用户修改后的内容
   - created_at = now()

4. COMMIT

5. 走 send_message 的步骤 7-12，基于新内容发起生成
```

> 设计决策：编辑用户消息后不保留后续消息历史，因为上下文已变，保留会造成语义混乱。

---

## 前端契约

| 前端操作 | Tauri 命令 | 传参 |
|---------|-----------|------|
| 重新生成 | `regenerate_message` | `conv_id` |
| 切换版本 | `switch_message_version` | `version_group_id`, `target_index` |
| 编辑用户消息 | `edit_user_message` | `conv_id`, `message_id`, `new_content` |
| 获取版本列表 | `get_message_versions` | `version_group_id` |
