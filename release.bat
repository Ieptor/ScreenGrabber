@echo off

set "cartelle=overlay_process background_listener edit_gui gui_sg"

for %%d in (%cartelle%) do (
    echo Entrando nella cartella %%d
    cd %%d
    
    echo Eseguo cargo clean
    cargo clean
    echo Eseguo cargo build --release
    cargo build --release
    
    cd ..
)

echo Operazioni completate.
