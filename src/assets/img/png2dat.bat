@echo off
setlocal



for %%F in ("%~dp0png\*.png") do (
    w4 png2src --rust "%%F" --template image.mustache > "%~dp0%%~nF.rs"
)

pause

endlocal
