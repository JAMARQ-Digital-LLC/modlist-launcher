# Modlist Launcher project state

Plan artifact: https://claude.ai/artifact/2qaZ63YM974LMKSFwgbQZD ("The Modlist
Launcher Plan", the owner-facing page in five tabs; update it in place, never
publish a second one).

History: ml-history.md

Working state across sessions. Durable detail lives in `README.md` and `docs/`
(`docs/safety-model.md`, `docs/steam-play-button.md`, `docs/adr/`); this file
holds what is true right now. Created 13 Nov 2026.

## Core Project Architecture & Stack

- Windows-only Rust + Slint 1.18 desktop app (software renderer), per-user
  Inno Setup installer plus portable zip. Workspace crates under `crates/`; UI
  in `crates/app/ui/*.slint`.
- `crates/app/ui/theme.slint` and `assets/fonts/` are shared byte-for-byte with
  Keeplight and Vaexil Mod Manager. Keep app-specific paths out of the theme;
  font imports live in `app.slint`. Agree any theme change with those sessions.
- Data: `%LOCALAPPDATA%\Modlist Launcher` (`config.json`, `launcher.log`,
  `steam-backups\`), user-only ACL. Install:
  `%LOCALAPPDATA%\Programs\Modlist Launcher`. Dev builds (anything but
  `scripts/package.ps1` in CI) use `Modlist Launcher Dev` and no Steam; scratch
  variables in `CONTRIBUTING.md`.
- CI: `ci.yml` (tests, cargo-deny, three smoke scripts on the release build;
  a `changes` job skips both required jobs on docs-only PRs: root `*.md`,
  `docs/**/*.md`, `LICENSE*`, 2026-11-17),
  `advisory-sweep.yml`, `main-build.yml` (dispatch on main only: version
  stamped `<version>+main.<sha7>` by `scripts/stamp-version.ps1`, read back by
  `scripts/check-installer-version.ps1`, artifact
  `modlist-launcher-main-<full sha>` with the setup exe and `SHA256SUMS.txt`
  for 7 days; publishes nothing), `release.yml` (dispatch on main only, when
  the owner asks: refuses a tagged or released version, tags main's head
  `v<version>`, makes a draft), `publish-feed.yml` (the owner publishing the
  draft copies the installer to `JAMARQ-Digital-LLC/jamarq-releases` with a
  one-hour token from the `jamarq-releases-publisher` App, org secrets
  `JAMARQ_RELEASES_APP_ID` and `JAMARQ_RELEASES_APP_PRIVATE_KEY`, since 2026-11-26).
- Verify: `cargo test --workspace --locked` (114 pass, 2 ignored helpers, on
  25 Nov 2026; always scratch), then `scripts/smoke-*.ps1` against a release
  build. Over SSH the two window tests cannot pass (no desktop) and leave a
  `FakeGame-window.exe` holding the output pipe: run the full suite from an
  interactive `schtasks /IT` task, or `--skip` those two.

## Current Structural Milestones / Active Feature Goals

- **History moved off the plan page (2026-11-23).** Finished work, the dated
  ledger and completed setup now live in `ml-history.md`; the plan page's Done
  tab keeps the last two weeks and links to it (agentic-instructions
  `docs/project-plan-artifact.md`, "Where history lives").
- Order set by the owner on 13 Nov 2026: stage 05 accessibility, then 06 a
  first GitHub Release (`v0.1.0`, unsigned, not announced), then 08 the
  "An update is available" notice.
- Stage 07 code signing is set aside (owner, 13 Nov 2026): the launcher is a
  tool for now and may be absorbed into Vaexil Mod Manager, or kept as a
  separate download. No public launch planned. Revisit signing (SignPath
  Foundation) only if it is offered on its own to other people.
- Stage 05 scope: live Narrator (owner listens), keyboard-only flows, Windows
  contrast themes, display scales 100-200% and mixed-DPI monitors, the
  off-screen card missing from the UIA tree. Leads: Slint's ScrollView already
  scrolls a focused control into view (Vaexil Mod Manager, 12 Nov), so the
  cards' `focused-card` tracking may only be needed for arrow keys.
- Stage 08 reads `modlist-launcher-latest/release.json` (url + sha256) from
  the shared feed: owner leaning yes on 13 Nov, default recorded on the plan.
  The owner's words (23 Oct): no auto-update, but "An update is available"
  with an easy click-to-download-and-install button.
- Claude in the app (draft-only, per agentic-instructions "Claude In The
  Apps", #416) is a future option if a language task appears; nothing planned.
  Recorded on the plan page's Roadmap tab, "Set aside for now" (14 Nov, from
  the App consolidation session); nothing pending (checked 18 Nov).
- Long-run fold: Vaexil Mod Manager's E9 plans an `mo2` backend for existing
  MO2 setups, so this launcher may be absorbed there; its hung-MO2 handling
  (#21) is the model for that backend.

## Active Task State & Known Regressions

- **Scratch safety (2026-11-25, agentic-instructions `[app-scratch-safety]`, branch `fix/scratch-safety`).** Dev identity: only a CI `scripts/package.ps1` build sets `MODLIST_LAUNCHER_RELEASE_BUILD=1` (compile time); every other build keeps data in `%LOCALAPPDATA%\Modlist Launcher Dev` and reads no Steam unless `MODLIST_LAUNCHER_STEAM=live`. Scratch variables `MODLIST_LAUNCHER_SCRATCH`, `_DATA_DIR` (old `_DATA` kept as an alias), `_STEAM` (fake Steam folder), `_NETWORK`, `_SECRETS` in `crates/core/src/scratch.rs`; with `SCRATCH=1` the app refuses to start on the release/dev data folder, `%APPDATA%` or the real HKCU SteamPath, and refuses starting programs, writing shortcuts or writing `localconfig.vdf` outside the scratch data folder and temp. Tests are always scratch (`cfg(test)`); all four scripts set the five variables, and the three smokes refuse the installed exe. Data folder ACL is current user only (no SYSTEM or Administrators: the installer never reads it), applied at every launch, read back by a test. No network and no single-instance lock existed, so nothing to change there. The installed 25 Oct build keeps its folder and the real Steam.
- **CI draft gate (2026-11-24, `[ci-draft-skip-rollout]` in agentic-instructions).** A draft pull request runs only `changes` and its secret scan; the heavy jobs skip until it is marked ready, when `ready_for_review` runs them all. Pushes to `main` and schedules run everything as before.

- **ADRs (2026-11-22).** `docs/architecture-decisions.md` holds 2 local ADRs (0001 Rust and Slint with the software renderer, 0002 the Vaexil theme; each listed with a pointer to its file in `docs/adr/`, numbers unchanged) and the central ADRs this repo follows, generated by agentic-instructions `scripts/adr-sync.mjs` (never edit that section by hand).
- **2026-11-22: `url` 2.5.9 for RUSTSEC-2026-0592** (agentic-instructions `[url-2.5.9-slint]`, branch `fix/url-2.5.9`). `cargo update -p url --precise 2.5.9` moved `url` 2.5.8 to 2.5.9 and `form_urlencoded` 1.2.2 to 1.2.3 in `Cargo.lock`: no manifest change and no advisory exception. `cargo deny check` passes without the ID.
- **2026-11-18: main builds for Atlas's Reinstall** (Next Up A, this repo's
  part of Atlas Z). `main-build.yml` added; `release.yml` moved from the `v*`
  tag push to the owner's dispatch. The branch run (a temporary push trigger,
  removed before review, because a dispatch needs the file on main), run
  44449548154 at `439a976`, read `0.1.0+main.439a976` back from Setup, the
  uninstall entry, the installed exe and the Settings line, with the install
  in `%LOCALAPPDATA%\Programs\Modlist Launcher`. The setup exe's binary file
  version is now the numeric part (`VersionInfoVersion`), its product version
  and the uninstall entry's DisplayVersion the full stamp. Still to come
  elsewhere: the row in the Atlas worker's app table (Atlas Z), which must
  keep `%LOCALAPPDATA%\Programs\Modlist Launcher`. The first real Reinstall
  is the owner's, from Atlas. The repository is public, so the artifact is
  downloadable by any signed-in GitHub user (GPL source; Atlas checks the
  digest).

- **2026-11-18: joined Tier A auto-merge** (owner). `.github/workflows/dependabot-auto-merge.yml` is the fleet template on `ubuntu-latest`, because the repository is public. Dependabot minor and patch PRs merge themselves through the jamarq-automerge App once the required checks pass; majors still wait for review.
- **2026-11-18: moved from `jmars319` to `JAMARQ-Digital-LLC`** (owner's decision, so Claude sessions on the PCs need one org-owned fine-grained token). Repository id, settings, `main` protection (Lint, test, and build; Dependency policy) and the `JAMARQ_RELEASES_TOKEN` secret carried over, and old URLs redirect. Clones on the Mac, JASON-PC and JASON-PC-2 now point at the org. The `repository` field, installer URLs and doc links name the org. Releases and update feeds were never here (they are in `JAMARQ-Digital-LLC/jamarq-releases`).
- Installed exe is the 25 Oct build of `99595b6` (#21); main at `b510def`
  (#26) differs only in dependencies, a test fix and workflows.
- After the 8 Nov account move all four Steam Play hooks were re-applied and
  point at `C:\Users\Jason\...\modlist-launcher.exe`; Steam Play worked 4 times
  through 12 Nov.
- No releases or tags; release and feed workflows have never run (checked
  18 Nov). A release now needs the owner to start `release.yml` and then
  publish its draft.
- The registry entry in agentic-instructions lists `ci.yml`,
  `advisory-sweep.yml` and `release.yml` but not `publish-feed.yml` (still so
  on 18 Nov); fix it with the next registry change for this repo.
- Watch: one copy left open four days kept running after its window closed
  (Oct); not reproduced.

## Temporary Logic Rules & Constraints

- No network access except the planned update check, which must use WinHTTP
  via the `windows` crate (deny.toml bans HTTP/TLS crates), be opt-out in
  Settings, download only on click, and be recorded in the decision log and
  `docs/safety-model.md`.
- No auto-update. New modlists are never added without the owner's OK.
- The `+main` rule, for stage 08's notice (test-ready): compare only the
  version before "+". An installed `X.Y.Z+main.<sha>` is not told about
  `X.Y.Z` or older, and is told about `X.Y.(Z+1)`. Cases: installed
  `0.1.0+main.abc1234` vs feed `0.1.0` -> no notice; vs `0.0.9` -> no
  notice; vs `0.1.1` -> notice; installed `0.1.0` vs `0.1.0` -> no notice.
- Official releases only when the owner asks (owner, 2026-11-18); main
  builds are the normal way to update JASON-PC. Never install by hand from a
  session.
- Steam's `localconfig.vdf` is written only while Steam is closed, always
  after a backup.
- `assets/icon.svg` and `scripts/generate-icon.ps1` change together; tell the
  Mac coordinating session the merge commit.
- pin-utils RUSTSEC-2026-0441 is ignored until 2027-02-08; fix together with
  Keeplight and Vaexil Mod Manager.
- chrono RUSTSEC-2026-0543 is ignored in `deny.toml` until 2027-02-17 (owner
  go 2026-11-17), cargo-deny only: Slint's i-slint-core needs it and no release
  drops it. Remove it when Slint does; same entry in Keeplight and Vaexil Mod Manager.

## Next Up

Order (owner, 2026-11-18: "go with current defaults"): B, C, D (A, the main
build, landed 18 Nov). B to D are the stages the owner ordered on 13 Nov (05,
06, 08). B stays a card, not started, until the owner says go.

### B. Stage 05: accessibility beyond the automation tree

Waits on the owner's go, and needs about 10 minutes of the owner listening to
live Narrator. Check Narrator on the cards, Play and Open MO2, the notices,
Settings and the dialogs; every flow keyboard-only, including Apply Steam Play,
Restore and End MO2; Windows contrast themes; display scales 100-200%, mixed-DPI
monitors and "Make text bigger"; and the known gap, a card scrolled out of view
missing from the UIA tree (23 Oct). Fix findings in this stage, each with a test
or smoke script, and record a checklist in `docs/`. No new dependencies;
`theme.slint` changes are agreed with Keeplight and Vaexil Mod Manager. Lead:
Slint's ScrollView already scrolls a focused control into view (VMM, 12 Nov).
Medium, 2 to 3 PRs.

### C. Stage 06: first GitHub Release

Unsigned, not announced (owner, 13 Nov). `release.yml` runs on the owner's
dispatch on main (since 18 Nov), tags main's head `v<version>` and makes the
draft. CI builds the zip and installer
into a draft; the owner publishes, and publishing copies the installer to
`jamarq-releases`. First real run of both workflows, so the first try may need
a fix. Decide whether it ships as 0.1.0. Small, after B.

### D. Stage 08: "An update is available"

A notice with a button that downloads the installer, checks its SHA-256 and
runs it; no auto-update. Default (owner leaning yes, 13 Nov): read
`modlist-launcher-latest/release.json` from the shared feed once a day through
WinHTTP, with a Settings switch to turn it off. The app's first network
access: record it in the decision log and `docs/safety-model.md`. Compare only
the version before "+" so `+main` builds are offered the next release (the
test-ready cases are under Temporary Logic Rules); VMM's
E8 self-updater (vaexil-mod-manager#35) is the model. Medium, after C.

## Context Compaction Checklist

- Keep the plan artifact line first; the URL does not change.
- Record open PRs, the installed build, and any owner decision here.
- Update this file when a stage lands or a rule changes.
