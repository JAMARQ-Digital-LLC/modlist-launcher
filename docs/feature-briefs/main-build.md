# Modlist Launcher: Windows builds for Atlas's Reinstall (Feature session brief)

Status: carded 2026-11-18 as Next Up item A in `ml-project-state.md`; built 2026-11-18 on `feat/windows-builds-for-atlas-s-reinstall` and removed from Next Up there. Fleet: main-build, with Vaexbot C, Studio AE, Pulse B, Keeplight A and Vaexil Mod Manager A.

**Title line:**

```
Modlist Launcher: main-build.yml for Atlas's Reinstall on JASON-PC
```

**Paste as the first message:**

```
Add Modlist Launcher's part of Atlas's "Reinstall on JASON-PC" (Atlas Next Up Z, SSH design phase 6, https://claude.ai/artifact/RZCV7mQTtCHf4nGnwbQt2J; Atlas brief docs/feature-briefs/reinstall-on-jason-pc.md, step 2), and make releases go out only when the owner asks. Read ml-project-state.md and that brief first. Keeplight A and Vaexil Mod Manager A are the same task in the same Rust, Slint and Inno Setup shape.

Facts, checked 2026-11-18:
- The owner's goal (2026-11-18): cut how often releases go out, while still being able to update every Windows app from main. The owner's decision the same day: once an app has its main build, an official release goes out only when the owner asks for one (replacing the nightly rule of ADR-0026 as updated in agentic-instructions#459), and main builds become the normal way to update JASON-PC.
- release.yml today: on `push: tags: ['v*']`, on windows-latest, dtolnay/rust-toolchain@stable, Inno Setup from choco, `cargo test --workspace --locked`, `./scripts/package.ps1`, then a draft GitHub release. No schedule. publish-feed.yml runs on `release: published`, refuses a tag that isn't v<MAJOR.MINOR.PATCH>, and copies `ModlistLauncher-<version>-setup.exe` to modlist-launcher-v<version> with modlist-launcher-latest/release.json in jamarq-releases. Neither has ever run: there are no tags or releases, and the feed does not exist yet (checked 2026-11-18).
- scripts/package.ps1 reads the version from `cargo metadata` (workspace version 0.1.0 on main 3f78e73) and builds with `--locked`; packaging/installer.iss is per user, `OutputBaseFilename=ModlistLauncher-{#AppVersion}-setup`, installing to %LOCALAPPDATA%\Programs\Modlist Launcher.
- JASON-PC had 0.1.0 on 2026-11-17 (Atlas's design): the 25 Oct local build of 99595b6. The four Steam Play hooks point at the installed modlist-launcher.exe, so a reinstall must keep the same AppId and install folder.
- The app has no update check (docs/safety-model.md: "No telemetry, accounts or update checks"), so nothing in it can offer a downgrade or miss a release. Stage 08 plans a notice from modlist-launcher-latest/release.json; it must compare only the part before "+", and this item writes that rule into stage 08's notes rather than building the notice.
- JAMARQ-Digital-LLC/modlist-launcher is a public repository, so a workflow artifact is downloadable by any signed-in GitHub user, not private as the Atlas design says. The source is public under GPL-3.0, so the exposure is small; say so in the PR. Standard GitHub-hosted runners are free for public repositories.

Build:
1. .github/workflows/main-build.yml: workflow_dispatch only, on main. The same build as the release, through the same package.ps1 (no forked copy), with the workspace version stamped `<version>+main.<sha7>` before packaging. `--locked` will refuse a changed workspace version until Cargo.lock matches, so refresh only the workspace entries offline (for example `cargo update --workspace --offline`) or pass the stamp another way; never unlock the dependencies. Check what Inno Setup writes for the file version and DisplayVersion with a "+" in AppVersion. Upload the setup exe and SHA256SUMS.txt as an artifact kept 7 days, named so Atlas can find it by commit. No GitHub release and no feed.
2. release.yml: move it from the tag push to workflow_dispatch, which the owner starts; it tags main's head as v<workspace version> and makes the draft, refusing a version that already has a tag. publish-feed.yml stays on the owner publishing the draft. Rewrite both header comments to say releases happen only when the owner asks. The first real release (stage 06, v0.1.0) stays the owner's call.
3. The +main rule, for when stage 08 adds a notice: a test-ready note in ml-project-state.md that an installed X.Y.Z+main.<sha> must not be told about X.Y.Z or older and must be told about X.Y.(Z+1).
4. Read the stamped version back from the built installer and from the window's version line.

Done when: no schedule or push trigger is left in release.yml, the feed can move only after the owner's dispatch and publish, a dispatch of main-build.yml on your branch produces the artifact with the stamped version read back, and the PR is open. Never install on a real PC; the first real Reinstall is the owner's, from Atlas, which needs a row in the worker's app table (Atlas Z) and must not change the install folder the Steam hooks use.

Follow the repo's verify (CONTRIBUTING.md: cargo fmt, clippy with -D warnings, tests, the release build and cargo deny, plus actionlint on the workflows), in one PR from your own worktree. Don't install on any machine by hand. Remove item A from `## Next Up` in ml-project-state.md and update the plan page https://claude.ai/artifact/2qaZ63YM974LMKSFwgbQZD before finishing.
```
