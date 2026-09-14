<#
    .SYNOPSIS
    Builds one language technology's contract module as a shared library,
    builds the probe against the same source, and runs it.

    .DESCRIPTION
    The language technologies of xmip-core-contract — c, cpp, java and
    python — differ in what they compile and how it links, not in what the
    probe asks of a module, so the probe lives here once and each
    technology's verify.ps1 calls this with its own source (ADR-0044). Exit 0
    when the module builds and the probe passes, non-zero otherwise; the
    technology's build directory receives the artifacts. One compiler
    everywhere, zig, declared in prerequisite.toml as `c`. The ABI header
    comes from xmip-core-abi, found in the estate when the capability is
    mounted there and through XMIP_ABI_INCLUDE otherwise.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string] $Directory,

    [Parameter(Mandatory = $true)]
    [ValidateSet('cc', 'c++')]
    [string] $Compiler,

    [Parameter(Mandatory = $true)]
    [string[]] $Source,

    [Parameter(Mandatory = $true)]
    [string] $Standard,

    [Parameter()]
    [string[]] $Include = @(),

    [Parameter()]
    [string[]] $Link = @()
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $Directory

if (-not (Get-Command zig -ErrorAction SilentlyContinue)) {
    Write-Host 'FAILED. zig is not installed; prerequisite.toml declares it as c.'
    exit 2
}

[string] $abi = $env:XMIP_ABI_INCLUDE
if ([string]::IsNullOrWhiteSpace($abi)) {
    $abi = Join-Path $PSScriptRoot '..' '..' '..' 'foundation' 'abi' 'include'
}
if (-not (Test-Path -LiteralPath (Join-Path $abi 'xmip_module.h'))) {
    Write-Host "FAILED. xmip_module.h not found under $abi; set XMIP_ABI_INCLUDE."
    exit 2
}

New-Item -ItemType Directory -Force -Path build | Out-Null

[string] $stem = "xmip_core_contract_$Standard"
[string] $library = ''
[string] $probe = ''
if ($IsWindows) {
    $library = "$stem.dll"
    $probe = 'probe.exe'
}
elseif ($IsMacOS) {
    $library = "lib$stem.dylib"
    $probe = 'probe'
}
else {
    $library = "lib$stem.so"
    $probe = 'probe'
}

[string[]] $includes = @('-I', $abi)
foreach ($extra in $Include) {
    $includes += @('-I', $extra)
}
[string[]] $warnings = @('-Wall', '-Wextra', '-Werror')

Write-Host "   zig $Compiler -shared -> build/$library"
[string[]] $shared = @('-shared', '-O2', '-fvisibility=hidden') + $warnings + $includes
$shared += $Source + @('-o', (Join-Path build $library)) + $Link
& zig $Compiler @shared
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

# The probe is one C file; a C++ module compiles it as C++ so the two link as
# one program, and the probe declares the entrypoint extern "C" for that.
[string[]] $language = @()
if ($Compiler -eq 'c++') {
    $language = @('-x', 'c++')
}
[string] $probeSource = Join-Path $PSScriptRoot 'probe.c'

Write-Host "   zig $Compiler -> build/$probe"
[string[]] $arguments = @('-O1') + $warnings + $includes + @("-DXMIP_STANDARD=$Standard")
$arguments += $Source + $language + @($probeSource, '-o', (Join-Path build $probe)) + $Link
& zig $Compiler @arguments
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

& (Join-Path $Directory 'build' $probe)
exit $LASTEXITCODE
