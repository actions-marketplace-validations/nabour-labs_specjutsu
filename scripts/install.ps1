# Install sj, the specjutsu CLI, on Windows (DIST-006, DIST-007).
#
#   powershell -ExecutionPolicy ByPass -c "irm https://nabour-labs.github.io/specjutsu/install.ps1 | iex"
#
# Environment:
#   SJ_VERSION         release to install, e.g. 0.4.0 (default: the latest release)
#   SJ_INSTALL_DIR     folder for sj.exe (default: %LOCALAPPDATA%\Programs\sj)
#   SJ_NO_MODIFY_PATH  1 to leave the user PATH alone and print the folder to add instead
#   SJ_TARGET          release target (default: detected, x64 or arm64)
#   SJ_DOWNLOAD_BASE   where releases live (default: https://github.com/nabour-labs/specjutsu/releases)
#
# The zip is checked against the release's SHA256SUMS before anything is installed.
# Works in Windows PowerShell 5.1 and PowerShell 7.

function Install-Sj {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'

    $base = $env:SJ_DOWNLOAD_BASE
    if (-not $base) { $base = 'https://github.com/nabour-labs/specjutsu/releases' }
    $dest = $env:SJ_INSTALL_DIR
    if (-not $dest) { $dest = Join-Path $env:LOCALAPPDATA 'Programs\sj' }

    $target = $env:SJ_TARGET
    if (-not $target) {
        $arch = $env:PROCESSOR_ARCHITEW6432
        if (-not $arch) { $arch = $env:PROCESSOR_ARCHITECTURE }
        if ($arch -eq 'ARM64') { $target = 'aarch64-pc-windows-msvc' }
        else { $target = 'x86_64-pc-windows-msvc' }
    }

    if ($env:SJ_VERSION) {
        $from = "$base/download/v$($env:SJ_VERSION.TrimStart('v'))"
    } else {
        $from = "$base/latest/download"
    }
    $asset = "sj-$target.zip"

    $tmp = Join-Path ([IO.Path]::GetTempPath()) ("sj-install-" + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmp | Out-Null
    try {
        $web = $base -match '^https?://'
        if ($web) {
            # Windows PowerShell 5.1 defaults to TLS 1.0, which GitHub refuses.
            [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        }
        Write-Host "downloading $asset from $from"
        foreach ($name in @($asset, 'SHA256SUMS')) {
            $out = Join-Path $tmp $name
            try {
                if ($web) { Invoke-WebRequest -UseBasicParsing -Uri "$from/$name" -OutFile $out }
                else { Copy-Item -LiteralPath (Join-Path $from $name) -Destination $out }
            } catch {
                [Console]::Error.WriteLine("sj install: could not download $from/$name")
                return 1
            }
        }

        $want = $null
        foreach ($line in Get-Content (Join-Path $tmp 'SHA256SUMS')) {
            $parts = $line -split '\s+'
            if ($parts.Count -ge 2 -and ($parts[1] -eq $asset -or $parts[1] -eq "*$asset")) { $want = $parts[0] }
        }
        $have = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $tmp $asset)).Hash
        if (-not $want -or $want.ToLower() -ne $have.ToLower()) {
            [Console]::Error.WriteLine("sj install: checksum mismatch for $asset")
            return 1
        }

        Expand-Archive -LiteralPath (Join-Path $tmp $asset) -DestinationPath $tmp -Force
        New-Item -ItemType Directory -Path $dest -Force | Out-Null
        $exe = Join-Path $dest 'sj.exe'
        Copy-Item -LiteralPath (Join-Path $tmp "sj-$target\sj.exe") -Destination $exe -Force
        Write-Host "installed sj to $exe"

        $onPath = ($env:Path -split ';') -contains $dest
        if (-not $onPath) {
            if ($env:SJ_NO_MODIFY_PATH -eq '1') {
                Write-Host "add $dest to your PATH to run sj"
            } else {
                $user = [Environment]::GetEnvironmentVariable('Path', 'User')
                if (-not $user) { $user = '' }
                if (-not (($user -split ';') -contains $dest)) {
                    [Environment]::SetEnvironmentVariable('Path', ($user.TrimEnd(';') + ";$dest").TrimStart(';'), 'User')
                }
                $env:Path = "$env:Path;$dest"
                Write-Host "added $dest to your user PATH; open a new terminal to use sj everywhere"
            }
        }
        return 0
    } finally {
        Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

$code = Install-Sj
# Run as a file, report the result as the exit code. Run through `irm | iex`, never call
# exit: it would close the user's terminal.
if ($PSCommandPath) { exit $code }
elseif ($code -ne 0) { Write-Error 'sj install failed' }
