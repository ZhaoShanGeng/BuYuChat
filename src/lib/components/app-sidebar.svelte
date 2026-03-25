<script lang="ts">
	/**
	 * 应用侧边栏 — LobeChat 风格：全高，会话列表按置顶分组。
	 */
	import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
	import MessageSquarePlusIcon from "@lucide/svelte/icons/message-square-plus";
	import PinIcon from "@lucide/svelte/icons/pin";
	import ArchiveIcon from "@lucide/svelte/icons/archive";
	import Trash2Icon from "@lucide/svelte/icons/trash-2";
	import Settings2Icon from "@lucide/svelte/icons/settings-2";
	import SidebarIcon from "@lucide/svelte/icons/sidebar";
	import type { ConversationSummary } from "$lib/transport/conversations";
	import type { ComponentProps } from "svelte";
	import * as Sidebar from "$lib/components/ui/sidebar/index.js";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { formatRelativeTime } from "../../components/workspace-state";

	let {
		ref = $bindable(null),
		conversations,
		activeConversationId,
		loading,
		onCreate,
		onSelect,
		onTogglePin,
		onToggleArchive,
		onDelete,
		onOpenSettings,
		...restProps
	}: ComponentProps<typeof Sidebar.Root> & {
		conversations: ConversationSummary[];
		activeConversationId: string | null;
		loading: boolean;
		onCreate: () => void | Promise<void>;
		onSelect: (id: string) => void | Promise<void>;
		onTogglePin: (conversation: ConversationSummary) => void | Promise<void>;
		onToggleArchive: (conversation: ConversationSummary) => void | Promise<void>;
		onDelete: (id: string) => void | Promise<void>;
		onOpenSettings: () => void;
	} = $props();

	const sidebar = Sidebar.useSidebar();

	/** 置顶的会话。 */
	let pinnedConversations = $derived(conversations.filter((c) => c.pinned));
	/** 非置顶的会话。 */
	let unpinnedConversations = $derived(conversations.filter((c) => !c.pinned));
</script>

<Sidebar.Root bind:ref {...restProps}>
	<Sidebar.Header>
		<div class="flex items-center justify-between px-2 py-1">
			<span class="text-sm font-semibold">BuYu</span>
			<div class="flex items-center gap-1">
				<Button class="size-7" onclick={onCreate} size="icon" variant="ghost" title="新建会话">
					<MessageSquarePlusIcon class="size-4" />
				</Button>
				<Button class="size-7" onclick={sidebar.toggle} size="icon" variant="ghost" title="收起侧栏">
					<SidebarIcon class="size-4" />
				</Button>
			</div>
		</div>
	</Sidebar.Header>

	<Sidebar.Content>
		{#if loading}
			<Sidebar.Group>
				<Sidebar.Menu>
					<Sidebar.MenuItem>
						<Sidebar.MenuButton aria-disabled="true">
							<span class="text-xs text-muted-foreground">正在加载...</span>
						</Sidebar.MenuButton>
					</Sidebar.MenuItem>
				</Sidebar.Menu>
			</Sidebar.Group>
		{:else if conversations.length === 0}
			<Sidebar.Group>
				<Sidebar.Menu>
					<Sidebar.MenuItem>
						<Sidebar.MenuButton aria-disabled="true">
							<span class="text-xs text-muted-foreground">还没有会话</span>
						</Sidebar.MenuButton>
					</Sidebar.MenuItem>
				</Sidebar.Menu>
			</Sidebar.Group>
		{:else}
			{#if pinnedConversations.length > 0}
				<Sidebar.Group>
					<Sidebar.GroupLabel class="text-[11px] uppercase tracking-wider">
						<PinIcon class="mr-1 size-3" />置顶
					</Sidebar.GroupLabel>
					<Sidebar.Menu>
						{#each pinnedConversations as conversation (conversation.id)}
							{@render conversationItem(conversation)}
						{/each}
					</Sidebar.Menu>
				</Sidebar.Group>
			{/if}

			<Sidebar.Group>
				{#if pinnedConversations.length > 0}
					<Sidebar.GroupLabel class="text-[11px] uppercase tracking-wider">最近</Sidebar.GroupLabel>
				{/if}
				<Sidebar.Menu>
					{#each unpinnedConversations as conversation (conversation.id)}
						{@render conversationItem(conversation)}
					{/each}
				</Sidebar.Menu>
			</Sidebar.Group>
		{/if}
	</Sidebar.Content>

	<Sidebar.Footer>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton onclick={onOpenSettings} class="text-muted-foreground">
					<Settings2Icon class="size-4" />
					<span class="text-xs">设置</span>
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>
</Sidebar.Root>

{#snippet conversationItem(conversation: ConversationSummary)}
	<Sidebar.MenuItem>
		<Sidebar.MenuButton
			isActive={conversation.id === activeConversationId}
			onclick={() => onSelect(conversation.id)}
			tooltipContent={conversation.title}
		>
			<span class="truncate text-[13px]">{conversation.title}</span>
		</Sidebar.MenuButton>

		<DropdownMenu.Root>
			<DropdownMenu.Trigger>
				{#snippet child({ props })}
					<Sidebar.MenuAction showOnHover {...props}>
						<EllipsisIcon class="size-4" />
						<span class="sr-only">更多</span>
					</Sidebar.MenuAction>
				{/snippet}
			</DropdownMenu.Trigger>

			<DropdownMenu.Content
				class="w-36"
				side={sidebar.isMobile ? "bottom" : "right"}
				align="start"
			>
				<DropdownMenu.Item onclick={() => onTogglePin(conversation)}>
					<PinIcon class="text-muted-foreground" />
					<span>{conversation.pinned ? "取消置顶" : "置顶"}</span>
				</DropdownMenu.Item>
				<DropdownMenu.Item onclick={() => onToggleArchive(conversation)}>
					<ArchiveIcon class="text-muted-foreground" />
					<span>{conversation.archived ? "取消归档" : "归档"}</span>
				</DropdownMenu.Item>
				<DropdownMenu.Separator />
				<DropdownMenu.Item onclick={() => onDelete(conversation.id)}>
					<Trash2Icon class="text-muted-foreground" />
					<span>删除</span>
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</Sidebar.MenuItem>
{/snippet}
