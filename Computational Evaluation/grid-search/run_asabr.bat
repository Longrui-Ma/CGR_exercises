@echo off
setlocal enabledelayedexpansion

:: powershell -Command "& {
::     .\run_asabr.bat 2>&1 |
::     Tee-Object -FilePath ('asabr_{0}.log' -f (Get-Date -Format 'yyyy-MM-dd_HH-mm-ss'))
:: }"

set "node_start=18"
set "node_end=30"
set "node_step=2"

set "hour_start=24"
set "hour_end=72"
set "hour_step=4"

:: seed values here only used as memo

if not exist "results" mkdir "results"

for /L %%n in (%node_start%,%node_step%,%node_end%) do (
    set "folder=nodes_%%n"
    if not exist "!folder!" (
        echo Warning: Folder not found: !folder!
    ) else (
        pushd "!folder!" >nul
        for /L %%h in (%hour_start%,%hour_step%,%hour_end%) do (
            set "input_file=02_ptvg_%%n_%%hh.json"
            if exist "!input_file!" (
                echo Running: ..\a_sabr "!input_file!" 4
                ..\a_sabr.exe "!input_file!" 4
            ) else (
                echo Warning: File not found: !input_file!
            )
        )
        popd >nul
    )
)

endlocal
