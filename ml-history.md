# Modlist Launcher history

What has been built and when, newest first. The plan page
(https://claude.ai/artifact/2qaZ63YM974LMKSFwgbQZD) shows what is upcoming and
links here; the state file (`ml-project-state.md`) holds current working state.
Plain Markdown on purpose: quick to read on GitHub or in an editor
(`docs/project-plan-artifact.md`, "Where history lives", in agentic-instructions).
Started 2026-11-23 from `templates/project-history.md`.

Moved from the plan page's Done tab on 2026-11-23, as written there; an entry's "now", "still" or "not yet" means as of its own date.

PR references rechecked 2026-11-23 against GitHub: 32 referenced, 32 merged; not merged: none. (31 in this repo, counting each PR in a range such as "#6 to #8", plus agentic-instructions#313.)

## Stages

- **00. The launcher (17 Oct):** Cards, Play and Open MO2, the one-MO2 block, shortcuts, the command line, Steam Play with backup and restore, zip and per-user installer (#1). MO2's waiting dialog minimized (#2), the repaint fix (#4, #5).
- **01. Three fixes (23 Oct):** Steam settings restored on Remove and uninstall, a game that's still closing, arrow keys between cards (#6 to #8).
- **02. Right Steam app, nothing left behind (23 Oct):** The app ID comes from the Play exe and never from modding tools, and stray connections can be restored from Settings (#11). Setup chips on the cards (#12).
- **03. New modlists noticed at startup (24 Oct):** Scan folders checked in the background, with Review or Dismiss; nothing is added without your OK (#15).
- **04. Smoke tests in CI (24 Oct):** Repaint and arrow keys on every PR (#16); real mouse clicks added in #20.
- **+. Look, repo and the stuck MO2 (24 to 25 Oct):** The Vaexil theme and Geist type (#13, ADR 0002); repo hardening (#17); the vector icon (#18); mouse clicks fixed (#20); Not responding with End MO2 (#21).

## Ledger

- 2026-11-28: `SECURITY.md` corrected to name every time the launcher writes Steam's launch options: Apply, Restore, removing a modlist connected to Steam's Play button, and uninstalling, as the safety model already said (`[docs-check-2026-11]`). Docs only.
- 2026-11-25: Dev builds stopped seeing your real data (agentic-instructions `[app-scratch-safety]`). A build made anywhere but the release workflows now keeps its files in `Modlist Launcher Dev` and leaves Steam alone unless asked by name, and scratch mode refuses your real data folder, the real Steam folder and starting any real MO2 or game, so tests and smoke runs cannot touch them. The launcher's data folder is now readable only by your Windows account. Your installed launcher and its Steam Play hooks are unchanged.
- 2026-11-18: Main builds for Atlas's Reinstall, and releases only when you ask ([#37](https://github.com/JAMARQ-Digital-LLC/modlist-launcher/pull/37), merged 19 Nov as `c61e62b`). A new Main build workflow builds main with the version stamped `0.1.0+main.<commit>` and keeps the installer for 7 days, publishing nothing. Its test run on the branch read `0.1.0+main.439a976` back from the installer, the uninstall entry, the installed exe and the Settings line, and the install stayed in `%LOCALAPPDATA%\Programs\Modlist Launcher`, where your Steam Play hooks point. That install was on GitHub's throwaway runner, not a PC. The Release workflow now runs only when you start it, and the feed still moves only when you publish its draft. The repo is public, so anyone signed in to GitHub can download a main build; the source is public too.
- 2026-11-18: Tier A auto-merge, your call: Dependabot minor and patch updates now merge themselves once the required checks pass (modlist-launcher#36); majors still wait for you.
- 2026-11-18: The repo moved from `jmars319` into `JAMARQ-Digital-LLC`. This reverses the 17 Oct home, by your decision, so the PCs need only one org-owned GitHub token. Its id, settings, `main` protection and the releases secret came across, old links redirect, and clones on the Mac and both PCs point at the new address. modlist-launcher#35 updates the links in the repo.
- 2026-11-18: Next Up A, Windows builds for Atlas's Reinstall (#33). Stages 05, 06 and 08 carded as Next Up B to D (#34) after the JASON-PC sessions were read back on the Mac; the Claude option was already recorded here, nothing pending. Docs-only PRs skip the required CI jobs, and chrono RUSTSEC-2026-0543 is ignored until 17 Feb 2027 (17 Nov).
- 2026-11-13: Plan page moved to tabs and re-checked first-hand: launcher log, Steam's settings, shortcuts, installed exe, tests. State file added (#27). You set code signing aside: the launcher is a tool for now and may fold into Vaexil Mod Manager.
- 2026-11-08: The account moved from "false" to "Jason". The launcher was reinstalled per-user with its saved modlists, all four Steam connections were re-applied, and the launcher's shortcuts were recreated. Release copies to the shared feed (#23, #26), the Slint advisories cleared (#24, pin-utils ignored until 8 Feb 2027), and a flaky fake-MO2 test fixed (#25).
- 2026-11-01: uuid 1.29 (#22).
- 2026-10-26: End MO2 used for real: a windowless Project Ultrapunk MO2 (pid 15808) was ended through the card.
- 2026-10-25: A hung MO2 held 15.7 GB for 14 hours; Not responding and End MO2 added and installed (#21).
- 2026-10-24: Vaexil look (#13), installer shortcut icons (#14), stage 03 (#15), stage 04 (#16), repo setup (#17), vector icon (#18), mouse clicks (#20).
- 2026-10-23: First full check of reality and this page (#9). Fixes #6 to #8, game-exe shortcut icons (#10), the Steam app ID fix (#11), setup chips (#12). Stray Steam app 1946160 restored.
- 2026-10-19: You added Frostbound and Life in the Ruins and connected both to Steam Play.
- 2026-10-17: Built, tested on your real installs, installed and registered (#1 to #5, agentic-instructions #313).

## Owner setup completed

- Per-user install in `%LOCALAPPDATA%\Programs\Modlist Launcher`, with Start menu and desktop shortcuts.
- Steam Play on all four modlists, one desktop shortcut per modlist.
- Repo secret `JAMARQ_RELEASES_TOKEN` set on 8 Nov for the feed copy.
