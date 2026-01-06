<div align="center">


# EssentialsCode

[![Rust](https://img.shields.io/badge/Rust-stable-000000?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![CLI/TUI](https://img.shields.io/badge/TUI-ratatui-7C3AED?style=flat-square)](https://github.com/ratatui/ratatui)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-2EA043?style=flat-square)](#)
[![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![CI](https://img.shields.io/badge/CI-GitHub%20Actions-2088FF?style=flat-square&logo=githubactions&logoColor=white)](#)


[**Quick Start**](#quick-start) · [**How It Works**](#how-it-works) · [**Detectors & Fixers**](#detectors--fixers) · [**Roadmap**](#roadmap) · [**FAQ**](#faq)

</div>

<div align="center">

🧠 **Root-cause detection** <br>
⚡ **Faster debugging** &nbsp;•&nbsp; 🔒 **Offline-first** &nbsp;•&nbsp; 🧩 **Extensible architecture**

</div>

---

> **Status:** Pre-release (Rust CLI/TUI).
> EssentialsCode analyzes errors and sends what went wrong.

---

## Key Features

### 🔍 Root-Cause Error Detection
- Analyzes stack traces and compilation/runtime output.
- Identifies the **actual root cause** (e.g. missing environment variables or dependencies), not just the last error line.
- Covers common error classes: dependencies, module systems, ports, file paths, permissions, JSON parsing, and configuration issues.

### 🧩 Extensible Detectors & Fixers
- Modular architecture built around a clear pipeline: `scanner → parser → fixer`.
- New languages and rules can be added without touching the core engine.

> ⚠️ **Note**
> EssentialsCode is also a personal learning project where I actively practice and improve my **Rust** skills.
> While the tool is fully functional, you may encounter bugs, breaking changes, or experimental behavior as the project evolves.


---

```text
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║   ███████╗███████╗███████╗  ╔═╗╔═╗╔╦╗╔═╗                      ║
    ║   ██╔════╝██╔════╝██╔════╝  ║  ║ ║ ║║║╣                       ║
    ║   █████╗  ███████╗███████╗  ╚═╝╚═╝═╩╝╚═╝                      ║
    ║   ██╔══╝  ╚════██║╚════██║                                    ║
    ║   ███████╗███████║███████║  Smart Error Fixer                 ║
    ║   ╚══════╝╚══════╝╚══════╝  v0.2.0                            ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝


────────────────────────────────────────────────────────────
  Scanning Project
────────────────────────────────────────────────────────────
  → Path: C:\Users\KUBA\PyCharmMiscProject
  → Languages: Python

  → Checking: C:\Users\KUBA\PyCharmMiscProject\test.py
  ✗ Syntax Error:
  → File "C:\Users\KUBA\PyCharmMiscProject\test.py", line 1

  ✗ SyntaxError: invalid syntax. Did you mean 'def'?


────────────────────────────────────────────────────────────
  Analyzing Error
────────────────────────────────────────────────────────────

  → Language: Python
  📄 C:\Users\KUBA\PyCharmMiscProject\test.py:1

  ✗ SyntaxError: invalid syntax. Did you mean 'def'?

────────────────────────────────────────────────────────────
  Syntax Error
────────────────────────────────────────────────────────────


────────────────────────────────────────────────────────────
  How to Fix
────────────────────────────────────────────────────────────

  Syntax error: invalid syntax. Did you mean 'def'?

  Check the line indicated in the error for typos or missing syntax.


  ● 1 error found
