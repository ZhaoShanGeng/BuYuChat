<script lang="ts">
  /**
   * 渠道管理页容器，负责装配数据加载、编辑状态与通知消息。
   */

  import ChannelFormPanel from "./ChannelFormPanel.svelte";
  import ChannelListPanel from "./ChannelListPanel.svelte";
  import {
    createChannel,
    deleteChannel,
    listChannels,
    testChannel,
    toAppError,
    updateChannel,
    type Channel,
    type ChannelInput
  } from "../lib/transport/channels";
  import {
    createEmptyForm,
    createFormFromChannel,
    humanizeError,
    removeChannel,
    submitChannelForm,
    verifyChannelConnectivity,
    type Notice
  } from "./channel-settings-state";

  let channels = $state<Channel[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let testingId = $state<string | null>(null);
  let editingId = $state<string | null>(null);
  let notice = $state<Notice | null>(null);
  let form = $state<ChannelInput>(createEmptyForm());
  let initialized = false;

  /**
   * 重置当前表单与编辑状态。
   */
  function resetForm() {
    editingId = null;
    form = createEmptyForm();
  }

  /**
   * 重新加载渠道列表。
   */
  async function reloadChannels() {
    loading = true;
    try {
      channels = await listChannels(true);
    } catch (error) {
      notice = { kind: "error", text: humanizeError(toAppError(error)) };
    } finally {
      loading = false;
    }
  }

  /**
   * 进入渠道编辑模式。
   */
  function startEdit(channel: Channel) {
    editingId = channel.id;
    form = createFormFromChannel(channel);
    notice = null;
  }

  /**
   * 提交渠道表单。
   */
  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    notice = null;

    try {
      notice = await submitChannelForm(
        {
          createChannel,
          updateChannel
        },
        editingId,
        form
      );
      resetForm();
      await reloadChannels();
    } finally {
      saving = false;
    }
  }

  /**
   * 删除指定渠道。
   */
  async function handleDelete(id: string) {
    notice = await removeChannel({ deleteChannel }, id);
    if (notice.kind === "success") {
      if (editingId === id) {
        resetForm();
      }
      await reloadChannels();
    }
  }

  /**
   * 测试指定渠道的连通性。
   */
  async function handleConnectivityTest(id: string) {
    testingId = id;
    notice = null;

    notice = await verifyChannelConnectivity({ testChannel }, id);
    testingId = null;
  }

  /**
   * 首次进入页面时加载渠道列表。
   */
  $effect(() => {
    if (initialized) {
      return;
    }

    initialized = true;
    void reloadChannels();
  });
</script>

<section class="grid gap-6 lg:grid-cols-[1.05fr_0.95fr]">
  <ChannelListPanel
    {channels}
    {loading}
    {notice}
    {testingId}
    onCreate={resetForm}
    onDelete={handleDelete}
    onEdit={startEdit}
    onTest={handleConnectivityTest}
  />

  <ChannelFormPanel
    {editingId}
    {form}
    {saving}
    onReset={resetForm}
    onSubmit={handleSubmit}
  />
</section>
