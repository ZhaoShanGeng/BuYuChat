# 步语 BuYu — 软件需求规格说明书（SRS）

**版本：** 0.2-draft  
**日期：** 2025  
**阶段：** MVP（P0）

---

## 主要功能清单

**P0（MVP）**
- 功能1：渠道管理（AI 服务商接入配置）
- 功能2：模型管理（渠道下的模型列表维护）
- 功能3：Agent 管理（含系统提示词）
- 功能4：会话管理（创建/删除/归档/置顶/重命名）
- 功能5：基本对话（流式回复 + 取消生成）
- 功能6：Reroll（同楼层多版本，版本切换）

**P1（MVP 后第一批）**
- 功能7：对话分支（Fork 到新会话）
- 功能8：MCP 工具调用
- 功能9：消息编辑（生成新版本）
- 功能10：多模型对比（同楼层横向并排 / Tab 切换）
- 功能11：多 Agent 协作（conversation_agents 多行，P1 才引入）

**P2（后续迭代）**
- 功能12：RAG 本地知识库（本地文件）
- 功能13：多平台支持（macOS / Linux）
- 功能14：后端上云与数据同步

**非功能性要求**
- 性能：流式首字符延迟 < 500ms（受限于 AI 服务商，客户端本身不引入额外延迟）
- 可靠性：崩溃后启动自动清理 generating 状态残留，已生成内容不丢失
- 数据安全：API Key 存本地 SQLite，不上传任何服务器
- 平台：优先 Windows 10/11 x64，架构跨平台（Tauri）
- 离线：配置管理功能完全离线可用；对话功能依赖网络

---

## 1. 用户角色与目标

| 角色 | 描述 | 核心目标 |
|------|------|----------|
| 普通用户 | 使用 AI 对话的任何人 | 快速开始对话，获得高质量回复 |
| 高级用户 | 有多个 AI 服务商账号 | 管理多渠道/模型，对比不同模型的回答质量 |

> 无注册/登录，本地单用户应用。

---

## 2. 数据库 Schema 说明（与功能相关的关键设计）

### MVP 阶段会话与 Agent 的绑定方式

MVP 只支持一个会话绑定一个 Agent + 渠道 + 模型。  
**直接在 `conversations` 表上增加字段**，不引入 `conversation_agents` 中间表，降低 MVP 复杂度：

```sql
conversations (
  id TEXT PK,
  title TEXT NOT NULL DEFAULT '新会话',
  agent_id TEXT → agents SET NULL,          -- 绑定的 Agent
  channel_id TEXT → api_channels SET NULL,  -- 绑定的渠道
  channel_model_id TEXT → api_channel_models SET NULL,  -- 绑定的模型
  archived INTEGER NOT NULL DEFAULT 0,
  pinned INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
)
```

**P1 阶段**引入多 Agent 协作时，再拆出 `conversation_agents` 中间表并做数据迁移。  
这样 MVP 的查询和写入都不需要 JOIN，逻辑最简单。

---

## 3. 功能清单（用户故事 + 验收标准）

---

### 功能1：渠道管理

**用户故事**
> 作为用户，我希望能添加、编辑、删除 AI 服务渠道，以便连接不同的 AI 服务商。

**验收标准（Gherkin）**

