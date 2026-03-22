<script lang="ts">
  import {
    Bell,
    Cable,
    ChevronRight,
    Database,
    Monitor,
    Palette,
    Plug,
    Shield,
    Sun,
    Moon
  } from "lucide-svelte";
  import { i18n } from "$lib/i18n.svelte";
  import { theme } from "$lib/theme.svelte";
  import { cn } from "$lib/utils";
  import Button from "$components/ui/button.svelte";
  import HeaderWindowGroup from "$components/layout/header-window-group.svelte";
  import PageShell from "$components/layout/page-shell.svelte";
  import ApiChannelsPanel from "$components/settings/api-channels-panel.svelte";
  import { mockSettingsSections } from "$lib/fixtures/workspaces";

  type SectionId = "channels" | "appearance" | "general";

  let activeSection = $state<SectionId>("channels");

  const sections = [
    { id: "channels", label: i18n.t("settings.section.channels"), icon: Cable },
    { id: "appearance", label: i18n.t("settings.section.appearance"), icon: Palette },
    { id: "general", label: i18n.t("settings.section.general"), icon: Monitor }
  ] satisfies { id: SectionId; label: string; icon: typeof Cable }[];
</script>

<PageShell>
  {#snippet sidebar()}
    <nav class="space-y-0.5 px-2 py-3">
      {#each sections as section}
        <Button
          type="button"
          variant="ghost"
          size="md"
          className={cn(
            "w-full justify-start gap-2.5 px-3",
            section.id === activeSection
              ? "bg-[var(--bg-active)] font-medium text-[var(--ink-strong)] hover:bg-[var(--bg-active)]"
              : "text-[var(--ink-muted)]"
          )}
          onclick={() => {
            activeSection = section.id;
          }}
        >
          <section.icon size={16} />
          {section.label}
        </Button>
      {/each}
    </nav>
  {/snippet}

  {#snippet header()}
    <header class="flex h-12 items-center justify-between gap-3 border-b border-[var(--border-soft)] px-4" data-tauri-drag-region>
      <h1 class="text-sm font-semibold text-[var(--ink-strong)]">{sections.find((section) => section.id === activeSection)?.label ?? i18n.t("nav.settings")}</h1>
      <HeaderWindowGroup />
    </header>
  {/snippet}

  {#snippet mobileTabs()}
    <div class="flex gap-1 border-b border-[var(--border-soft)] px-3 py-2 lg:hidden">
      {#each sections as section}
        <Button
          type="button"
          variant={section.id === activeSection ? "default" : "ghost"}
          size="sm"
          className={cn("gap-1.5 rounded-[var(--radius-full)]", section.id !== activeSection && "text-[var(--ink-muted)]")}
          onclick={() => {
            activeSection = section.id;
          }}
        >
          <section.icon size={14} />
          {section.label}
        </Button>
      {/each}
    </div>
  {/snippet}

  {#snippet body()}
    <div class="app-scrollbar h-full overflow-y-auto p-6">
      <div class="max-w-6xl">
        {#if activeSection === "channels"}
          <ApiChannelsPanel />
        {:else if activeSection === "appearance"}
          <div class="space-y-6">
            <div>
              <h2 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("settings.appearance.theme_title")}</h2>
              <p class="mt-0.5 text-xs text-[var(--ink-muted)]">{i18n.t("settings.appearance.theme_desc")}</p>
              <div class="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-3">
                <Button type="button" variant="secondary" size="md" className={cn("h-auto justify-start border-2 px-4 py-3", theme.preference === "light" ? "border-[var(--brand)] bg-[var(--brand-soft)] hover:bg-[var(--brand-soft)]" : "border-[var(--border-soft)] hover:border-[var(--border-medium)]")} onclick={() => theme.set("light")}>
                  <Sun size={20} class={theme.preference === "light" ? "text-[var(--brand)]" : "text-[var(--ink-faint)]"} />
                  <div class="text-left">
                    <span class="text-sm font-medium text-[var(--ink-strong)]">{i18n.t("theme.light")}</span>
                  </div>
                </Button>
                <Button type="button" variant="secondary" size="md" className={cn("h-auto justify-start border-2 px-4 py-3", theme.preference === "dark" ? "border-[var(--brand)] bg-[var(--brand-soft)] hover:bg-[var(--brand-soft)]" : "border-[var(--border-soft)] hover:border-[var(--border-medium)]")} onclick={() => theme.set("dark")}>
                  <Moon size={20} class={theme.preference === "dark" ? "text-[var(--brand)]" : "text-[var(--ink-faint)]"} />
                  <div class="text-left">
                    <span class="text-sm font-medium text-[var(--ink-strong)]">{i18n.t("theme.dark")}</span>
                  </div>
                </Button>
                <Button type="button" variant="secondary" size="md" className={cn("h-auto justify-start border-2 px-4 py-3", theme.isSystem ? "border-[var(--brand)] bg-[var(--brand-soft)] hover:bg-[var(--brand-soft)]" : "border-[var(--border-soft)] hover:border-[var(--border-medium)]")} onclick={() => theme.set("system")}>
                  <Monitor size={20} class={theme.isSystem ? "text-[var(--brand)]" : "text-[var(--ink-faint)]"} />
                  <div class="text-left">
                    <span class="text-sm font-medium text-[var(--ink-strong)]">{i18n.t("theme.system")}</span>
                  </div>
                </Button>
              </div>
            </div>

            <hr class="border-[var(--border-soft)]" />

            <div>
              <h2 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("settings.appearance.language_title")}</h2>
              <p class="mt-0.5 text-xs text-[var(--ink-muted)]">{i18n.t("settings.appearance.language_desc")}</p>
              <div class="mt-3 grid grid-cols-2 gap-3">
                <Button type="button" variant="secondary" size="md" className={cn("h-auto justify-start border-2 px-4 py-3", i18n.locale === "zh-CN" ? "border-[var(--brand)] bg-[var(--brand-soft)] hover:bg-[var(--brand-soft)]" : "border-[var(--border-soft)] hover:border-[var(--border-medium)]")} onclick={() => i18n.setLocale("zh-CN")}>
                  <span class="text-lg">🇨🇳</span>
                  <span class="text-sm font-medium text-[var(--ink-strong)]">简体中文</span>
                </Button>
                <Button type="button" variant="secondary" size="md" className={cn("h-auto justify-start border-2 px-4 py-3", i18n.locale === "en" ? "border-[var(--brand)] bg-[var(--brand-soft)] hover:bg-[var(--brand-soft)]" : "border-[var(--border-soft)] hover:border-[var(--border-medium)]")} onclick={() => i18n.setLocale("en")}>
                  <span class="text-lg">🇺🇸</span>
                  <span class="text-sm font-medium text-[var(--ink-strong)]">English</span>
                </Button>
              </div>
            </div>
          </div>
        {:else}
          <div class="space-y-6">
            {#each [
              { ...mockSettingsSections[0], title: i18n.t("settings.general.database"), iconComponent: Database },
              { ...mockSettingsSections[1], title: i18n.t("settings.general.notifications"), iconComponent: Bell },
              { ...mockSettingsSections[2], title: i18n.t("settings.general.security"), iconComponent: Shield },
              { ...mockSettingsSections[3], title: i18n.t("settings.general.integrations"), iconComponent: Plug }
            ] as section}
              <div>
                <div class="flex items-center gap-2">
                  <section.iconComponent size={16} class="text-[var(--ink-faint)]" />
                  <div>
                    <h2 class="text-sm font-semibold text-[var(--ink-strong)]">{section.title}</h2>
                    <p class="text-xs text-[var(--ink-muted)]">{section.desc}</p>
                  </div>
                </div>
                <div class="mt-3 space-y-1">
                  {#each section.items as item}
                    <Button type="button" variant="secondary" size="md" className="h-auto w-full justify-between px-3 py-2.5 text-sm text-[var(--ink-body)]">
                      {item}
                      <ChevronRight size={14} class="text-[var(--ink-faint)]" />
                    </Button>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/snippet}
</PageShell>
