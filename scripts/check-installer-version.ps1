# Reads the version back from a built installer: Setup's own version info,
# then a silent per-user install the way Atlas's Reinstall runs it, then the
# uninstall entry, the installed exe and the version line in the window's
# Settings sheet. Exits 1 if any of them is not -Expect, or if the install
# lands anywhere but %LOCALAPPDATA%\Programs\Modlist Launcher (the folder
# Steam's Play hooks point at).
#
# It installs, so it runs only on a GitHub-hosted runner, which is thrown
# away after the job. Never run it on a real PC.
#
#   ./scripts/check-installer-version.ps1 -Installer dist\ModlistLauncher-<v>-setup.exe -Expect <v>
param(
    [Parameter(Mandatory)] [string] $Installer,
    [Parameter(Mandatory)] [string] $Expect
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
    throw 'This script installs the app; it runs only on a GitHub-hosted runner.'
}
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]

$failed = $false
function Check($label, $got) {
    $ok = $got -eq $Expect
    if (-not $ok) { $script:failed = $true }
    '{0,-34} {1,-34} {2}' -f $label, $got, $(if ($ok) { 'ok' } else { "FAIL (expected $Expect)" })
}

$setup = Get-Item $Installer
Check 'Setup ProductVersion' $setup.VersionInfo.ProductVersion
'{0,-34} {1}' -f 'Setup FileVersion', $setup.VersionInfo.FileVersion

# The same flags as Atlas's worker task on JASON-PC.
$p = Start-Process $setup.FullName -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/CURRENTUSER' -PassThru -Wait
if ($p.ExitCode -ne 0) { throw "Setup exited with $($p.ExitCode)" }

$folder = Join-Path $env:LOCALAPPDATA 'Programs\Modlist Launcher'
$exe = Join-Path $folder 'modlist-launcher.exe'
if (-not (Test-Path $exe)) { throw "not installed at $exe" }
$key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{6B1E2B53-7C1E-4E0F-9A35-2C7B8F0D4A61}_is1'
$entry = Get-ItemProperty $key
Check 'Uninstall entry DisplayVersion' $entry.DisplayVersion
$location = $entry.InstallLocation.TrimEnd('\')
if ($location -ne $folder) { $failed = $true }
'{0,-34} {1} {2}' -f 'Uninstall entry InstallLocation', $location, $(if ($location -eq $folder) { 'ok' } else { "FAIL (expected $folder)" })
Check 'Installed exe ProductVersion' (Get-Item $exe).VersionInfo.ProductVersion

# The window: Settings shows "Modlist Launcher <version> · GPL-3.0 · ...".
$data = Join-Path ([IO.Path]::GetTempPath()) "modlist-version-$PID"
New-Item -ItemType Directory -Force $data | Out-Null
$psi = [System.Diagnostics.ProcessStartInfo]::new($exe)
$psi.Environment['MODLIST_LAUNCHER_DATA'] = $data
$app = [System.Diagnostics.Process]::Start($psi)
try {
    $h = [IntPtr]::Zero
    for ($i = 0; $i -lt 50 -and $h -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 200; $app.Refresh(); $h = $app.MainWindowHandle }
    if ($h -eq [IntPtr]::Zero) { throw 'the launcher window did not appear' }
    Start-Sleep -Milliseconds 1500
    $byPid = New-Object System.Windows.Automation.PropertyCondition ($A::ProcessIdProperty), $app.Id
    # The automation tree can come back empty on the first query.
    function Find([scriptblock] $match) {
        for ($k = 0; $k -lt 20; $k++) {
            $r = $A::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $byPid)
            if ($r) {
                $e = @($r.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) |
                    Where-Object $match | Select-Object -First 1
                if ($e) { return $e }
            }
            Start-Sleep -Milliseconds 500
        }
        $null
    }
    $settings = Find { $_.Current.Name -eq 'Settings' }
    if (-not $settings) { throw 'no Settings button' }
    $settings.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    $line = Find { $_.Current.Name -like 'Modlist Launcher * GPL-3.0*' }
    if (-not $line) { throw 'no version line in the Settings sheet' }
    Check 'Window version line' ($line.Current.Name -replace '^Modlist Launcher (\S+) .*$', '$1')
} finally {
    if (-not $app.HasExited) { $app.Kill(); $app.WaitForExit() }
}
if ($failed) { exit 1 }
exit 0
