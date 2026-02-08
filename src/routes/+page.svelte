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
      // Zobrazit jen nalezené agenty
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

  // Ensure each agent has an empty messages array
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
    // Trigger reactivity by reassigning the map
    messagesMap = new Map(messagesMap);
  }

  function setAgentStatus(agentId: string, status: Agent['status']) {
    agents = agents.map((a) => (a.id === agentId ? { ...a, status } : a));
  }

  // ---- Actions ----

  // Rozpoznání příkazů v chatu
  function parseCommand(text: string): { type: string; target?: string } | null {
    const lower = text.toLowerCase().trim();
    // Spuštění
    if (/^(zapni|spusť|start|spustit)\s*(agenta?|vse|vše|all)?$/i.test(lower)) {
      if (/vse|vše|all|agenty/.test(lower)) return { type: 'start-all' };
      return { type: 'start' };
    }
    if (/^(zapni|spusť|start|spustit)\s+(.+)$/i.test(lower)) {
      const match = lower.match(/^(?:zapni|spusť|start|spustit)\s+(.+)$/i);
      return { type: 'start', target: match?.[1] };
    }
    // Zastavení
    if (/^(vypni|zastav|stop|zastavit)\s*(agenta?|vse|vše|all)?$/i.test(lower)) {
      if (/vse|vše|all|agenty/.test(lower)) return { type: 'stop-all' };
      return { type: 'stop' };
    }
    if (/^(vypni|zastav|stop|zastavit)\s+(.+)$/i.test(lower)) {
      const match = lower.match(/^(?:vypni|zastav|stop|zastavit)\s+(.+)$/i);
      return { type: 'stop', target: match?.[1] };
    }
    // Help
    if (/^(help|pomoc|příkazy|prikazy|\?)$/.test(lower)) return { type: 'help' };
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
      await handleCommand(cmd);
      return;
    }

    pushMessage(activeAgentId, 'user', text);

    // Automaticky spustit agenta pokud je offline
    if (activeAgent.status === 'offline') {
      await startAgent(activeAgentId);
    }

    try {
      await invoke('send_to_agent', {
        id: activeAgentId,
        input: text,
      });
    } catch (err: any) {
      pushMessage(activeAgentId, 'system', `[red]Chyba při odesílání: ${err}[/red]`);
    }
  }

  async function handleCommand(cmd: { type: string; target?: string }) {
    switch (cmd.type) {
      case 'start': {
        if (cmd.target) {
          const agent = findAgentByName(cmd.target);
          if (agent) {
            activeAgentId = agent.id;
            await startAgent(agent.id);
          } else {
            pushMessage(activeAgentId, 'system', `[red]Agent "${cmd.target}" nenalezen.[/red]`);
          }
        } else {
          await startAgent(activeAgentId);
        }
        break;
      }
      case 'start-all': {
        for (const agent of agents) {
          if (agent.status === 'offline') {
            await startAgent(agent.id);
          }
        }
        break;
      }
      case 'stop': {
        if (cmd.target) {
          const agent = findAgentByName(cmd.target);
          if (agent) {
            await stopAgent(agent.id);
          } else {
            pushMessage(activeAgentId, 'system', `[red]Agent "${cmd.target}" nenalezen.[/red]`);
          }
        } else {
          await stopAgent(activeAgentId);
        }
        break;
      }
      case 'stop-all': {
        for (const agent of agents) {
          if (agent.status === 'running') {
            await stopAgent(agent.id);
          }
        }
        break;
      }
      case 'help': {
        pushMessage(activeAgentId, 'system',
          '[bold]Dostupné příkazy:[/bold]\n' +
          '  zapni / start - spustí aktuálního agenta\n' +
          '  zapni claude - spustí konkrétního agenta\n' +
          '  zapni vše - spustí všechny agenty\n' +
          '  vypni / stop - zastaví aktuálního agenta\n' +
          '  vypni vše - zastaví všechny agenty\n' +
          '  help / pomoc - zobrazí tuto nápovědu'
        );
        break;
      }
    }
  }

  async function startAgent(agentId: string) {
    const agent = agents.find((a) => a.id === agentId);
    if (!agent) return;

    setAgentStatus(agentId, 'starting');
    pushMessage(agentId, 'system', `Spouštím ${agent.name}...`);

    try {
      await invoke('spawn_agent', {
        id: agent.id,
        command: agent.command,
        args: agent.args,
      });
      setAgentStatus(agentId, 'running');
      pushMessage(agentId, 'system', `[green]${agent.name} spuštěn.[/green]`);
    } catch (err: any) {
      setAgentStatus(agentId, 'error');
      pushMessage(agentId, 'system', `[red]Chyba při spouštění: ${err}[/red]`);
    }
  }

  async function stopAgent(agentId: string) {
    try {
      await invoke('stop_agent', { id: agentId });
      setAgentStatus(agentId, 'offline');
      pushMessage(agentId, 'system', 'Agent zastaven');
    } catch (err: any) {
      pushMessage(agentId, 'system', `[red]Chyba při zastavování: ${err}[/red]`);
    }
  }

  function selectAgent(id: string) {
    activeAgentId = id;
  }

  function addAgent() {
    // Placeholder: in future this will open a dialog
    const newId = `agent-${genId()}`;
    agents = [
      ...agents,
      {
        id: newId,
        name: 'Nový agent',
        shortName: 'NA',
        command: '',
        args: [],
        status: 'offline',
        color: '#9333EA',
      },
    ];
    activeAgentId = newId;
  }

  function openSettings() {
    // Placeholder for settings panel
    pushMessage(activeAgentId, 'system', '[dim]Nastavení zatím není dostupné.[/dim]');
  }

  // ---- Tauri event listeners ----

  onMount(() => {
    // Discover installed agents
    discoverAgents();

    // Initialize messages map
    for (const agent of agents) {
      if (!messagesMap.has(agent.id)) {
        messagesMap.set(agent.id, []);
      }
    }

    let unlisten: (() => void) | null = null;

    listen<{
      id: string;
      data: string;
      stream: string;
    }>('agent-output', (event) => {
      const { id, data, stream } = event.payload;
      pushMessage(id, 'agent', data, 0, 0);
    }).then((fn) => {
      unlisten = fn;
    });

    // Listen for agent exit events
    let unlistenExit: (() => void) | null = null;

    listen<{
      id: string;
      code: number | null;
    }>('agent-exit', (event) => {
      const { id, code } = event.payload;
      if (code === 0 || code === null) {
        setAgentStatus(id, 'offline');
        pushMessage(id, 'system', 'Agent ukončen.');
      } else {
        setAgentStatus(id, 'error');
        pushMessage(id, 'system', `[red]Agent ukončen s kódem ${code}[/red]`);
      }
    }).then((fn) => {
      unlistenExit = fn;
    });

    return () => {
      unlisten?.();
      unlistenExit?.();
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