```gherkin
Feature: 渠道管理

  Scenario: 成功添加一个 OpenAI-compatible 渠道
    Given 用户打开渠道管理页面
    When 用户填写名称="My OpenAI"，base_url="https://api.openai.com"，api_key="sk-xxx"
    And 用户点击保存
    Then 渠道列表中出现"My OpenAI"
    And 该渠道的 enabled 状态为 true

  Scenario: base_url 格式验证
    Given 用户正在添加渠道
    When 用户填写 base_url="not-a-url"
    And 用户点击保存
    Then 显示错误提示"请输入有效的 URL"
    And 渠道未被保存

  Scenario: 编辑渠道名称
    Given 已存在渠道"My OpenAI"
    When 用户修改名称为"OpenAI Pro"并保存
    Then 渠道列表中显示"OpenAI Pro"

  Scenario: 删除渠道
    Given 已存在渠道"My OpenAI"
    When 用户点击删除并确认
    Then 渠道从列表中移除
    And 该渠道下所有模型同步删除（CASCADE）
    And 引用该渠道的 conversations.channel_id 置为 NULL
    And 受影响的会话下次发消息时提示"未配置渠道"

  Scenario: 禁用渠道
    Given 已存在渠道"My OpenAI"
    When 用户将渠道启用开关切换为关闭
    Then 该渠道 enabled = false
    And 使用该渠道的会话发消息时提示"渠道已禁用"
```

---

### 功能2：模型管理

**用户故事**
> 作为用户，我希望在渠道下管理模型列表，以便选择具体模型进行对话。

**验收标准（Gherkin）**

```gherkin
Feature: 模型管理

  Scenario: 手动添加模型
    Given 渠道"My OpenAI"已存在
    When 用户填写 model_id="gpt-4o"，display_name="GPT-4o" 并保存
    Then 模型出现在该渠道的模型列表中

  Scenario: 同一渠道不允许重复 model_id
    Given 渠道下已有模型 model_id="gpt-4o"
    When 用户再次添加 model_id="gpt-4o"
    Then 显示错误"该渠道下已存在此模型 ID"

  Scenario: 删除模型
    Given 模型"gpt-4o"存在
    When 用户删除该模型
    Then 模型从列表消失
    And 引用该模型的 conversations.channel_model_id 置为 NULL

  Scenario: 从渠道拉取模型列表（可选功能，MVP 可后做）
    Given 渠道配置了有效的 api_key 和 models_endpoint
    When 用户点击"从渠道拉取模型"
    Then 系统请求 models_endpoint
    And 返回的模型列表供用户勾选批量添加
```

---

### 功能3：Agent 管理

**用户故事**
> 作为用户，我希望创建、编辑、删除 Agent，每个 Agent 有名称和系统提示词，以便定制 AI 的行为。

**验收标准（Gherkin）**

```gherkin
Feature: Agent 管理

  Scenario: 创建 Agent
    Given 用户打开 Agent 管理页面
    When 用户输入名称="助手"，system_prompt 留空并保存
    Then Agent 列表出现"助手"
    And system_prompt 为空（无内置默认提示词）

  Scenario: 编辑 Agent 系统提示词
    Given Agent"助手"已存在
    When 用户修改 system_prompt="你是一个有帮助的助手"并保存
    Then Agent 的 system_prompt 更新成功
    And 该 Agent 绑定的所有会话，从下一条消息起使用新的 system_prompt
    And 已发送的历史消息不受影响（上下文在发送时构建，不回溯）

  Scenario: 删除已绑定会话的 Agent
    Given Agent"助手"已被会话"我的会话"引用（conversations.agent_id）
    When 用户删除该 Agent
    Then Agent 被删除
    And conversations.agent_id 置为 NULL（SET NULL）
    And 受影响的会话无法发新消息，提示"未配置 Agent"

  Scenario: 禁用 Agent
    When 用户禁用 Agent"助手"
    Then 绑定该 Agent 的会话发消息时提示"Agent 已禁用"

  Scenario: 首次打开应用无任何 Agent
    Given 应用首次启动，Agent 列表为空
    Then 显示引导提示"请先创建一个 Agent 以开始对话"
    And 不内置任何默认 Agent
```

---

### 功能4：会话管理

**用户故事**
> 作为用户，我希望创建、删除、归档、置顶、重命名会话，并为每个会话绑定 Agent 和模型。

**验收标准（Gherkin）**

