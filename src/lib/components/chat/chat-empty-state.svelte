<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { MessageCircle, Lightbulb, Sparkles } from "lucide-svelte";
  import type { AgentSummary } from "$lib/api/agents";
  import { i18n } from "$lib/i18n.svelte";
  import BuYuLogo from "$components/shared/buyu-logo.svelte";
  import Button from "$components/ui/button.svelte";

  let {
    conversationTitle = "Conversation",
    availableAgents = [],
    startingAgentId = "",
    onSelectSuggestion = (_text: string) => {},
    onStartWithAgent = (_agentId: string) => {}
  }: {
    conversationTitle?: string;
    availableAgents?: AgentSummary[];
    startingAgentId?: string;
    onSelectSuggestion?: (text: string) => void;
    onStartWithAgent?: (agentId: string) => void | Promise<void>;
  } = $props();

  const suggestions = $derived([
    { icon: MessageCircle, text: i18n.t("suggest.chat"), desc: i18n.t("suggest.chat_desc") },
    { icon: Lightbulb, text: i18n.t("suggest.brainstorm"), desc: i18n.t("suggest.brainstorm_desc") },
    { icon: Sparkles, text: i18n.t("suggest.write"), desc: i18n.t("suggest.write_desc") }
  ]);
</script>

<div class="chat-empty-state-shell" transition:fade={{ duration: 180 }}>
  <div class="chat-empty-state-hero" in:fly={{ y: 12, duration: 240 }}>
    <div class="relative">
      <BuYuLogo size={80} className="shadow-lg" />
      <div class="absolute -bottom-1 -right-1 flex h-7 w-7 items-center justify-center rounded-full border-2 border-[var(--bg-surface)] bg-[var(--success)] shadow-sm">
        <Sparkles size={14} class="text-white" />
      </div>
    </div>
    <div class="text-center">
      <h2 class="text-xl font-bold text-[var(--ink-strong)]">{conversationTitle}</h2>
      <p class="mt-1 text-sm text-[var(--ink-muted)]">{i18n.t("chat.start_hint")}</p>
    </div>
  </div>

  <div class="chat-empty-state-suggestions">
    {#each suggestions as suggestion, index}
      <div in:fly={{ y: 16, duration: 260, delay: 40 + index * 45 }}>
        <Button
          type="button"
          variant="ghost"
          size="md"
          className="suggestion-card h-auto w-full flex-col gap-2 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-5 text-center shadow-none hover:border-[var(--border-medium)] hover:bg-[var(--bg-elevated)]"
          onclick={() => onSelectSuggestion(suggestion.text)}
        >
          <div class="flex h-10 w-10 items-center justify-center rounded-[var(--radius-md)] bg-[var(--brand-soft)]">
            <suggestion.icon size={20} class="text-[var(--brand)]" />
          </div>
          <span class="text-sm font-medium text-[var(--ink-strong)]">{suggestion.text}</span>
          <span class="text-xs text-[var(--ink-faint)]">{suggestion.desc}</span>
        </Button>
      </div>
    {/each}
  </div>

  {#if availableAgents.length > 0}
    <div class="w-full max-w-[var(--empty-state-max-width)]" in:fly={{ y: 18, duration: 260, delay: 120 }}>
      <div class="mb-3 text-center">
        <p class="text-sm font-medium text-[var(--ink-strong)]">{i18n.t("chat.agent_section_title")}</p>
        <p class="mt-1 text-xs text-[var(--ink-faint)]">{i18n.t("chat.agent_section_desc")}</p>
      </div>
      <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {#each availableAgents.slice(0, 6) as agent}
          <Button
            type="button"
            variant="ghost"
            size="md"
            disabled={startingAgentId === agent.id}
            className="h-auto items-start justify-start rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--bg-surface)] px-4 py-4 text-left shadow-none hover:border-[var(--border-medium)] hover:bg-[var(--bg-elevated)] disabled:opacity-70"
            onclick={() => void onStartWithAgent(agent.id)}
          >
            <div class="flex w-full items-start gap-3">
              <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-[var(--brand-soft)] text-sm font-semibold text-[var(--brand)]">
                {agent.name.slice(0, 1).toUpperCase()}
              </div>
              <div class="min-w-0">
                <p class="truncate text-sm font-medium text-[var(--ink-strong)]">{agent.name}</p>
                {#if agent.title}
                  <p class="truncate text-xs text-[var(--ink-faint)]">{agent.title}</p>
                {/if}
              </div>
            </div>
          </Button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .chat-empty-state-shell {
    display: flex;
    min-height: 100%;
    flex-direction: column;
    align-items: center;
    gap: clamp(1.75rem, 4vh, 2.75rem);
    padding: clamp(2.5rem, 9vh, 5.5rem) 1.5rem 2rem;
  }

  .chat-empty-state-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    text-align: center;
  }

  .chat-empty-state-suggestions {
    display: grid;
    width: min(100%, var(--empty-state-max-width));
    gap: 0.75rem;
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  @media (max-width: 767px) {
    .chat-empty-state-shell {
      padding-top: 2.25rem;
      gap: 1.5rem;
    }

    .chat-empty-state-suggestions {
      grid-template-columns: 1fr;
    }
  }
</style>
