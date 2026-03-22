<script lang="ts">
  import { Loader2, Save } from "lucide-svelte";
  import { toast } from "svelte-sonner";
  import type { AgentSummary } from "$lib/api/agents";
  import {
    replaceConversationParticipants,
    updateConversationMeta,
    type ConversationDetail,
    type ConversationParticipantInput
  } from "$lib/api/conversations";
  import {
    buildConversationChatConfigFromParticipants,
    getAgentParticipants,
    mergeConversationChatConfig,
    resolvePrimaryResponderParticipantId
  } from "$lib/chat/conversation-preferences";
  import { i18n } from "$lib/i18n.svelte";
  import Button from "$components/ui/button.svelte";

  let {
    conversationDetail = null,
    availableAgents = []
  }: {
    conversationDetail?: ConversationDetail | null;
    availableAgents?: AgentSummary[];
  } = $props();

  let selectedAgentIds = $state<string[]>([]);
  let primaryAgentId = $state<string | null>(null);
  let loadedConversationId = $state<string | null>(null);
  let saving = $state(false);

  const availableAgentOptions = $derived(
    availableAgents.filter((agent) => agent.enabled)
  );

  $effect(() => {
    const detail = conversationDetail;
    const conversationId = detail?.summary.id ?? null;
    if (!conversationId) {
      selectedAgentIds = [];
      primaryAgentId = null;
      loadedConversationId = null;
      return;
    }

    const currentAgentParticipants = getAgentParticipants(detail);
    const nextSelectedAgentIds = currentAgentParticipants
      .map((participant) => participant.agent_id)
      .filter((value): value is string => !!value);
    const primaryParticipantId = resolvePrimaryResponderParticipantId(detail);
    const nextPrimaryAgentId =
      currentAgentParticipants.find((participant) => participant.id === primaryParticipantId)?.agent_id ??
      nextSelectedAgentIds[0] ??
      null;

    if (loadedConversationId !== conversationId) {
      selectedAgentIds = nextSelectedAgentIds;
      primaryAgentId = nextPrimaryAgentId;
      loadedConversationId = conversationId;
      return;
    }

    const validSelectedIds = selectedAgentIds.filter((agentId) =>
      nextSelectedAgentIds.includes(agentId)
    );
    if (validSelectedIds.length !== selectedAgentIds.length) {
      selectedAgentIds = validSelectedIds.length > 0 ? validSelectedIds : nextSelectedAgentIds;
    }

    if (!primaryAgentId || !selectedAgentIds.includes(primaryAgentId)) {
      primaryAgentId = nextPrimaryAgentId;
    }
  });

  function toggleAgent(agentId: string) {
    if (selectedAgentIds.includes(agentId)) {
      if (selectedAgentIds.length === 1) {
        return;
      }

      selectedAgentIds = selectedAgentIds.filter((item) => item !== agentId);
      if (primaryAgentId === agentId) {
        primaryAgentId = selectedAgentIds.find((item) => item !== agentId) ?? null;
      }
      return;
    }

    selectedAgentIds = [...selectedAgentIds, agentId];
    if (!primaryAgentId) {
      primaryAgentId = agentId;
    }
  }

  async function saveParticipants() {
    const detail = conversationDetail;
    if (!detail || saving) {
      return;
    }

    if (selectedAgentIds.length === 0) {
      toast.error(i18n.t("chat.no_agent_title"), {
        description: i18n.t("chat.select_responder_desc")
      });
      return;
    }

    saving = true;

    try {
      const humanParticipant = detail.participants.find(
        (participant) => participant.participant_type === "human"
      );

      const existingAgentParticipants = new Map(
        detail.participants
          .filter((participant) => participant.participant_type === "agent" && participant.agent_id)
          .map((participant) => [participant.agent_id as string, participant])
      );

      const orderedAgentIds = availableAgentOptions
        .filter((agent) => selectedAgentIds.includes(agent.id))
        .map((agent) => agent.id);

      const items: ConversationParticipantInput[] = [
        {
          agent_id: null,
          display_name: humanParticipant?.display_name ?? i18n.t("chat.user_label"),
          participant_type: "human",
          enabled: true,
          sort_order: 0,
          config_json: humanParticipant?.config_json ?? {}
        },
        ...orderedAgentIds.map((agentId, index) => {
          const existing = existingAgentParticipants.get(agentId);
          return {
            agent_id: agentId,
            display_name: existing?.display_name ?? null,
            participant_type: "agent",
            enabled: true,
            sort_order: index + 1,
            config_json: existing?.config_json ?? {}
          };
        })
      ];

      const participants = await replaceConversationParticipants(detail.summary.id, items);
      const chatConfig = buildConversationChatConfigFromParticipants(
        participants,
        primaryAgentId,
        orderedAgentIds
      );

      await updateConversationMeta(detail.summary.id, {
        title: detail.summary.title,
        description: detail.summary.description,
        archived: detail.summary.archived,
        pinned: detail.summary.pinned,
        config_json: mergeConversationChatConfig(detail.summary, chatConfig)
      });
    } catch (error) {
      console.error("Failed to save conversation bindings:", error);
      toast.error(i18n.t("chat.bindings_save_failed"), {
        description: error instanceof Error ? error.message : i18n.t("chat.generic_error")
      });
    } finally {
      saving = false;
    }
  }
</script>

{#if !conversationDetail}
  <div class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-medium)] bg-[var(--bg-app)] px-4 py-8 text-center text-xs text-[var(--ink-faint)]">
    {i18n.t("inspector.select_msg")}
  </div>
{:else}
  <div class="space-y-4">
    <div class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] p-4">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h4 class="text-sm font-semibold text-[var(--ink-strong)]">{i18n.t("chat.recipient_title")}</h4>
          <p class="mt-1 text-xs leading-relaxed text-[var(--ink-faint)]">{i18n.t("chat.recipient_desc")}</p>
        </div>
        <Button
          type="button"
          size="sm"
          className="gap-1.5"
          onclick={() => void saveParticipants()}
          disabled={saving || selectedAgentIds.length === 0}
        >
          {#if saving}
            <Loader2 size={14} class="animate-spin" />
          {:else}
            <Save size={14} />
          {/if}
          {i18n.t("chat.save")}
        </Button>
      </div>
    </div>

    <div class="grid gap-3">
      {#each availableAgentOptions as agent}
        {@const checked = selectedAgentIds.includes(agent.id)}
        <label class="flex cursor-pointer items-start gap-3 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--bg-app)] px-4 py-3 transition-colors hover:border-[var(--border-medium)]">
          <input
            type="checkbox"
            class="mt-1"
            checked={checked}
            onchange={() => toggleAgent(agent.id)}
          />
          <div class="min-w-0 flex-1">
            <div class="flex items-center justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-medium text-[var(--ink-strong)]">{agent.name}</p>
                {#if agent.title}
                  <p class="truncate text-xs text-[var(--ink-faint)]">{agent.title}</p>
                {/if}
              </div>
              <div class="flex items-center gap-2 text-xs text-[var(--ink-muted)]">
                <input
                  type="radio"
                  name="conversation-primary-agent"
                  checked={primaryAgentId === agent.id}
                  disabled={!checked}
                  onchange={() => {
                    if (checked) {
                      primaryAgentId = agent.id;
                    }
                  }}
                />
                {i18n.t("chat.primary_agent")}
              </div>
            </div>
          </div>
        </label>
      {/each}
    </div>
  </div>
{/if}
