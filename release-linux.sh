#!/bin/bash

cartelle="overlay_process background_listener edit_gui gui_sg"

for d in $cartelle; do
    echo "Entering folder $d"
    cd "$d" || exit 1

    echo "Running cargo clean"
    cargo clean
    echo "Running cargo build --release"
    cargo build --release

    cd ..
done

echo "Operations completed."