```gherkin
Feature: 会话管理

  Scenario: 创建新会话
    Given 用户点击"新建会话"
    Then 创建标题为"新会话"的会话
    And agent_id / channel_id / channel_model_id 均为 NULL
    And 会话出现在列表顶部

  Scenario: 为会话绑定 Agent 和模型
    Given 会话"新会话"已创建
    When 用户在会话设置中选择 Agent="助手"，渠道="My OpenAI"，模型="gpt-4o"
    Then conversations 的 agent_id / channel_id / channel_model_id 更新成功

  Scenario: 重命名会话
    Given 会话"新会话"存在
    When 用户双击标题，输入"关于 Rust 的讨论"并确认
    Then 会话标题更新

  Scenario: 归档会话
    Given 会话存在
    When 用户选择归档
    Then conversations.archived = 1
    And 会话从活跃列表消失，在归档区域可查看

  Scenario: 删除会话
    Given 会话存在
    When 用户点击删除并确认
    Then 会话及其所有 message_nodes 和 message_versions 被级联删除

  Scenario: 置顶会话
    Given 存在多个会话
    When 用户置顶某会话
    Then conversations.pinned = 1
    And 该会话始终显示在活跃列表最顶部

  Scenario: 会话列表排序规则
    Given 存在普通会话和置顶会话
    Then 置顶会话排在最前（按 updated_at 降序）
    And 普通会话按 updated_at 降序排列（新创建或新有消息的在前）
    And 归档会话不出现在主列表
```

---

### 功能5：基本对话（流式回复 + 取消）

**用户故事**
> 作为用户，我希望在会话中发送消息并收到 AI 的流式回复，同时可以随时取消生成。

**验收标准（Gherkin）**

```gherkin
Feature: 基本对话

  Scenario: 发送消息并收到流式回复
    Given 会话已绑定 Agent、渠道和模型
    When 用户输入"你好"并点击发送
    Then 输入框内容立即清空（前端临时保留，用于失败恢复）
    And 用户消息楼层立即显示（status=committed）
    And AI 回复楼层立即出现（status=generating，内容为空）
    And AI 内容逐字流式填充
    And 生成完成后 status 变为 committed
    And 输入框临时保留的内容被清除

  Scenario: 发送失败时恢复输入内容
    Given 用户发送了消息
    When 后端返回错误（如未配置渠道）
    Then 输入框恢复为用户刚才输入的内容
    And 显示错误提示

  Scenario: 未配置 Agent 时发送消息
    Given 会话的 agent_id 为 NULL
    When 用户尝试发送消息
    Then 提示"请先为会话配置 Agent"，消息不发送
    And 输入框内容不清空

  Scenario: 未配置渠道时发送消息
    Given 会话已绑定 Agent，但 channel_id 为 NULL
    When 用户发送消息
    Then 后端返回 AiError::NoChannel，提示"请先配置渠道"

  Scenario: 取消正在生成的回复
    Given AI 正在流式生成某 version（status=generating）
    When 用户点击取消按钮
    Then 后台取消该 version_id 对应的 CancellationToken
    And 已生成的内容保留，version.status 变为 cancelled
    And 前端该楼层显示已生成的内容，并显示"已取消"状态标签
    And 输入框可重新输入

  Scenario: 生成过程中网络中断
    Given AI 正在流式生成
    When 网络连接中断
    Then 已生成内容保留，version.status 变为 failed
    And 前端显示已生成的内容，并显示"生成失败"状态标签

  Scenario: 应用崩溃后重启
    Given 上次崩溃时有 status=generating 的 message_versions
    When 应用重启
    Then 所有 generating 状态的 message_versions 自动改为 failed
    And 已生成的内容（content 字段）保留可读

  Scenario: 多会话并发生成
    Given 会话 A 正在生成回复（version_id=V1，conversation_id=C1）
    When 用户切换到会话 B 并发送消息（version_id=V2，conversation_id=C2）
    Then 两个生成任务同时独立运行
    And generation:chunk 事件通过 conversation_id + version_id 路由
    And C1 的事件只更新会话 A 的 UI，C2 的事件只更新会话 B 的 UI
```

---

### 功能6：Reroll（同楼层多版本）

