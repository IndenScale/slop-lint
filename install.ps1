$ErrorActionPreference = "Stop"

$Repo = if ($env:SLOP_LINT_REPO) { $env:SLOP_LINT_REPO } else { "IndenScale/slop-lint" }
$Version = if ($env:SLOP_LINT_VERSION) { $env:SLOP_LINT_VERSION } else { "latest" }
$InstallDir = if ($env:SLOP_LINT_INSTALL_DIR) {
    $env:SLOP_LINT_INSTALL_DIR
} else {
    Join-Path $HOME ".local\bin"
}

$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { "x86_64" }
    "ARM64" { "aarch64" }
    default {
        throw "slop-lint: unsupported architecture: $env:PROCESSOR_ARCHITECTURE"
    }
}

$target = "$arch-pc-windows-msvc"
$asset = "slop-lint-$target.zip"

if ($Version -eq "latest") {
    $url = "https://github.com/$Repo/releases/latest/download/$asset"
} else {
    $url = "https://github.com/$Repo/releases/download/$Version/$asset"
}

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("slop-lint-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

try {
    $archive = Join-Path $tmp $asset
    Write-Host "slop-lint: downloading $url"
    Invoke-WebRequest -Uri $url -OutFile $archive
    Expand-Archive -Path $archive -DestinationPath $tmp -Force

    $exe = Join-Path $tmp "slop-lint.exe"
    if (!(Test-Path $exe)) {
        throw "slop-lint: archive did not contain slop-lint.exe"
    }

    Copy-Item -Path $exe -Destination (Join-Path $InstallDir "slop-lint.exe") -Force
    Write-Host "slop-lint: installed to $(Join-Path $InstallDir "slop-lint.exe")"

    $pathParts = [Environment]::GetEnvironmentVariable("Path", "User") -split ";"
    if ($pathParts -notcontains $InstallDir) {
        Write-Host "slop-lint: add $InstallDir to your user PATH if slop-lint is not found"
    }
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
