<script lang="ts">
  import Badge from "$ui/badge.svelte";
  import Button from "$ui/button.svelte";
  import Card from "$ui/card.svelte";
  import { Database, GitBranchPlus, Sparkles } from "lucide-svelte";

  const channels = ["Local OpenAI", "Claude Gateway", "Gemini Bridge"];
  const conversations = [
    { title: "Schema Review", meta: "3 summaries · 12 nodes" },
    { title: "Workflow Design", meta: "2 agents · running" },
    { title: "Media Storage", meta: "4 files · 1 image" }
  ];
</script>

<svelte:head>
  <title>BuYu</title>
</svelte:head>

<div class="app-shell">
  <Card className="sidebar">
    <div class="brand-block">
      <p class="eyebrow">Tauri 2 + Svelte 5</p>
      <h1>BuYu</h1>
      <p class="lede">Local-first multi-agent workspace with explicit workflow traces.</p>
    </div>

    <section class="sidebar-section">
      <div class="section-header">
        <h2>Channels</h2>
        <Button size="sm">Add</Button>
      </div>
      <ul class="chip-list">
        {#each channels as channel}
          <li>{channel}</li>
        {/each}
      </ul>
    </section>

    <section class="sidebar-section">
      <div class="section-header">
        <h2>Conversations</h2>
        <Button size="sm">New</Button>
      </div>
      <div class="conversation-list">
        {#each conversations as conversation}
          <article class="conversation-card">
            <h3>{conversation.title}</h3>
            <p>{conversation.meta}</p>
          </article>
        {/each}
      </div>
    </section>
  </Card>

  <main class="workspace">
    <Card className="hero-card">
      <div>
        <p class="eyebrow">Frontend reset in progress</p>
        <h2>Svelte shell is active</h2>
      </div>
      <p>
        Next step is wiring Tailwind and shadcn-svelte components onto this shell, then
        reconnecting the Tauri command layer.
      </p>
      <div class="hero-actions">
        <Badge>Local-first</Badge>
        <Badge className="bg-orange-50 text-orange-700">Workflow-ready</Badge>
      </div>
    </Card>

    <section class="panel-grid">
      <Card className="panel">
        <header>
          <span>Context</span>
          <strong>Viewer</strong>
        </header>
        <Database class="panel-icon" size={20} />
        <p>Messages, summaries and hidden layers will be reattached here after the component migration.</p>
      </Card>

      <Card className="panel">
        <header>
          <span>Workflow</span>
          <strong>Execution graph</strong>
        </header>
        <GitBranchPlus class="panel-icon" size={20} />
        <p>Node-level read/write modes will map to UI controls once the new component library is in place.</p>
      </Card>

      <Card className="panel panel-accent">
        <header>
          <span>Status</span>
          <strong>Ready for shadcn-svelte init</strong>
        </header>
        <Sparkles class="panel-icon panel-icon-light" size={20} />
        <p>The React shell has been retired. The project now boots from a native Svelte entrypoint.</p>
      </Card>
    </section>
  </main>
</div>
