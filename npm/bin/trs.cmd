@echo off
rem trs launcher for Windows — execs the native binary directly.
rem See bin/trs for the Unix equivalent.
setlocal

set "DIR=%~dp0"
set "PKG=trs-cli-win32-x64"
set "BIN="

if defined TRS_BINARY (
    if exist "%TRS_BINARY%" set "BIN=%TRS_BINARY%"
)

rem Candidate locations npm may use for the optional dependency.
if not defined BIN if exist "%DIR%..\..\%PKG%\trs.exe" set "BIN=%DIR%..\..\%PKG%\trs.exe"
if not defined BIN if exist "%DIR%..\node_modules\@dpeluche\%PKG%\trs.exe" set "BIN=%DIR%..\node_modules\@dpeluche\%PKG%\trs.exe"
if not defined BIN if exist "%DIR%..\..\..\@dpeluche\%PKG%\trs.exe" set "BIN=%DIR%..\..\..\@dpeluche\%PKG%\trs.exe"

if not defined BIN (
    echo trs: platform binary not found 1>&2
    echo. 1>&2
    echo Expected package: @dpeluche/%PKG% 1>&2
    echo Searched near:    %DIR% 1>&2
    echo. 1>&2
    echo Alternatives: 1>&2
    echo   cargo install tars-cli    ^(build from source^) 1>&2
    exit /b 1
)

"%BIN%" %*
exit /b %ERRORLEVEL%
