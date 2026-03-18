# 手动测试指南

这份文档只放当前已落地能力的标准化手动测试方式。

---

## 1. 原始 Provider 请求

用途：

- 验证网关是否可用
- 查看原始响应
- 快速排查模型、鉴权、网关映射问题

命令模板：

```powershell
cargo run --bin raw_request -- --base-url https://api.wataruu.me/v1 --api-key "你的key" --method GET --path /models
```

聊天请求：

```powershell
cargo run --bin raw_request -- --base-url https://api.wataruu.me/v1 --api-key "你的key" --method POST --path /chat/completions --header "Content-Type: application/json" --body '{"model":"gpt-5.1","messages":[{"role":"user","content":"Reply with exactly: ok"}],"stream":false}'
```

如果在 `Git Bash` 里运行，前面加：

```bash
MSYS_NO_PATHCONV=1
```

例如：

```bash
MSYS_NO_PATHCONV=1 cargo run --bin raw_request -- --base-url https://api.wataruu.me/v1 --api-key "你的key" --method GET --path /models
```

如果不想和命令行引号搏斗，可以把请求体写进文件：

`request.json`

```json
{"model":"gpt-5.1","messages":[{"role":"user","content":"Reply with exactly: ok"}],"stream":false}
```

然后执行：

```powershell
cargo run --bin raw_request -- --base-url https://api.wataruu.me/v1 --api-key "你的key" --method POST --path /chat/completions --header "Content-Type: application/json" --body-file request.json
```

---

## 2. 后端最小闭环测试

用途：

- 验证 Provider 配置
- 验证 API Key 存取
- 验证模型列表
- 验证基础对话

PowerShell：

```powershell
$env:OMNICHAT_TEST_BASE_URL='https://api.wataruu.me/v1'
$env:OMNICHAT_TEST_API_KEY='你的key'
$env:OMNICHAT_TEST_MODEL='gpt-5.1'
cargo test backend_provider_smoke_test -- --nocapture
```

通过标准：

- 能看到 `conversation_id=...`
- 能看到 `models_available=...`
- 能看到 `reply=...`

---

## 3. 聊天主链路测试

用途：

- 创建会话
- 发送消息
- 重新生成消息回复
- 切换回复版本
- 编辑消息后单独保存
- 编辑保存后再重新生成

命令：

```powershell
cargo run --bin chat_flow -- --base-url https://api.wataruu.me/v1 --api-key "你的key" --model gpt-5.1
```

可选参数：

```powershell
--prompt "Reply with exactly: ok"
--edit-prompt "Reply with exactly: edited"
```

通过标准：

- 能看到 `conversation_id=...`
- 能看到 `send_message ...`
- 能看到 `regenerate ...`
- 能看到 `assistant_versions=2` 或更大
- 能看到 `save_message_edit ...`
- 能看到 `regenerate_after_edit ...`
- 最后能输出 `final_active_messages=2`

---

## 4. 前端页面测试

用途：

- 在桌面界面里验证当前主链路
- 验证多渠道配置、真实模型加载、模型库维护、建会话、会话重命名/删除、系统消息、发消息、重新生成、编辑消息、删除单条消息、左右切换消息版本、分支为新会话

启动：

```powershell
pnpm tauri dev
```

如果要确认系统消息是否真的发给后端，建议带日志启动：

```powershell
$env:RUST_LOG="debug"
pnpm tauri dev
```

测试步骤：

1. 在左侧 `API 渠道` 面板填入 `渠道名称`、`渠道类型`、`Base URL`、`Models Path`、`Chat Path`、`Stream Path` 和 `API Key`
2. 点击 `创建渠道`
3. 点击 `拉取模型`
4. 确认模型下拉框里出现真实可用模型，例如 `gpt-5.1`
5. 在下方渠道列表中确认该渠道已保存，可再次点选回填
6. 点击 `新建`
7. 在右侧输入框输入 `Reply with exactly: ok`
8. 点击 `发送消息`
9. 确认消息列表里出现一条 user 和一条 assistant，assistant 内容为 `ok`
10. 点击 `重新生成`
11. 确认状态栏显示“已生成新版本回复”，assistant 消息出现版本切换按钮
12. 点击 assistant 消息上的左右按钮，确认可以切换不同版本
13. 在右侧 `系统消息` 输入框填写内容并点击 `保存系统消息`，确认保存成功
14. 发送一条简单消息，确认终端日志里 `sending chat request` 带有 `system_prompt_present=true`
15. 点击会话标题旁的 `重命名`，确认名称更新；再点击 `删除会话`，确认当前会话可以删除
16. 点击任意一条消息的 `编辑`，修改内容并点击 `保存编辑`，确认当前消息被替换为新版本，后续未修改消息仍然保留在当前会话
17. 点击某条 user 消息的 `重新生成`，确认该 user 的回复和后续当前可见子树被新回复取代
18. 点击某条 assistant 消息的 `重新生成`，确认只对该 assistant 消息生成新版本
19. 在非首条消息上切换旧版本，确认只切这条消息，不会自动切它上游的父链
20. 点击某条消息上的 `分支会话`，确认自动跳转到一个新会话，新会话保留该消息之前路径上的所有版本
21. 在新分支会话里继续编辑、发送或重新生成，确认不会影响原会话内容
22. 点击某条消息的 `删除消息`，确认只删除当前消息，其他消息仍然保留
23. 确认消息列表显示楼层、时间精确到秒；assistant 消息显示渠道、模型和 token
24. 调整窗口大小和位置后关闭再打开，确认窗口状态被持久化
25. 在 `模型库` 输入框手动添加一个模型，确认可保存
26. 点击 `拉取并保存`，确认模型列表会被真实接口刷新并持久化

通过标准：

- 渠道可以创建和切换
- 渠道切换后可回填 `渠道类型`、路径、`Base URL` 和 `API Key`
- 模型列表可以从真实接口正常拉取
- 模型库可以手动增删并持久化
- 会话可以创建
- 会话可以重命名和删除
- 会话级系统消息可以保存
- `sending chat request` 日志里能看到 `system_prompt_present=true`
- 消息可以发送
- assistant 回复能显示在界面中
- assistant 版本可以左右切换
- user 和 assistant 消息都可以编辑并保存，且不会导致当前会话后续消息丢失
- user 消息重新生成会替换当前可见回复链
- assistant 消息重新生成只作用于该条 assistant
- 非首条消息切旧版本时，只切当前消息，不自动切父链
- `分支会话` 会生成独立的新会话，后续修改不会回写原会话
- `删除消息` 只删除当前消息，不清空整段会话
- 消息显示楼层、精确到秒的时间，以及 assistant 渠道/模型/token
- 窗口大小和位置会被记住
- `重新生成` 不报错

---

## 当前说明

- `raw_request` 是命令行调试工具，不属于正式应用 IPC 接口
- 正式桌面功能仍然只通过 Tauri 命令暴露
- 后续每新增一块主线能力，都补到这份文档里再让你手测
