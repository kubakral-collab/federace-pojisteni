param(
    [string]$SourceDatabase = (Join-Path $PSScriptRoot "..\dd.sqlite"),
    [switch]$ResetSmokeCopy
)

$ErrorActionPreference = "Stop"
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$sourcePath = [System.IO.Path]::GetFullPath($SourceDatabase)
$smokeDirectory = Join-Path $repositoryRoot "smoke-data"
$smokePath = Join-Path $smokeDirectory "dd-smoke.sqlite"

if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
    throw "Zdrojová kopie databáze neexistuje: $sourcePath"
}
if ($sourcePath -eq $smokePath) {
    throw "Zdrojová a smoke databáze musí být různé soubory."
}

New-Item -ItemType Directory -Path $smokeDirectory -Force | Out-Null
if ($ResetSmokeCopy -or -not (Test-Path -LiteralPath $smokePath -PathType Leaf)) {
    Copy-Item -LiteralPath $sourcePath -Destination $smokePath -Force
}

$sourceHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash
$env:FEDERACE_DB_PATH = [System.IO.Path]::GetFullPath($smokePath)
Write-Host "SMOKE DATABASE (absolute): $env:FEDERACE_DB_PATH"
Write-Host "Zdroj zůstává pouze pro čtení: $sourcePath"

Push-Location $repositoryRoot
try {
    & npm.cmd run smoke
    if ($LASTEXITCODE -ne 0) { throw "Smoke build skončil s kódem $LASTEXITCODE." }
}
finally {
    Pop-Location
    Remove-Item Env:FEDERACE_DB_PATH -ErrorAction SilentlyContinue
    $sourceHashAfter = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash
    if ($sourceHashAfter -ne $sourceHash) {
        throw "BEZPEČNOSTNÍ CHYBA: zdrojová databáze se během smoke testu změnila."
    }
}
