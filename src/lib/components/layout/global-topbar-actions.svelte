<script lang="ts">
  import { Languages, Monitor, Moon, Settings2, Sun } from "lucide-svelte";
  import SelectMenuButton from "$components/shared/select-menu-button.svelte";
  import ActionIconButton from "$components/shared/action-icon-button.svelte";
  import { i18n, type Locale } from "$lib/i18n.svelte";
  import { theme, type ThemePreference } from "$lib/theme.svelte";

  let {
    workspaceLabel = "BuYu",
    onOpenSettings = () => {}
  }: {
    workspaceLabel?: string;
    onOpenSettings?: () => void;
  } = $props();

  const localeOptions = $derived([
    {
      id: "zh-CN",
      label: "简体中文",
      active: i18n.locale === "zh-CN",
      onSelect: () => i18n.setLocale("zh-CN" satisfies Locale)
    },
    {
      id: "en",
      label: "English",
      active: i18n.locale === "en",
      onSelect: () => i18n.setLocale("en" satisfies Locale)
    }
  ]);

  const themeOptions = $derived([
    {
      id: "light",
      label: i18n.t("theme.light"),
      active: theme.preference === "light",
      onSelect: () => theme.set("light" satisfies ThemePreference)
    },
    {
      id: "dark",
      label: i18n.t("theme.dark"),
      active: theme.preference === "dark",
      onSelect: () => theme.set("dark" satisfies ThemePreference)
    },
    {
      id: "system",
      label: i18n.t("theme.system"),
      active: theme.preference === "system",
      onSelect: () => theme.set("system" satisfies ThemePreference)
    }
  ]);
</script>

<div class="flex min-w-0 flex-1 items-center justify-between gap-3">
  <div class="flex min-w-0 items-center gap-3">
    <div class="hidden items-center gap-2 rounded-[var(--radius-full)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-3 py-1 sm:flex">
      <span class="h-2 w-2 rounded-full bg-[var(--brand)]"></span>
      <span class="truncate text-xs font-medium text-[var(--ink-muted)]">{workspaceLabel}</span>
    </div>
  </div>

  <div class="flex items-center gap-1" data-no-drag>
    <ActionIconButton title={i18n.t("nav.settings")} className="hidden sm:inline-flex" onClick={onOpenSettings}>
      <Settings2 size={16} />
    </ActionIconButton>

    <SelectMenuButton
      title={i18n.locale === "zh-CN" ? "Language" : "语言"}
      options={localeOptions}
      placement="bottom"
      className="hidden sm:inline-flex"
    >
      {#snippet children()}
        <Languages size={16} />
      {/snippet}
    </SelectMenuButton>

    <SelectMenuButton
      title={theme.preference === "system" ? i18n.t("theme.system") : theme.isDark ? i18n.t("theme.dark") : i18n.t("theme.light")}
      options={themeOptions}
      placement="bottom"
    >
      {#snippet children()}
        {#if theme.preference === "system"}
          <Monitor size={16} />
        {:else if theme.isDark}
          <Sun size={16} />
        {:else}
          <Moon size={16} />
        {/if}
      {/snippet}
    </SelectMenuButton>
  </div>
</div>
