# Code signing script placeholder
#
# Prerequisites:
#   1. Obtain an EV Code Signing Certificate (e.g. from SSL.com, DigiCert)
#   2. Install the certificate on the build machine
#   3. Set environment variables:
#      $env:CODESIGN_CERT_THUMBPRINT = "<certificate thumbprint>"
#      $env:CODESIGN_TIMESTAMP_URL = "http://timestamp.digicert.com"
#
# For CI, store the certificate as a GitHub secret (base64-encoded PFX):
#   $env:CODESIGN_CERT_BASE64 = "${{ secrets.CODESIGN_CERT_PFX }}"
#   $env:CODESIGN_CERT_PASSWORD = "${{ secrets.CODESIGN_CERT_PASSWORD }}"
#
# Usage: .\scripts\sign.ps1 -Path "path\to\file.exe"

param(
    [Parameter(Mandatory=$true)]
    [string]$Path
)

$ErrorActionPreference = "Stop"

Write-Host "========================================"
Write-Host " Code Signing Placeholder"
Write-Host "========================================"
Write-Host "Target: $Path"

if (-not (Test-Path $Path)) {
    Write-Error "File not found: $Path"
    exit 1
}

# Check for certificate
if ($env:CODESIGN_CERT_BASE64) {
    Write-Host "[SIGN] Using certificate from environment variable"
    # Decode PFX from base64
    $pfxBytes = [Convert]::FromBase64String($env:CODESIGN_CERT_BASE64)
    $pfxPath = Join-Path $env:TEMP "codesign_cert.pfx"
    [IO.File]::WriteAllBytes($pfxPath, $pfxBytes)

    # Import certificate
    $cert = Import-PfxCertificate -FilePath $pfxPath -CertStoreLocation Cert:\CurrentUser\My -Password (ConvertTo-SecureString $env:CODESIGN_CERT_PASSWORD -AsPlainText -Force)

    # Sign with signtool
    $timestampUrl = if ($env:CODESIGN_TIMESTAMP_URL) { $env:CODESIGN_TIMESTAMP_URL } else { "http://timestamp.digicert.com" }
    & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe" sign `
        /fd SHA256 `
        /sha1 $cert.Thumbprint `
        /tr $timestampUrl `
        /td SHA256 `
        /v `
        $Path

    Write-Host "[SIGN] Successfully signed: $Path"
} elseif ($env:CODESIGN_CERT_THUMBPRINT) {
    Write-Host "[SIGN] Using certificate from Windows certificate store"
    $timestampUrl = if ($env:CODESIGN_TIMESTAMP_URL) { $env:CODESIGN_TIMESTAMP_URL } else { "http://timestamp.digicert.com" }
    & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe" sign `
        /fd SHA256 `
        /sha1 $env:CODESIGN_CERT_THUMBPRINT `
        /tr $timestampUrl `
        /td SHA256 `
        /v `
        $Path

    Write-Host "[SIGN] Successfully signed: $Path"
} else {
    Write-Host "[SIGN] SKIPPED — No signing certificate configured."
    Write-Host "[SIGN] Set CODESIGN_CERT_BASE64 or CODESIGN_CERT_THUMBPRINT to enable."
    Write-Host "[SIGN] For MVP/V1.0: unsigned binary will show SmartScreen warning."
}

Write-Host "========================================"
Write-Host " Done"
Write-Host "========================================"
