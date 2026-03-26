<script lang="ts">
  import WindowControls from "./WindowControls.svelte";

  type Props = {
    onHeaderMouseDown: (event: MouseEvent) => void | Promise<void>;
  };

  const props: Props = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="settings-page__header flex h-14 items-center justify-between border-b px-6"
  data-ui="settings-page-header"
  onmousedown={props.onHeaderMouseDown}
>
  <div>
    <h2 class="text-sm font-semibold">模型服务</h2>
    <p class="text-xs text-muted-foreground">统一管理渠道、模型和远程模型候选。</p>
  </div>

  <WindowControls compact />
</div>
