# Contributing

Bug reports and pull requests are welcome. Modlist Launcher is small on purpose: one job, done safely, with no network access.

## Before you start

- For anything bigger than a fix, open an issue first so we can agree on the approach.
- Read the [safety model](docs/safety-model.md). Changes must keep its promises:
  - never write to an MO2 installation;
  - change Steam settings only on an explicit action, with Steam closed and after a backup;
  - make no network connections;
  - need no admin rights.
- Design decisions are recorded in [docs/adr](docs/adr). The look comes from the Vaexil theme in `crates/app/ui/theme.slint` (ADR 0002). Use its components and `Theme` tokens rather than new colours.

## Working on it

You need the Rust toolchain (stable, MSVC) and Visual Studio Build Tools.

### Dev builds and scratch mode

Any build except the one `scripts/package.ps1` makes in CI is the **dev app** (a debug build, `cargo build --release`, a worktree build). It keeps its files in `%LOCALAPPDATA%\Modlist Launcher Dev`, never `%LOCALAPPDATA%\Modlist Launcher`, and does not read the real Steam unless `MODLIST_LAUNCHER_STEAM=live` asks for it by name. The launcher has no single-instance lock, so a dev copy and the installed one can run side by side.

For tests, smoke scripts and anything an agent runs, use **scratch mode**:

| Variable | Value |
| --- | --- |
| `MODLIST_LAUNCHER_SCRATCH` | `1` |
| `MODLIST_LAUNCHER_DATA_DIR` | A throwaway folder (required). The old name `MODLIST_LAUNCHER_DATA` still works. |
| `MODLIST_LAUNCHER_STEAM` | A fake Steam folder, or unset for no Steam. `live` is refused. |
| `MODLIST_LAUNCHER_NETWORK` | `offline`. The launcher makes no network connections; `live` is refused. |
| `MODLIST_LAUNCHER_SECRETS` | `memory`. The launcher stores no secrets; `keychain` is refused. |

With `MODLIST_LAUNCHER_SCRATCH=1` the launcher refuses to start if the data folder is inside the release or dev data folder, `%APPDATA%` or the real Steam folder (from `HKCU\Software\Valve\Steam\SteamPath`), and it refuses to start a program, write a shortcut or write Steam's settings anywhere outside the scratch data folder and the temp folder. So a scratch run can never start a real MO2 or game, or change the real Steam `localconfig.vdf`. Tests always run in scratch (`cfg(test)`), each test process with its own data folder under temp, and the smoke scripts set all five variables. The data folder's access list grants only your account, set when it is created and again at each launch.

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p modlist-launcher
./scripts/smoke-minimize-restore.ps1
./scripts/smoke-arrow-keys.ps1
```

CI runs all of these on every pull request, plus `cargo deny check`. `cargo deny` bans HTTP and TLS crates and rejects yanked or vulnerable ones. Both CI checks must pass before a pull request can merge. A pull request that changes only root Markdown, Markdown under `docs/` or `LICENSE` skips both checks, which then pass as skipped; a `.github/` change always runs them.

- Keep each pull request to one change.
- Add tests for behaviour that can be tested. Platform-neutral logic belongs in `crates/core`, where it is unit tested.
- For UI changes:
  - check light and dark mode;
  - check keyboard use: Tab, arrow keys between cards, Space/Enter;
  - check that controls have screen-reader labels.
- Update the README or docs when behaviour changes.

## License

By contributing you agree that your work is licensed under [GPL-3.0-only](LICENSE), like the rest of the project.