**用户故事**
> 作为用户，我希望对回复执行 Reroll，在同一楼层生成新版本，并能通过版本切换器查看所有版本。

**版本切换器 UI 规范**

- 样式：`< [1] 2  3 >` — 数字可点击，当前 active 版本高亮显示
- 左右箭头仅作相邻切换，数字支持直接跳转（如从版本3直接点击1）
- 切换时立即写库（UPDATE message_nodes.active_version_id），不做 debounce
- 位置：楼层底部，仅在鼠标 hover 该楼层时显示

**验收标准（Gherkin）**

```gherkin
Feature: Reroll

  Scenario: 对 assistant 楼层执行 Reroll
    Given AI 回复楼层有 1 个版本（status=committed）
    When 用户点击 Reroll 按钮
    Then 在同一 node 下新建一个 message_version（status=generating，content=""）
    And active_version_id 切换到新版本
    And 新版本开始流式生成
    And 旧版本保留，版本切换器显示"< 1 [2] >"

  Scenario: 对 user 楼层执行 Reroll（仅限最后一个 user 楼层）
    Given 最后一个楼层是 user 消息（role=user，后无 assistant 楼层）
    Then 该楼层显示 Reroll 按钮
    When 用户点击 Reroll
    Then 在同一 user node 下新建一个 message_version（内容复制自当前 active version，status=committed）
    And active_version_id 切换到新版本
    And 随后创建新的 assistant node + message_version（status=generating）
    And 开始流式生成
    And user 楼层版本切换器显示新版本（内容相同，代表"这次请求的版本"）

  Scenario: 编辑 user 消息（P1，与 Reroll 共用 version 机制）
    Given 某 user 楼层存在
    When 用户点击编辑，修改内容后点击"保存并重新发送"
    Then 在该 user node 下新建一个 message_version（内容为修改后的文本，status=committed）
    And active_version_id 切换到新版本
    And 创建新的 assistant node + message_version（status=generating）开始生成
    And 旧 user version 保留，用户可通过版本切换器查看历史编辑内容

  Scenario: user 楼层 Reroll 按钮显示条件
    Given 某 user 楼层后面已有 assistant 楼层
    Then 该 user 楼层不显示 Reroll 按钮（只有末尾的 user 楼层才显示）

  Scenario: Reroll 的上下文构建规则
    Given 对话顺序：用户A → 助手B（3个版本，active=版本2）→ 用户C → 助手D
    When 用户对助手D执行 Reroll
    Then 上下文包含：用户A、助手B的版本2（active）、用户C
    And 不包含助手D本身（防止重复）
    And 助手B切换 active version 不影响已触发的 Reroll 请求

  Scenario: Reroll 期间切换版本不影响生成中的请求
    Given 楼层有版本1（committed）和版本2（generating）
    When 用户点击版本切换器切到版本1
    Then active_version_id 立即写库，更新为版本1
    And 版本2的生成任务在后台继续，不受影响
    And 生成完成后版本2内容正常写库
    And active_version_id 仍然是版本1（不自动跳回版本2）

  Scenario: Reroll 生成完成后 AI 返回空内容
    Given Reroll 触发了版本2生成
    When AI 返回空内容（content=""，finish_reason=stop）
    Then 后端执行空内容回滚：删除版本2，active_version_id 退回版本1
    And 后端发送 generation:empty_rollback 事件
    And 前端收到事件后更新 UI，显示提示"AI 返回了空回复，已自动撤销"

  Scenario: 唯一版本生成完成后 AI 返回空内容
    Given 某楼层只有版本1（generating）
    When AI 返回空内容
    Then 后端删除版本1，并删除整个 node
    And 后端发送 generation:empty_rollback 事件（node_id，无 fallback_version_id）
    And 前端收到事件后移除该楼层，显示提示"AI 返回了空回复"

  Scenario: 非末尾楼层执行 Reroll
    Given 对话顺序：用户A → 助手B → 用户C → 助手D
    When 用户对助手B执行 Reroll
    Then 助手B的新版本正常生成
    And 用户C、助手D楼层保持不变，不级联删除

  Scenario: 版本切换器直接跳转
    Given 某楼层有版本1、版本2、版本3，当前 active=版本3
    When 用户直接点击版本切换器上的"1"
    Then active_version_id 立即更新为版本1
    And 立即写库
    And 前端显示版本1内容，切换器显示"< [1] 2  3 >"

  Scenario: 删除正在 generating 的版本
    Given 某楼层版本2 status=generating
    When 用户点击删除版本2
    Then 自动取消版本2的生成请求（CancellationToken）
    And 版本2从数据库删除
    And active_version_id 切换到同 node 最新的其他版本
    And 若版本2是唯一版本，整个 node 被删除
```

