<script lang="ts">
  interface Props {
    onSend: (text: string) => void;
    disabled?: boolean;
    placeholder?: string;
  }

  let { onSend, disabled = false, placeholder = 'Zadej příkaz nebo zprávu...' }: Props = $props();

  let text = $state('');
  let textareaEl: HTMLTextAreaElement | undefined = $state();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  }

  function send() {
    const trimmed = text.trim();
    if (!trimmed || disabled) return;
    onSend(trimmed);
    text = '';
    // Reset height
    if (textareaEl) {
      textareaEl.style.height = 'auto';
    }
  }

  function autoGrow() {
    if (!textareaEl) return;
    textareaEl.style.height = 'auto';
    const max = 140;
    textareaEl.style.height = Math.min(textareaEl.scrollHeight, max) + 'px';
  }

  $effect(() => {
    // Reactively auto-grow when text changes
    text;
    if (textareaEl) {
      requestAnimationFrame(autoGrow);
    }
  });
</script>

<div class="input-bar" class:disabled>
  <div class="input-bar-inner">
    <!-- Mic button -->
    <button class="input-icon-btn" title="Hlasový vstup" disabled={disabled}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
        <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
        <line x1="12" y1="19" x2="12" y2="23" />
        <line x1="8" y1="23" x2="16" y2="23" />
      </svg>
    </button>

    <!-- Screenshot button -->
    <button class="input-icon-btn" title="Snímek obrazovky" disabled={disabled}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z" />
        <circle cx="12" cy="13" r="4" />
      </svg>
    </button>

    <!-- Text input -->
    <textarea
      bind:this={textareaEl}
      bind:value={text}
      onkeydown={handleKeydown}
      oninput={autoGrow}
      {placeholder}
      {disabled}
      rows="1"
      class="input-textarea"
    ></textarea>

    <!-- Send button -->
    <button
      class="send-btn"
      onclick={send}
      disabled={disabled || text.trim().length === 0}
      title="Odeslat (Enter)"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="22" y1="2" x2="11" y2="13" />
        <polygon points="22 2 15 22 11 13 2 9 22 2" />
      </svg>
    </button>
  </div>
</div>

<style>
  .input-bar {
    padding: 8px 16px 12px;
    background: rgba(8, 8, 8, 0.6);
    border-top: 1px solid rgba(255, 255, 255, 0.04);
  }

  .input-bar.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .input-bar-inner {
    display: flex;
    align-items: flex-end;
    gap: 6px;
    padding: 6px 8px;
    background: rgba(10, 10, 10, 0.75);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(0, 255, 100, 0.15);
    border-radius: 12px;
    transition: border-color var(--transition-normal, 200ms ease), box-shadow var(--transition-normal, 200ms ease);
  }

  .input-bar-inner:focus-within {
    border-color: rgba(0, 255, 100, 0.3);
    box-shadow: 0 0 20px rgba(0, 255, 100, 0.05);
  }

  .input-textarea {
    flex: 1;
    resize: none;
    min-height: 24px;
    max-height: 140px;
    padding: 4px 8px;
    font-size: 13.5px;
    line-height: 1.5;
    color: var(--text, #E0E0E0);
    background: transparent;
    border: none;
    outline: none;
    overflow-y: auto;
  }

  .input-textarea::placeholder {
    color: var(--text-muted, #444);
  }

  .input-icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 8px;
    color: var(--text-dimmed, #666);
    transition: all var(--transition-fast, 120ms ease);
    flex-shrink: 0;
  }

  .input-icon-btn:hover:not(:disabled) {
    color: var(--text, #E0E0E0);
    background: rgba(255, 255, 255, 0.05);
  }

  .input-icon-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .send-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: 10px;
    background: var(--primary, #00FF64);
    color: #000;
    flex-shrink: 0;
    transition: all var(--transition-fast, 120ms ease);
  }

  .send-btn:hover:not(:disabled) {
    background: #33FF83;
    box-shadow: 0 0 16px rgba(0, 255, 100, 0.3);
  }

  .send-btn:active:not(:disabled) {
    transform: scale(0.95);
  }

  .send-btn:disabled {
    background: rgba(0, 255, 100, 0.15);
    color: rgba(0, 0, 0, 0.3);
    cursor: not-allowed;
  }
</style>
