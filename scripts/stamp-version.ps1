# Stamps build metadata onto the workspace version for a main build, so the
# exe, the window's version line and the installer all read
# <version>+<Build>, for example 0.1.0+main.abc1234. Used by
# .github/workflows/main-build.yml before scripts/package.ps1; never commit
# the result.
#
# Only the two workspace entries in Cargo.lock change: the registry index is
# fetched first, then `cargo update --workspace --offline` rewrites the
# workspace versions without touching any dependency. Fails if anything else
# in Cargo.lock moved.
#
#   ./scripts/stamp-version.ps1 -Build main.abc1234
param([Parameter(Mandatory)] [string] $Build)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($Build -notmatch '^[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*$') { throw "'$Build' is not valid semver build metadata" }
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
Push-Location $root
try {
    if (git status --porcelain -- Cargo.toml Cargo.lock) { throw 'Cargo.toml or Cargo.lock already has changes' }

    cargo fetch --locked
    if ($LASTEXITCODE -ne 0) { throw 'cargo fetch failed' }

    # The workspace version is the only line that starts with `version = `.
    $manifest = Get-Content Cargo.toml -Raw
    $pattern = '(?m)^version = "(\d+\.\d+\.\d+)"\r?$'
    $found = [regex]::Matches($manifest, $pattern)
    if ($found.Count -ne 1) { throw "expected one plain X.Y.Z workspace version in Cargo.toml, found $($found.Count)" }
    $version = "$($found[0].Groups[1].Value)+$Build"
    $manifest = [regex]::Replace($manifest, $pattern, "version = `"$version`"")
    [IO.File]::WriteAllText((Join-Path $root 'Cargo.toml'), $manifest)

    cargo update --workspace --offline
    if ($LASTEXITCODE -ne 0) { throw 'cargo update --workspace --offline failed' }

    # Exactly the two workspace version lines, nothing else.
    $lock = git diff --numstat -- Cargo.lock
    if ($lock -notmatch '^2\s+2\s+Cargo\.lock$') { throw "Cargo.lock changed beyond the workspace versions: $lock" }
    $changed = git diff -U0 -- Cargo.lock | Where-Object { $_ -match '^[+-]version' }
    if (@($changed | Where-Object { $_ -match '^\+' -and $_ -ne "+version = `"$version`"" }).Count) {
        throw "unexpected Cargo.lock change: $($changed -join '; ')"
    }

    $read = (cargo metadata --no-deps --format-version 1 --locked | ConvertFrom-Json).packages |
        Where-Object name -eq 'modlist-launcher' | Select-Object -ExpandProperty version
    if ($read -ne $version) { throw "cargo metadata reads $read, expected $version" }
    $version
} finally {
    Pop-Location
}
