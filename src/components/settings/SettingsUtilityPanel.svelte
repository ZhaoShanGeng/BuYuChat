<script lang="ts">
  import Button from "../../lib/components/ui/button/button.svelte";

  type Props = {
    busy: boolean;
    onImport: () => void | Promise<void>;
    onExport: () => void | Promise<void>;
    onOpenDataDir: () => void | Promise<void>;
    onOpenLogDir: () => void | Promise<void>;
  };

  const { busy, onImport, onExport, onOpenDataDir, onOpenLogDir }: Props = $props();
</script>

<section class="rounded-3xl border bg-card p-6 shadow-sm" data-ui="settings-utility-panel">
  <div class="flex flex-col gap-1">
    <h3 class="text-sm font-semibold">工作台工具</h3>
    <p class="text-sm text-muted-foreground">
      导入导出配置备份，或直接打开应用数据与日志目录。
    </p>
  </div>

  <div class="mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
    <Button disabled={busy} onclick={() => void onImport()} variant="outline">导入配置</Button>
    <Button disabled={busy} onclick={() => void onExport()} variant="outline">导出配置</Button>
    <Button disabled={busy} onclick={() => void onOpenDataDir()} variant="outline">打开数据目录</Button>
    <Button disabled={busy} onclick={() => void onOpenLogDir()} variant="outline">打开日志目录</Button>
  </div>
</section>
