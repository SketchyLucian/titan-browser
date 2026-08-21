[CmdletBinding()]
param(
    [string]$ExecutablePath = "target\release\titan-browser.exe",
    [string]$InstallerPath = "target\TitanBrowserInstaller.msi"
)

$ErrorActionPreference = "Stop"

function Resolve-SignTool {
    $configured = $env:TITAN_WINDOWS_SIGNTOOL
    if (-not [string]::IsNullOrWhiteSpace($configured)) {
        if (-not (Test-Path -LiteralPath $configured -PathType Leaf)) {
            throw "TITAN_WINDOWS_SIGNTOOL does not point to a file: $configured"
        }
        return (Resolve-Path -LiteralPath $configured).Path
    }

    $command = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($null -ne $command) {
        return $command.Source
    }

    $kitsRoot = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
    if (Test-Path -LiteralPath $kitsRoot -PathType Container) {
        $candidate = Get-ChildItem -LiteralPath $kitsRoot -Filter signtool.exe -File -Recurse |
            Where-Object { $_.DirectoryName -like "*\x64" } |
            Sort-Object FullName -Descending |
            Select-Object -First 1
        if ($null -ne $candidate) {
            return $candidate.FullName
        }
    }

    throw "SignTool was not found. Install the Windows SDK or set TITAN_WINDOWS_SIGNTOOL."
}

function Resolve-Artifact([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Release artifact does not exist: $Path"
    }
    return (Resolve-Path -LiteralPath $Path).Path
}

$rawThumbprint = [string]$env:TITAN_WINDOWS_CERT_SHA1
$thumbprint = ($rawThumbprint -replace "\s", "").ToUpperInvariant()
if ($thumbprint -notmatch "^[0-9A-F]{40}$") {
    throw "Set TITAN_WINDOWS_CERT_SHA1 to the 40-character SHA-1 thumbprint of the code-signing certificate."
}

$storeLocation = if ($env:TITAN_WINDOWS_CERT_STORE -eq "LocalMachine") {
    "LocalMachine"
} else {
    "CurrentUser"
}
$certificate = Get-Item -LiteralPath "Cert:\$storeLocation\My\$thumbprint" -ErrorAction Stop
$codeSigningOid = "1.3.6.1.5.5.7.3.3"
if (-not $certificate.HasPrivateKey) {
    throw "The selected certificate does not have an accessible private key."
}
if ($certificate.NotAfter -le (Get-Date)) {
    throw "The selected certificate has expired."
}
if ($certificate.EnhancedKeyUsageList.ObjectId.Value -notcontains $codeSigningOid) {
    throw "The selected certificate is not valid for code signing."
}

$signTool = Resolve-SignTool
$artifacts = @(
    Resolve-Artifact $ExecutablePath
    Resolve-Artifact $InstallerPath
)
$timestampUrl = if ([string]::IsNullOrWhiteSpace($env:TITAN_WINDOWS_TIMESTAMP_URL)) {
    "http://timestamp.digicert.com"
} else {
    $env:TITAN_WINDOWS_TIMESTAMP_URL
}
$parsedTimestamp = $null
if (-not [Uri]::TryCreate($timestampUrl, [UriKind]::Absolute, [ref]$parsedTimestamp) -or
    $parsedTimestamp.Scheme -notin @("http", "https")) {
    throw "TITAN_WINDOWS_TIMESTAMP_URL must be an absolute HTTP or HTTPS URL."
}

$signArguments = @(
    "sign",
    "/fd", "SHA256",
    "/td", "SHA256",
    "/tr", $timestampUrl,
    "/sha1", $thumbprint,
    "/s", "My"
)
if ($storeLocation -eq "LocalMachine") {
    $signArguments += "/sm"
}

foreach ($artifact in $artifacts) {
    & $signTool @signArguments $artifact
    if ($LASTEXITCODE -ne 0) {
        throw "SignTool failed for $artifact with exit code $LASTEXITCODE."
    }
    & $signTool verify /pa /all $artifact
    if ($LASTEXITCODE -ne 0) {
        throw "Signature verification failed for $artifact with exit code $LASTEXITCODE."
    }

    $signature = Get-AuthenticodeSignature -LiteralPath $artifact
    if ($signature.Status -ne "Valid") {
        throw "Authenticode verification returned $($signature.Status) for $artifact."
    }
    Write-Host "Signed and verified $artifact"
}
