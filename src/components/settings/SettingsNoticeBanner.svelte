<script lang="ts">
  import type { Notice } from "./settings-page.types";

  type Props = {
    notice: Notice;
  };

  const props: Props = $props();

  let toneClass = $derived.by(() => {
    if (props.notice.kind === "success") {
      return "bg-emerald-500/10 text-emerald-700";
    }

    if (props.notice.kind === "info") {
      return "bg-blue-500/10 text-blue-700";
    }

    return "bg-destructive/10 text-destructive";
  });
</script>

<div
  class={`settings-page__notice rounded-xl px-3 py-2 text-sm ${toneClass}`}
  data-kind={props.notice.kind}
  data-ui="settings-notice-banner"
>
  <div>{props.notice.text}</div>
  {#if props.notice.detail}
    <div class="mt-1 whitespace-pre-wrap break-all text-xs opacity-80">{props.notice.detail}</div>
  {/if}
</div>
