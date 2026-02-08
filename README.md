# Agent Hub

Desktopova aplikace pro spouštění a správu CLI AI agentů z jednoho místa. Žádný terminál, žádné přepínání oken - všechno v jednom.

![Tauri](https://img.shields.io/badge/Tauri_v2-FFC131?style=flat&logo=tauri&logoColor=black)
![Svelte](https://img.shields.io/badge/Svelte_5-FF3E00?style=flat&logo=svelte&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=flat&logo=typescript&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green)

## Co to umí

- **Automatická detekce agentů** - Při startu naskenuje systém (PATH + npm global) a najde nainstalované CLI agenty. Žádné ruční nastavování.
- **Podporovaní agenti** - Claude Code, Codex CLI, Gemini CLI, Aider, Cody, Cursor, Amp, Continue
- **Ovládání příkazy** - Napiš `zapni`, `start claude`, `vypni vše` nebo `help` přímo do chatu
- **Real-time výstup** - Stdout/stderr agenta se streamuje do chatového panelu v reálném čase
- **Detekce verzí** - U každého nalezeného agenta zobrazí verzi (`--version`)
- **Glassmorphism UI** - Tmavý design se zelenými neonovými akcenty a efektem skla

## Stack

| Vrstva | Technologie |
|--------|-------------|
| Frontend | Svelte 5 + SvelteKit + TypeScript |
| Backend | Rust (Tauri v2) |
| Build | Vite 6 |
| Výstup | Windows .exe / .msi / NSIS instalátor |

## Instalace a spuštění

### Požadavky

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/) prerequisites

### Dev mode

```bash
npm install
npm run tauri dev
```

### Release build

```bash
npm run tauri build
```

Výstup:
- `src-tauri/target/release/agent-hub.exe` - Přímé spuštění
- `src-tauri/target/release/bundle/nsis/Agent Hub_0.1.0_x64-setup.exe` - Instalátor
- `src-tauri/target/release/bundle/msi/Agent Hub_0.1.0_x64_en-US.msi` - MSI balíček

## Jak funguje detekce agentů

1. Spustí `npm list -g --json` a hledá známé balíčky (`@anthropic-ai/claude-code`, `@openai/codex`, ...)
2. Projde adresáře v PATH přes `where` (Windows) / `which` (Linux/Mac)
3. U nalezených agentů ověří dostupnost přes `--version` (s 3s timeoutem)
4. Zobrazí pouze agenty, kteří jsou skutečně nainstalovaní

## Příkazy v chatu

| Příkaz | Co udělá |
|--------|----------|
| `zapni` / `start` | Spustí aktuálního agenta |
| `zapni claude` | Spustí konkrétního agenta |
| `zapni vše` / `start all` | Spustí všechny agenty |
| `vypni` / `stop` | Zastaví aktuálního agenta |
| `vypni vše` / `stop all` | Zastaví všechny agenty |
| `help` / `pomoc` | Zobrazí nápovědu |

Cokoliv jiného se odešle jako vstup běžícímu agentovi.

## Struktura projektu

```
AgentHub/
├── src/                        # Frontend (Svelte)
│   ├── lib/
│   │   ├── components/         # Titlebar, Sidebar, ChatPanel, InputBar
│   │   ├── styles/             # Globální CSS / design system
│   │   └── types.ts            # TypeScript typy
│   └── routes/
│       └── +page.svelte        # Hlavní stránka + logika
├── src-tauri/                  # Backend (Rust)
│   ├── src/
│   │   └── lib.rs              # Agent manager, spawn, discovery
│   ├── capabilities/           # Tauri v2 permissions
│   └── tauri.conf.json         # Konfigurace okna
├── package.json
└── vite.config.js
```

## Architektura

```
┌──────────────┐     invoke()     ┌─────────────────┐     spawn     ┌──────────────┐
│   Svelte UI  │ ──────────────►  │  Rust Backend   │ ──────────►  │  CLI Agent   │
│  (WebView)   │ ◄──────────────  │ (AgentManager)  │ ◄──────────  │  (process)   │
└──────────────┘   Tauri events   └─────────────────┘   stdin/out  └──────────────┘
```

## Autor

**Lukáš Drštička** - [@Lukedaca](https://github.com/Lukedaca)

## Licence

MIT
