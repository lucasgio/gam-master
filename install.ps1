$ErrorActionPreference = "Stop"

$Repo = "giolabs/gam"
$Asset = "gam-windows-amd64.zip"
$Url = "https://github.com/$Repo/releases/latest/download/$Asset"
$InstallDir = "$env:USERPROFILE\.gam\bin"
$ZipPath = "$env:TEMP\$Asset"

Write-Host "Downloading gam from $Url..."
Invoke-WebRequest -Uri $Url -OutFile $ZipPath

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

Write-Host "Extracting to $InstallDir..."
Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force

$ExePath = "$InstallDir\gam.exe"
if (Test-Path $ExePath) {
    Write-Host "✅ gam installed successfully to $ExePath"
} else {
    Write-Host "❌ Installation failed. gam.exe not found."
    exit 1
}

# Check if in PATH
if ($env:Path -notlike "*$InstallDir*") {
    Write-Host "⚠️  $InstallDir is not in your PATH."
    Write-Host "Run the following command to add it permanently:"
    Write-Host "[System.Environment]::SetEnvironmentVariable('Path', [System.Environment]::GetEnvironmentVariable('Path', 'User') + ';$InstallDir', 'User')"
} else {
    Write-Host "✅ $InstallDir is already in your PATH."
    Write-Host "Try running 'gam --help'"
}

Remove-Item $ZipPath -Force
