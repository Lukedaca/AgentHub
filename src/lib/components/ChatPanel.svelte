<script lang="ts">
  import type { Agent, Message } from '$lib/types.js';

  interface Props {
    agent: Agent | null;
    messages: Message[];
    onSendMessage?: (text: string) => void;
  }

  let { agent, messages }: Props = $props();

  let messagesContainer: HTMLDivElement | undefined = $state();

  // Auto-scroll to bottom when messages change
  $effect(() => {
    if (messages.length && messagesContainer) {
      // Small delay to let DOM update
      requestAnimationFrame(() => {
        if (messagesContainer) {
          messagesContainer.scrollTop = messagesContainer.scrollHeight;
        }
      });
    }
  });

  function statusLabel(status: string): string {
    switch (status) {
      case 'running': return 'Aktivní';
      case 'error': return 'Chyba';
      case 'starting': return 'Startuje...';
      case 'unavailable': return 'Nedostupný';
      default: return 'Offline';
    }
  }

  function statusClass(status: string): string {
    switch (status) {
      case 'running': return 'status-running';
      case 'error': return 'status-error';
      case 'starting': return 'status-starting';
      default: return 'status-offline';
    }
  }

  function formatTime(ts: number): string {
    const d = new Date(ts);
    return d.toLocaleTimeString('cs-CZ', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }

  function tokenCount(): number {
    return messages.reduce((acc, m) => acc + (m.tokens || 0), 0);
  }

  function filesCount(): number {
    return messages.reduce((acc, m) => acc + (m.files || 0), 0);
  }

  function elapsedTime(): string {
    if (messages.length === 0) return '0s';
    const first = messages[0].timestamp;
    const last = messages[messages.length - 1].timestamp;
    const diff = Math.round((last - first) / 1000);
    if (diff < 60) return `${diff}s`;
    const mins = Math.floor(diff / 60);
    const secs = diff % 60;
    return `${mins}m ${secs}s`;
  }

  // Parse simple ANSI-like markers in agent output
  // [green]text[/green], [red]text[/red], [bold]text[/bold]
  function parseOutput(text: string): string {
    return text
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/\[green\](.*?)\[\/green\]/gs, '<span class="out-green">$1</span>')
      .replace(/\[red\](.*?)\[\/red\]/gs, '<span class="out-red">$1</span>')
      .replace(/\[yellow\](.*?)\[\/yellow\]/gs, '<span class="out-yellow">$1</span>')
      .replace(/\[bold\](.*?)\[\/bold\]/gs, '<span class="out-bold">$1</span>')
      .replace(/\[dim\](.*?)\[\/dim\]/gs, '<span class="out-dim">$1</span>');
  }
</script>

