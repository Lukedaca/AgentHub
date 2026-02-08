<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';

  import Titlebar from '$lib/components/Titlebar.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import ChatPanel from '$lib/components/ChatPanel.svelte';
  import InputBar from '$lib/components/InputBar.svelte';

  import type { Agent, Message } from '$lib/types.js';

  // ---- State ----

  interface DiscoveredAgent {
    id: string;
    name: string;
    short_name: string;
    command: string;
    path: string;
    color: string;
    version: string;
    available: boolean;
  }

  let agents: Agent[] = $state([]);
  let activeAgentId: string = $state('');
  let discoveryDone = $state(false);

  async function discoverAgents() {
    try {
      const discovered: DiscoveredAgent[] = await invoke('discover_agents');
      agents = discovered
        .filter((d) => d.available)
        .map((d) => ({
          id: d.id,
          name: d.name,
          shortName: d.short_name,
          command: d.command,
          args: [],
          status: 'offline' as const,
          color: d.color,
          version: d.version || '',
          path: d.path || '',
        }));
      if (agents.length > 0) {
        activeAgentId = agents[0].id;
      }
    } catch (err) {
      console.error('Agent discovery failed:', err);
      agents = [{
        id: 'claude', name: 'Claude Code', shortName: 'CC',
        command: 'claude', args: [], status: 'offline', color: '#00FF64',
      }];
      activeAgentId = 'claude';
    }
    discoveryDone = true;
  }

  let messagesMap: Map<string, Message[]> = $state(new Map());

  $effect(() => {
    for (const agent of agents) {
      if (!messagesMap.has(agent.id)) {
        messagesMap.set(agent.id, []);
      }
    }
  });

  // ---- Derived state ----

  let activeAgent: Agent | null = $derived(
    agents.find((a) => a.id === activeAgentId) ?? null
  );

  let activeMessages: Message[] = $derived(
    messagesMap.get(activeAgentId) ?? []
  );

  // ---- Helpers ----

  function genId(): string {
    return Date.now().toString(36) + Math.random().toString(36).slice(2, 8);
  }

  function pushMessage(agentId: string, role: Message['role'], text: string, tokens = 0, files = 0) {
    const msgs = messagesMap.get(agentId) ?? [];
    msgs.push({
      id: genId(),
      role,
      text,
      timestamp: Date.now(),
      tokens,
      files,
    });
    messagesMap = new Map(messagesMap);
  }

  function setAgentStatus(agentId: string, status: Agent['status']) {
    agents = agents.map((a) => (a.id === agentId ? { ...a, status } : a));
  }

  // ---- Actions ----

  function parseCommand(text: string): { type: string; target?: string } | null {
    const lower = text.toLowerCase().trim();
    if (/^(help|pomoc|příkazy|prikazy|\?)$/.test(lower)) return { type: 'help' };
    // Přepnutí agenta
    if (/^(přepni|prepni|switch)\s+(.+)$/i.test(lower)) {
      const match = lower.match(/^(?:přepni|prepni|switch)\s+(.+)$/i);
      return { type: 'switch', target: match?.[1] };
    }
    // Skenování agentů
    if (/^(skenuj|scan|hledej agenty|najdi agenty)$/i.test(lower)) return { type: 'scan' };
    return null;
  }

  function findAgentByName(name: string): Agent | undefined {
    const lower = name.toLowerCase();
    return agents.find((a) =>
      a.id.toLowerCase() === lower ||
      a.name.toLowerCase().includes(lower) ||
      a.shortName.toLowerCase() === lower
    );
  }

  async function handleSend(text: string) {
    if (!activeAgent) return;

    // Zkusit příkaz
    const cmd = parseCommand(text);
    if (cmd) {
      pushMessage(activeAgentId, 'user', text);
      handleCommand(cmd);
      return;
    }

    pushMessage(activeAgentId, 'user', text);

    // Odeslat zprávu agentovi (one-shot: spustí příkaz -p "zpráva")
    setAgentStatus(activeAgentId, 'running');

    try {
      await invoke('run_agent', {
        id: activeAgentId,
        command: activeAgent.command,
        message: text,
      });
    } catch (err: any) {
      setAgentStatus(activeAgentId, 'error');
      pushMessage(activeAgentId, 'system', `[red]Chyba: ${err}[/red]`);
    }
  }

  function handleCommand(cmd: { type: string; target?: string }) {
    switch (cmd.type) {
      case 'switch': {
        if (cmd.target) {
          const agent = findAgentByName(cmd.target);
          if (agent) {
            activeAgentId = agent.id;
            pushMessage(agent.id, 'system', `Přepnuto na ${agent.name}.`);
          } else {
            pushMessage(activeAgentId, 'system', `[red]Agent "${cmd.target}" nenalezen.[/red]`);
          }
        }
        break;
      }
      case 'scan': {
        pushMessage(activeAgentId, 'system', 'Skenuji nainstalované agenty...');
        discoverAgents();
        break;
      }
      case 'help': {
        const agentList = agents.map(a => `  ${a.shortName} - ${a.name} (${a.command})`).join('\n');
        pushMessage(activeAgentId, 'system',
          '[bold]Agent Hub - Nápověda[/bold]\n\n' +
          'Každá zpráva se odešle aktuálnímu agentovi.\n' +
          'Agent zpracuje dotaz a vrátí odpověď.\n\n' +
          '[bold]Příkazy:[/bold]\n' +
          '  přepni claude - přepne na jiného agenta\n' +
          '  skenuj - znovu naskenuje systém\n' +
          '  help / pomoc - zobrazí tuto nápovědu\n\n' +
          '[bold]Nalezení agenti:[/bold]\n' +
          agentList
        );
        break;
      }
    }
  }

  function selectAgent(id: string) {
    activeAgentId = id;
  }

  function addAgent() {
    const newId = `agent-${genId()}`;
    agents = [
      ...agents,
      {
        id: newId,
        name: 'Vlastní agent',
        shortName: 'VA',
        command: '',
        args: [],
        status: 'offline',
        color: '#9333EA',
      },
    ];
    activeAgentId = newId;
  }

  function openSettings() {
    pushMessage(activeAgentId, 'system', '[dim]Nastavení zatím není dostupné.[/dim]');
  }

  // ---- Tauri event listeners ----

  onMount(() => {
    discoverAgents();

    let unlistenOutput: (() => void) | null = null;
    let unlistenDone: (() => void) | null = null;

    listen<{
      id: string;
      data: string;
      stream: string;
    }>('agent-output', (event) => {
      const { id, data } = event.payload;
      pushMessage(id, 'agent', data);
    }).then((fn) => {
      unlistenOutput = fn;
    });

    listen<{
      id: string;
      code: number | null;
    }>('agent-done', (event) => {
      const { id, code } = event.payload;
      if (code === 0 || code === null) {
        setAgentStatus(id, 'offline');
      } else {
        setAgentStatus(id, 'error');
        pushMessage(id, 'system', `[red]Agent skončil s chybou (kód ${code})[/red]`);
      }
    }).then((fn) => {
      unlistenDone = fn;
    });

    return () => {
      unlistenOutput?.();
      unlistenDone?.();
    };
  });
</script>

<div class="app-shell">
  <Titlebar />

  <div class="app-body">
    <Sidebar
      {agents}
      {activeAgentId}
      onSelectAgent={selectAgent}
      onAddAgent={addAgent}
      onOpenSettings={openSettings}
    />

    <div class="main-area">
      <ChatPanel
        agent={activeAgent}
        messages={activeMessages}
        onSendMessage={handleSend}
      />

      <InputBar
        onSend={handleSend}
        disabled={!activeAgent}
      />
    </div>
  </div>
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--bg, #0a0a0a);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .app-body {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .main-area {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    background: rgba(10, 10, 10, 0.75);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }
</style>
