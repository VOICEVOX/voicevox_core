::https://github.com/idanmiara/addlib/blob/main/src/addlib/makelib.bat Copyright (c) 2021 Idan Miara

@echo off

::https://stackoverflow.com/questions/9946322/how-to-generate-an-import-library-lib-file-from-a-dll
if %1x neq x goto step1
echo missing library name

goto exit
:step1
SET NAME=%~d1%~p1%~n1
if exist "%NAME%.dll" goto step2
echo file not found "%NAME%.dll"
goto exit

:step2
SET ARCH=x64

echo Creating LIB file from DLL file for %NAME%...
dumpbin /exports "%NAME%.dll"

echo creating "%NAME%.def"

echo LIBRARY %NAME% > "%NAME%.def"
echo EXPORTS >> "%NAME%.def"
for /f "skip=19 tokens=4" %%A in ('dumpbin /exports "%NAME%.dll"') do echo %%A >> "%NAME%.def"

echo creating "%NAME%.lib" from "%NAME%.def"
lib /def:"%NAME%.def" /out:"%NAME%.lib" /machine:%ARCH%

:exit
pause