<div class="chat-panel">
  {#if agent}
    <!-- Header -->
    <div class="chat-header">
      <div class="chat-header-left">
        <span class="chat-agent-name" style="color: {agent.color || 'var(--primary)'}">{agent.name}</span>
        <span class="chat-status-badge {statusClass(agent.status)}">
          {#if agent.status === 'running'}
            <span class="activity-dot"></span>
          {/if}
          {statusLabel(agent.status)}
        </span>
      </div>
      <div class="chat-header-right">
        {#if agent.version}
          <span class="chat-version">{agent.version}</span>
        {/if}
        {#if agent.command}
          <span class="chat-command">{agent.command}</span>
        {/if}
      </div>
    </div>

    <!-- Messages -->
    <div class="chat-messages" bind:this={messagesContainer}>
      {#if messages.length === 0}
        <div class="chat-empty">
          <div class="chat-empty-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--text-muted)">
              <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
            </svg>
          </div>
          <p class="chat-empty-text">Žádný výstup. Napiš "help" pro seznam příkazů.</p>
        </div>
      {:else}
        {#each messages as msg (msg.id)}
          <div class="chat-msg" class:msg-user={msg.role === 'user'} class:msg-agent={msg.role === 'agent'} class:msg-system={msg.role === 'system'}>
            <div class="msg-meta">
              <span class="msg-role">{msg.role === 'user' ? 'Ty' : msg.role === 'system' ? 'System' : agent.shortName}</span>
              <span class="msg-time">{formatTime(msg.timestamp)}</span>
            </div>
            <div class="msg-content" class:msg-content-mono={msg.role === 'agent'}>
              {#if msg.role === 'agent'}
                {@html parseOutput(msg.text)}
              {:else}
                {msg.text}
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <!-- Status bar -->
    <div class="chat-statusbar">
      <span class="statusbar-item">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
        {elapsedTime()}
      </span>
      <span class="statusbar-item">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>
        {tokenCount().toLocaleString('cs-CZ')} tokens
      </span>
      <span class="statusbar-item">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
        {filesCount()} souborů
      </span>
      <span class="statusbar-item">
        {messages.length} zpráv
      </span>
    </div>
  {:else}
    <div class="chat-empty-state">
      <div class="empty-logo">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--primary)" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" opacity="0.3">
          <polygon points="12 2 22 8.5 22 15.5 12 22 2 15.5 2 8.5 12 2" />
          <line x1="12" y1="22" x2="12" y2="15.5" />
          <polyline points="22 8.5 12 15.5 2 8.5" />
        </svg>
      </div>
      <p class="empty-title">Agent Hub</p>
      <p class="empty-subtitle">Vyber agenta v bočním panelu</p>
    </div>
  {/if}
</div>

<style>
  .chat-panel {
    display: flex;
    flex-direction: column;
    flex: 1;
    height: 100%;
    min-width: 0;
    overflow: hidden;
  }

  /* Header */
  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    min-height: 48px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    background: rgba(10, 10, 10, 0.5);
  }

  .chat-header-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .chat-header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .chat-agent-name {
    font-size: 15px;
    font-weight: 600;
    letter-spacing: 0.3px;
  }

  .chat-status-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-weight: 500;
    padding: 2px 8px;
    border-radius: 20px;
    letter-spacing: 0.3px;
  }

  .status-running {
    background: rgba(0, 255, 100, 0.1);
    color: var(--primary);
  }

  .status-error {
    background: rgba(255, 59, 59, 0.1);
    color: var(--error);
  }

  .status-starting {
    background: rgba(255, 184, 0, 0.1);
    color: var(--warning);
  }

  .status-offline {
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-dimmed);
  }

  .activity-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--primary);
    animation: pulse-green 2s ease-in-out infinite;
  }

  .chat-version {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    padding: 2px 6px;
    background: rgba(0, 255, 100, 0.05);
    border: 1px solid rgba(0, 255, 100, 0.1);
    border-radius: 4px;
  }

  .chat-command {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-dimmed);
    padding: 2px 8px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 4px;
  }

  /* Messages */
  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .chat-msg {
    animation: fade-in 200ms ease;
  }

  .msg-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 3px;
  }

  .msg-role {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-dimmed);
  }

  .msg-user .msg-role {
    color: var(--primary-dim);
  }

  .msg-system .msg-role {
    color: var(--warning);
  }

  .msg-time {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .msg-content {
    font-size: 13px;
    line-height: 1.6;
    color: var(--text);
    padding: 8px 12px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.03);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .msg-content-mono {
    font-family: var(--font-mono);
    font-size: 12.5px;
    line-height: 1.7;
    background: rgba(0, 0, 0, 0.3);
    border-color: rgba(255, 255, 255, 0.04);
  }

  .msg-user .msg-content {
    background: rgba(0, 255, 100, 0.04);
    border-color: rgba(0, 255, 100, 0.08);
  }

  .msg-system .msg-content {
    background: rgba(255, 184, 0, 0.04);
    border-color: rgba(255, 184, 0, 0.08);
    font-size: 12px;
    color: var(--text-dimmed);
  }

  /* ANSI-like colors in output */
  :global(.out-green) { color: var(--primary); }
  :global(.out-red) { color: var(--error); }
  :global(.out-yellow) { color: var(--warning); }
  :global(.out-bold) { font-weight: 700; color: var(--text-bright); }
  :global(.out-dim) { color: var(--text-dimmed); }

  /* Status bar */
  .chat-statusbar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 6px 20px;
    min-height: 28px;
    border-top: 1px solid rgba(255, 255, 255, 0.03);
    background: rgba(5, 5, 5, 0.6);
  }

  .statusbar-item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-dimmed);
  }

  .statusbar-item svg {
    opacity: 0.5;
  }

  /* Empty states */
  .chat-empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    opacity: 0.4;
  }

  .chat-empty-icon {
    opacity: 0.5;
  }

  .chat-empty-text {
    font-size: 13px;
    color: var(--text-dimmed);
  }

  .chat-empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
  }

  .empty-logo {
    opacity: 0.5;
  }

  .empty-title {
    font-size: 20px;
    font-weight: 600;
    color: var(--text-dimmed);
    letter-spacing: 1px;
  }

  .empty-subtitle {
    font-size: 13px;
    color: var(--text-muted);
  }

  @keyframes fade-in {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @keyframes pulse-green {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