---

## 4. 业务规则

### 4.1 状态机（message_versions.status）

```
generating  →  committed   （生成正常完成，且 content 不为空）
generating  →  failed      （网络/API 错误，或启动清理）
generating  →  cancelled   （用户主动取消）
committed   →  （终态）
failed      →  （终态）
cancelled   →  （终态）
```

- 用户消息的 version 创建时直接为 committed
- AI 消息：创建为 generating，完成后根据结果转换
- 生成完成但 content 为空时，不转为 committed，直接执行空内容回滚逻辑（见 4.5）

### 4.2 order_key 规则

- 格式：`{timestamp_ms:016}-{position_tag}-{random_suffix}`
- 同批创建的 user/assistant：user 用 `-0-`，assistant 用 `-1-`
- 字典序即消息顺序；UNIQUE(conversation_id, order_key) 冲突时应用层重试

### 4.3 上下文构建规则

- 取每个 node 的 active_version 的 content，按 order_key 字典序排列
- 排除当前正在生成的 assistant node（防止空内容入上下文）
- 若 agent.system_prompt 不为空，加入 messages[0] 作为 system 消息
- system_prompt 在每次发送时实时读取（不做快照），修改立即生效

### 4.4 渠道 endpoint 拼接规则

```
final_url = base_url.trimEnd('/') + endpoint
```

- base_url 只到域名（如 `https://api.openai.com`）
- endpoint 含完整路径（如 `/v1/chat/completions`）
- NULL 时使用 channel_type 默认值：

| channel_type | auth_type | models_endpoint | chat/stream_endpoint |
|---|---|---|---|
| openai_compatible | bearer | /v1/models | /v1/chat/completions |

### 4.5 空内容回滚规则

生成完成时（finish_reason 到达），若 content 为空字符串：
1. 若 node 下还有其他 version → 删除当前 version，active_version_id 切回最新的其他 version
2. 若当前 version 是该 node 唯一的 version → 删除 version + 删除 node
3. 前端弹出提示"AI 返回了空回复"

### 4.6 版本切换写库规则

- 用户点击版本切换器时，立即执行 `UPDATE message_nodes SET active_version_id = ? WHERE id = ?`
- 不做 debounce，不做延迟批量
- 后台正在生成的 version 不受影响，生成完成后仍正常写库，但不改变 active_version_id

### 4.7 Reroll 按钮显示规则

| 楼层类型 | 显示条件 |
|---|---|
| role=assistant | 始终显示 |
| role=user | 仅当该楼层是会话中最后一个楼层时显示 |

### 4.8 user 楼层 Reroll 与编辑的 version 语义

user node 同样支持多 version，version 由以下两种操作产生：

| 操作 | version 内容 | 触发后续行为 |
|---|---|---|
| Reroll（末尾 user 楼层） | 复制当前 active version 内容 | 创建新 assistant node 开始生成 |
| 编辑消息（P1） | 用户修改后的新内容 | 创建新 assistant node 开始生成 |

两种操作都复用同一套 version 机制，版本切换器对 user/assistant 楼层行为一致。

### 4.9 model_name 存储规则

