<script lang="ts">
  /**
   * 设置页主内容区 — 渠道编辑器 + 模型管理。
   * 侧边栏（渠道列表）由 WorkspaceShell 统一渲染。
   */
  import SettingsChannelEditor from "./SettingsChannelEditor.svelte";
  import SettingsModelManager from "./SettingsModelManager.svelte";
  import SettingsNoticeBanner from "./SettingsNoticeBanner.svelte";
  import SettingsUtilityPanel from "./SettingsUtilityPanel.svelte";
  import {
    AUTH_TYPE_OPTIONS,
    CHANNEL_TYPE_OPTIONS,
    type SettingsPageStateReturn
  } from "./settings-page-state.svelte.js";

  type Props = {
    settings: SettingsPageStateReturn;
  };

  const { settings }: Props = $props();
</script>

<section class="min-h-0 flex-1 overflow-y-auto" data-ui="settings-page-content">
  {#if settings.state.notice}
    <div class="px-6 pt-4 md:hidden">
      <SettingsNoticeBanner notice={settings.state.notice} />
    </div>
  {/if}

  <div class="settings-page__content-inner mx-auto flex flex-col gap-6 p-4 sm:p-6">
    <SettingsUtilityPanel
      busy={settings.state.utilitiesBusy}
      onExport={settings.handleExportSettings}
      onImport={settings.handleImportSettings}
      onOpenDataDir={settings.handleOpenDataDirectory}
      onOpenLogDir={settings.handleOpenLogDirectory}
    />

    <SettingsChannelEditor
      authTypeOptions={AUTH_TYPE_OPTIONS}
      channelTypeOptions={CHANNEL_TYPE_OPTIONS}
      form={settings.state.form}
      onDelete={settings.handleDeleteChannel}
      onReset={settings.resetCurrentDraft}
      onSave={(event) => {
        event.preventDefault();
        return settings.handleSaveChannel();
      }}
      onTest={settings.handleTestChannel}
      saving={settings.state.saving}
      selectedChannel={settings.selectedChannel}
      selectedChannelId={settings.state.selectedChannelId}
      testingId={settings.state.testingId}
    />

    <SettingsModelManager
      addingModel={settings.state.addingModel}
      groupedModels={settings.groupedModels}
      loadingModels={settings.state.loadingModels}
      loadingRemoteModels={settings.state.loadingRemoteModels}
      managingModels={settings.state.managingModels}
      models={settings.state.models}
      newModelDisplayName={settings.state.newModelDisplayName}
      newModelId={settings.state.newModelId}
      onCreateModel={settings.handleCreateModel}
      onDeleteModel={settings.handleDeleteModel}
      onFetchRemoteModels={settings.handleFetchRemoteModels}
      onImportRemoteModel={settings.handleImportRemoteModel}
      onNewModelDisplayNameChange={(value) => (settings.state.newModelDisplayName = value)}
      onNewModelIdChange={(value) => (settings.state.newModelId = value)}
      onToggleAdding={() => (settings.state.addingModel = !settings.state.addingModel)}
      onToggleManaging={() => (settings.state.managingModels = !settings.state.managingModels)}
      remoteModels={settings.state.remoteModels}
      selectedChannelId={settings.state.selectedChannelId}
    />
  </div>
</section>
