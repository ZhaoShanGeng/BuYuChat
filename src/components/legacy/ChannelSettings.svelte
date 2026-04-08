<script lang="ts">
  /**
   * 渠道管理 — 列表 + 详情表单 + 内嵌模型管理。
   * 在三栏布局中，渠道列表渲染在 ContextPanel（由 WorkspaceShell 传入 slot），
   * 但 ChannelSettings 自身也可以独立渲染（兼容旧调用方式）。
   */
  import ChannelFormPanel from "./ChannelFormPanel.svelte";
  import ChannelListPanel from "./ChannelListPanel.svelte";
  import { createChannelSettingsState } from "./channel-settings.svelte.js";

  type Props = {
    onChanged?: () => void | Promise<void>;
  };

  const { onChanged = () => undefined }: Props = $props();
  const cs = createChannelSettingsState({ onChanged: () => onChanged() });
</script>

<div class="flex h-full min-h-0">
  <!-- 渠道列表由外部 WorkspaceShell 负责 header，这里渲染列表内容 -->
  <div class="flex w-full min-h-0 flex-col">
    <!-- 分割布局：如果有选中的渠道，显示表单 -->
    <div class="flex h-full min-h-0">
      <!-- 左侧列表（内嵌在设置的 ContextPanel 区域不够用，这里自己画列表） -->
      <div class="w-64 shrink-0 border-r">
        <ChannelListPanel
          channels={cs.state.channels}
          editingId={cs.state.editingId}
          loading={cs.state.loading}
          notice={cs.state.notice}
          testingId={cs.state.testingId}
          onCreate={cs.resetForm}
          onDelete={cs.handleDelete}
          onEdit={cs.startEdit}
          onTest={cs.handleConnectivityTest}
        />
      </div>

      <!-- 右侧表单 -->
      <div class="min-h-0 flex-1 overflow-y-auto">
        <ChannelFormPanel
          editingId={cs.state.editingId}
          form={cs.state.form}
          saving={cs.state.saving}
          onReset={cs.resetForm}
          onSubmit={cs.handleSubmit}
        />
      </div>
    </div>
  </div>
</div>
