/**
 * 渠道设置页面的 Svelte 5 runes 状态工厂。
 *
 * 这个文件负责承接 `ChannelSettings.svelte` 中会驱动界面的状态：
 * 1. 首屏加载、保存中、测试中等页面级状态。
 * 2. 表单编辑态与通知态。
 * 3. 与 transport 的异步交互和刷新时机。
 */

import {
  createChannel,
  deleteChannel,
  listChannels,
  testChannel,
  toAppError,
  updateChannel,
  type Channel,
  type ChannelInput,
  type ChannelTestResult
} from "../../lib/transport/channels";
import {
  createEmptyForm,
  createFormFromChannel,
  humanizeError,
  removeChannel,
  submitChannelForm,
  verifyChannelConnectivity,
  type Notice
} from "./channel-settings-state";

/**
 * 渠道设置页依赖的 transport 能力集合。
 */
export type ChannelSettingsDeps = {
  createChannel: (input: ChannelInput) => Promise<Channel>;
  updateChannel: (id: string, input: ChannelInput) => Promise<Channel>;
  deleteChannel: (id: string) => Promise<void>;
  listChannels: (includeDisabled?: boolean) => Promise<Channel[]>;
  testChannel: (id: string) => Promise<ChannelTestResult>;
  onChanged?: () => void | Promise<void>;
};

/**
 * 渠道设置页使用的默认依赖。
 */
const defaultDeps: ChannelSettingsDeps = {
  createChannel,
  updateChannel,
  deleteChannel,
  listChannels,
  testChannel
};

/**
 * 创建渠道设置页的响应式状态和行为。
 */
export function createChannelSettingsState(overrides: Partial<ChannelSettingsDeps> = {}) {
  const deps = {
    ...defaultDeps,
    ...overrides
  };

  const state = $state({
    channels: [] as Channel[],
    loading: true,
    saving: false,
    testingId: null as string | null,
    editingId: null as string | null,
    notice: null as Notice | null,
    form: createEmptyForm()
  });

  let initialized = false;

  /**
   * 重置当前表单与编辑状态。
   */
  function resetForm() {
    state.editingId = null;
    state.form = createEmptyForm();
  }

  /**
   * 重新加载渠道列表，并把变更通知给父级工作台。
   */
  async function reloadChannels() {
    state.loading = true;
    try {
      state.channels = await deps.listChannels(true);
      await deps.onChanged?.();
    } catch (error) {
      state.notice = { kind: "error", text: humanizeError(toAppError(error)) };
    } finally {
      state.loading = false;
    }
  }

  /**
   * 进入渠道编辑模式。
   */
  function startEdit(channel: Channel) {
    state.editingId = channel.id;
    state.form = createFormFromChannel(channel);
    state.notice = null;
  }

  /**
   * 提交渠道表单。
   */
  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    state.saving = true;
    state.notice = null;

    try {
      state.notice = await submitChannelForm(
        {
          createChannel: deps.createChannel,
          updateChannel: deps.updateChannel
        },
        state.editingId,
        state.form
      );
      resetForm();
      await reloadChannels();
    } finally {
      state.saving = false;
    }
  }

  /**
   * 删除指定渠道，并在必要时清理当前编辑态。
   */
  async function handleDelete(id: string) {
    state.notice = await removeChannel({ deleteChannel: deps.deleteChannel }, id);
    if (state.notice.kind === "success") {
      if (state.editingId === id) {
        resetForm();
      }
      await reloadChannels();
    }
  }

  /**
   * 测试指定渠道的连通性。
   */
  async function handleConnectivityTest(id: string) {
    state.testingId = id;
    state.notice = null;
    state.notice = await verifyChannelConnectivity({ testChannel: deps.testChannel }, id);
    state.testingId = null;
  }

  const destroy = $effect.root(() => {
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
  });

  return {
    state,
    destroy,
    reloadChannels,
    resetForm,
    startEdit,
    handleSubmit,
    handleDelete,
    handleConnectivityTest
  };
}
