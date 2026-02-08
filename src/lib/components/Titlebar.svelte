<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';

  const appWindow = getCurrentWindow();

  async function minimize() {
    try { await appWindow.minimize(); } catch (e) { console.error('minimize failed:', e); }
  }

  async function toggleMaximize() {
    try { await appWindow.toggleMaximize(); } catch (e) { console.error('maximize failed:', e); }
  }

  async function close() {
    try { await appWindow.close(); } catch (e) { console.error('close failed:', e); }
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="titlebar-left" data-tauri-drag-region>
    <span class="titlebar-icon">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polygon points="12 2 22 8.5 22 15.5 12 22 2 15.5 2 8.5 12 2" />
        <line x1="12" y1="22" x2="12" y2="15.5" />
        <polyline points="22 8.5 12 15.5 2 8.5" />
      </svg>
    </span>
    <span class="titlebar-title" data-tauri-drag-region>Agent Hub</span>
  </div>

  <div class="titlebar-controls">
    <button class="titlebar-btn minimize" onclick={minimize} aria-label="Minimize">
      <svg width="10" height="10" viewBox="0 0 10 10">
        <line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" stroke-width="1.2" />
      </svg>
    </button>
    <button class="titlebar-btn maximize" onclick={toggleMaximize} aria-label="Maximize">
      <svg width="10" height="10" viewBox="0 0 10 10">
        <rect x="1" y="1" width="8" height="8" stroke="currentColor" stroke-width="1.2" fill="none" />
      </svg>
    </button>
    <button class="titlebar-btn close" onclick={close} aria-label="Close">
      <svg width="10" height="10" viewBox="0 0 10 10">
        <line x1="1" y1="1" x2="9" y2="9" stroke="currentColor" stroke-width="1.2" />
        <line x1="9" y1="1" x2="1" y2="9" stroke="currentColor" stroke-width="1.2" />
      </svg>
    </button>
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--titlebar-height, 36px);
    min-height: var(--titlebar-height, 36px);
    background: rgba(8, 8, 8, 0.95);
    border-bottom: 1px solid rgba(0, 255, 100, 0.12);
    padding: 0 8px;
    z-index: 1000;
    -webkit-app-region: drag;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    -webkit-app-region: drag;
  }

  .titlebar-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--primary, #00FF64);
    filter: drop-shadow(0 0 4px rgba(0, 255, 100, 0.4));
  }

  .titlebar-title {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.5px;
    color: var(--text, #E0E0E0);
    text-shadow: 0 0 12px rgba(0, 255, 100, 0.15);
    -webkit-app-region: drag;
  }

  .titlebar-controls {
    display: flex;
    align-items: center;
    gap: 2px;
    -webkit-app-region: no-drag;
  }

  .titlebar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 28px;
    border-radius: 6px;
    color: var(--text-dimmed, #666);
    transition: all 150ms ease;
    -webkit-app-region: no-drag;
    pointer-events: auto;
  }

  .titlebar-btn:hover {
    color: var(--text, #E0E0E0);
    background: rgba(255, 255, 255, 0.06);
  }

  .titlebar-btn.close:hover {
    color: #fff;
    background: rgba(255, 59, 59, 0.8);
  }
</style>