- `message_versions.model_name` 存储 `model_id`（实际调用标识符）
- 不存 display_name，不存外键
- 渠道/模型被删除后，历史版本的 model_name 仍可读（有意为之）

### 4.10 级联规则

| 删除对象 | 级联行为 |
|---|---|
| api_channels | 级联删除 api_channel_models；conversations.channel_id → NULL |
| api_channel_models | conversations.channel_model_id → NULL |
| agents | conversations.agent_id → NULL |
| conversations | 级联删除 message_nodes → 级联删除 message_versions |
| message_nodes | 级联删除 message_versions |
| message_versions（最后一个） | 应用层检测后删除父 node |

---

## 5. 边界条件与异常处理场景

### 5.1 配置缺失

| 场景 | 处理 |
|---|---|
| conversations.agent_id 为 NULL | 前端拦截，提示配置 Agent，不发请求 |
| agent.enabled = false | 后端返回 AiError::AgentDisabled |
| conversations.channel_id 为 NULL | 后端返回 AiError::NoChannel |
| conversations.channel_model_id 为 NULL | 后端返回 AiError::NoModel |
| channel.enabled = false | 后端返回 AiError::ChannelDisabled |
| 首次启动，无任何 Agent | 前端显示引导，不内置默认 Agent |

### 5.2 并发场景

| 场景 | 处理 |
|---|---|
| 同一楼层多个版本同时 generating | 每个 version 独立 CancellationToken（DashMap key=version_id） |
| 跨会话并发生成 | 事件携带 conversation_id + node_id + version_id，前端按 conversation_id 路由 |
| 取消不存在的生成任务 | 忽略，返回 OK（幂等） |

### 5.3 输入框状态管理

| 时机 | 行为 |
|---|---|
| 用户点击发送 | 输入框立即清空；内容临时保留在前端状态 |
| 后端返回成功（node 创建完成） | 临时保留内容清除 |
| 后端返回失败 | 输入框恢复临时保留的内容；显示错误提示 |

### 5.4 AI 调用异常

| 场景 | version.status | 内容 |
|---|---|---|
| API 返回 4xx（如 401） | failed | 已收内容保留 |
| API 返回 5xx | failed | 已收内容保留 |
| 流中断（网络超时） | failed | 已收内容保留 |
| 用户取消 | cancelled | 已收内容保留，UI 显示"已取消" |
| 完成但 content="" | 执行空内容回滚 | version 被删除 |
| 应用崩溃重启 | failed（启动清理） | 已收内容保留 |

### 5.5 删除边界

| 场景 | 处理 |
|---|---|
| 删除 generating 的 version | 先取消 CancellationToken，再删除 version（及可能的 node） |
| 删除 active_version | active_version_id 切换到同 node 最新的其他版本；无其他版本则删 node |
| 删除会话中唯一的消息 node | node + version 一并删除，会话保留（变为空会话） |

---

## 6. 事件规范（Tauri Channel 推送）

所有 AI 生成相关事件通过 Tauri `Channel`（有序、有 back-pressure）推送，不使用全局广播事件。

| 事件名 | 数据字段 |
|---|---|
| generation:chunk | conversation_id, node_id, version_id, delta |
| generation:completed | conversation_id, node_id, version_id, prompt_tokens, completion_tokens, finish_reason, model |
| generation:failed | conversation_id, node_id, version_id, error |
| generation:cancelled | conversation_id, node_id, version_id |
| generation:empty_rollback | conversation_id, node_id, fallback_version_id（NULL 表示 node 已被删除） |

---

## 7. 待确认问题

1. 多模型对比（P1 功能10）：用户如何选择"用哪几个模型同时生成"？在发送时弹选择器，还是在会话设置中预配置？
2. RAG（P2）支持的文件类型范围（PDF / TXT / Markdown / Word？）
3. 错误信息的 i18n 策略：后端返回 error code，前端翻译？还是直接返回中文？
4. 是否需要本地日志文件用于问题排查？最大大小和路径？
