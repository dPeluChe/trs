@echo off
rem trs launcher for Windows — execs the native binary directly.
rem See bin/trs for the Unix equivalent.
setlocal

set "DIR=%~dp0"
set "BIN=%DIR%..\..\trs-cli-win32-x64\trs.exe"

if defined TRS_BINARY set "BIN=%TRS_BINARY%"

if not exist "%BIN%" (
    echo trs: binary not found at %BIN% 1>&2
    echo. 1>&2
    echo Expected platform package: @dpeluche/trs-cli-win32-x64 1>&2
    echo. 1>&2
    echo Alternatives: 1>&2
    echo   cargo install tars-cli    ^(build from source^) 1>&2
    exit /b 1
)

"%BIN%" %*
exit /b %ERRORLEVEL%
