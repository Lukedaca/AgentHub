<script lang="ts">
  import type { Agent } from '$lib/types.js';

  interface Props {
    agents: Agent[];
    activeAgentId: string;
    onSelectAgent: (id: string) => void;
    onAddAgent?: () => void;
    onOpenSettings?: () => void;
  }

  let { agents, activeAgentId, onSelectAgent, onAddAgent, onOpenSettings }: Props = $props();

  function statusColor(status: string): string {
    switch (status) {
      case 'running': return 'var(--primary)';
      case 'error': return 'var(--error)';
      case 'unavailable': return '#333';
      default: return 'var(--text-muted)';
    }
  }
</script>

<aside class="sidebar">
  <nav class="sidebar-agents">
    {#each agents as agent (agent.id)}
      <button
        class="agent-btn"
        class:active={activeAgentId === agent.id}
        class:unavailable={agent.status === 'unavailable'}
        onclick={() => onSelectAgent(agent.id)}
        title={agent.status === 'unavailable' ? `${agent.name} (není nainstalovaný)` : agent.name}
      >
        <div
          class="agent-avatar"
          style="--agent-color: {agent.color || 'var(--primary)'}"
        >
          <span class="agent-initials">{agent.shortName}</span>
          <span
            class="status-dot"
            style="background: {statusColor(agent.status)}"
            class:pulse={agent.status === 'running'}
          ></span>
        </div>
      </button>
    {/each}

    <button class="agent-btn add-btn" onclick={onAddAgent} title="Přidat agenta">
      <div class="agent-avatar add-avatar">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </div>
    </button>
  </nav>

  <div class="sidebar-bottom">
    <button class="agent-btn" onclick={onOpenSettings} title="Nastavení">
      <div class="agent-avatar settings-avatar">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      </div>
    </button>
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    width: var(--sidebar-width, 72px);
    min-width: var(--sidebar-width, 72px);
    height: 100%;
    background: rgba(10, 10, 10, 0.75);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-right: 1px solid rgba(0, 255, 100, 0.1);
    padding: 12px 0;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .sidebar-agents {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 0 10px;
  }

  .sidebar-bottom {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 0 10px 4px;
  }

  .agent-btn {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    padding: 4px 0;
    border-radius: 10px;
    transition: all var(--transition-normal, 200ms ease);
  }

  .agent-btn:hover .agent-avatar {
    border-color: rgba(0, 255, 100, 0.25);
    background: rgba(0, 255, 100, 0.06);
  }

  .agent-btn.active .agent-avatar {
    border-color: rgba(0, 255, 100, 0.4);
    box-shadow: 0 0 40px rgba(0, 255, 100, 0.1), inset 0 0 20px rgba(0, 255, 100, 0.03);
    background: rgba(0, 255, 100, 0.08);
  }

  .agent-avatar {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(20, 20, 20, 0.8);
    transition: all var(--transition-normal, 200ms ease);
  }

  .agent-initials {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.5px;
    color: var(--agent-color, var(--primary));
    text-shadow: 0 0 8px color-mix(in srgb, var(--agent-color, var(--primary)) 40%, transparent);
  }

  .status-dot {
    position: absolute;
    bottom: -1px;
    right: -1px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 2px solid var(--bg, #0a0a0a);
    transition: background var(--transition-normal, 200ms ease);
  }

  .status-dot.pulse {
    animation: pulse-green 2s ease-in-out infinite;
  }

  .add-avatar {
    border-style: dashed;
    border-color: rgba(255, 255, 255, 0.1);
    color: var(--text-dimmed, #666);
  }

  .agent-btn:hover .add-avatar {
    border-color: rgba(0, 255, 100, 0.3);
    color: var(--primary);
  }

  .settings-avatar {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    border-color: transparent;
    background: transparent;
    color: var(--text-dimmed, #666);
  }

  .agent-btn:hover .settings-avatar {
    color: var(--text, #E0E0E0);
    background: rgba(255, 255, 255, 0.04);
    border-color: transparent;
  }

  .agent-btn.unavailable {
    opacity: 0.35;
  }

  .agent-btn.unavailable .agent-initials {
    text-decoration: line-through;
  }

  @keyframes pulse-green {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
