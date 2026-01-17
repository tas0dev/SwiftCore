#!/bin/sh

REQUIRED_CMDS="qemu-system-x86_64 cargo rustc"
MISSING=0

echo "　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　
⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⢀⡀⡀⡀⡀⢀⣤⣶⣶⣶⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀
⡀⢀⣴⣾⣿⣿⣷⣦⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⢿⡿⡀⡀⡀⣿⡟⡀⡀⠈⡀⣶⣿⡀⡀⡀⡀⡀⡀⡀⣠⣶⣿⣿⣿⣷⣦⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀
⡀⣿⡟⡀⡀⡀⡀⠉⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⢸⣿⡀⡀⡀⡀⡀⣿⣿⡀⡀⡀⡀⡀⡀⣾⡿⠉⡀⡀⡀⡀⠉⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀
⡀⣿⣧⡀⡀⡀⡀⡀⡀⠈⣿⡇⡀⡀⡀⣿⣿⡀⡀⡀⢠⣿⠃⡀⣿⣿⡀⡀⢸⣿⠿⠿⠿⠿⡀⣿⣿⠿⠿⠿⡀⡀⣾⣿⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⣴⣿⠿⠿⣿⣦⡀⡀⡀⢰⣿⡿⠿⠿⡟⡀⡀⣴⣿⠿⠿⣿⣦⡀
⡀⠈⠿⣿⣷⣦⣀⡀⡀⡀⢿⣿⡀⡀⢀⣿⣿⡆⡀⡀⣾⣿⡀⡀⣿⣿⡀⡀⢸⣿⡀⡀⡀⡀⡀⣿⣿⡀⡀⡀⡀⡀⣿⣿⡀⡀⡀⡀⡀⡀⡀⡀⡀⣾⡿⡀⡀⡀⡀⢿⣿⡀⡀⢸⣿⡀⡀⡀⡀⡀⣿⡿⡀⡀⡀⠈⣿⡇
⡀⡀⡀⡀⠈⠛⣿⣿⡀⡀⠘⣿⡄⡀⣾⡟⢹⣿⡀⢀⣿⠃⡀⡀⣿⣿⡀⡀⢸⣿⡀⡀⡀⡀⡀⣿⣿⡀⡀⡀⡀⡀⣿⣿⡀⡀⡀⡀⡀⡀⡀⡀⡀⣿⡇⡀⡀⡀⡀⢸⣿⡄⡀⢸⣿⡀⡀⡀⡀⡀⣿⣿⣿⣿⣿⣿⣿⣿
⡀⡀⡀⡀⡀⡀⡀⣿⡧⡀⡀⢿⣿⢀⣿⡀⡀⣿⡄⣾⡿⡀⡀⡀⣿⣿⡀⡀⢸⣿⡀⡀⡀⡀⡀⣿⣿⡀⡀⡀⡀⡀⢻⣿⡄⡀⡀⡀⡀⡀⡀⡀⡀⣿⣇⡀⡀⡀⡀⣸⣿⡀⡀⢸⣿⡀⡀⡀⡀⡀⣿⣧⡀⡀⡀⡀⡀⡀
⢀⣦⣄⣀⣀⣠⣾⣿⠃⡀⡀⡀⣿⣿⡏⡀⡀⢹⣿⣿⠁⡀⡀⡀⣿⣿⡀⡀⢸⣿⡀⡀⡀⡀⡀⠹⣿⣄⣀⣠⡄⡀⡀⠻⣿⣶⣄⣀⣀⣠⣴⡀⡀⠘⣿⣦⣀⣀⣴⣿⠋⡀⡀⢸⣿⡀⡀⡀⡀⡀⠙⣿⣦⣀⣀⣀⣤⡀
⡀⠉⠙⠛⠛⠛⠉⡀⡀⡀⡀⡀⠙⠛⡀⡀⡀⡀⠛⠋⡀⡀⡀⡀⠛⠛⡀⡀⠘⠛⡀⡀⡀⡀⡀⡀⠉⠛⠛⠋⠁⡀⡀⡀⡀⠉⠛⠛⠛⠋⠉⡀⡀⡀⡀⠉⠛⠛⠉⡀⡀⡀⡀⠘⠛⡀⡀⡀⡀⡀⡀⡀⠉⠛⠛⠛⠉⡀
⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀
⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀⡀"　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　

# Auto install dependencies
if [ "$1" = "install" ]; then
    echo "Installing dependencies..."

    if command -v apt >/dev/null 2>&1; then
        PKG_MANAGER="apt"
        UPDATE_CMD="sudo apt update"
        INSTALL_CMD="sudo apt install -y qemu-system llvm ovmf"
    elif command -v dnf >/dev/null 2>&1; then
        PKG_MANAGER="dnf"
        UPDATE_CMD="sudo dnf update"
        INSTALL_CMD="sudo dnf install -y qemu-system llvm ovmf"
    else
        echo "No supported package manager found (apt or dnf). Please install dependencies manually."
        exit 1
    fi

    echo "Using $PKG_MANAGER to install system dependencies..."
    $UPDATE_CMD
    $INSTALL_CMD

    echo "\nThe following tools should be installed via rustup:"
    echo "  - cargo"
    echo "  - rustc"
    echo "Please run: curl https://sh.rustup.rs -sSf | sh"
    exit 0
fi

echo "Checking required dependencies..."
for cmd in $REQUIRED_CMDS; do
	if ! command -v "$cmd" >/dev/null 2>&1; then
		echo "\033[31m[ MISSING ]\033[0m $cmd"
		MISSING=1
        MISSING_LIST="$MISSING_LIST $cmd"
	else
		echo "\033[32m[  FOUND  ]\033[0m $cmd"
	fi
done

# Check for OVMF.fd
if [ -f /usr/share/ovmf/OVMF.fd ]; then
    echo "\033[32m[  FOUND  ]\033[0m /usr/share/ovmf/OVMF.fd"
else
    echo "\033[31m[ MISSING ]\033[0m /usr/share/ovmf/OVMF.fd"
    MISSING=1
    MISSING_LIST="$MISSING_LIST　OVMF"
fi

if [ $MISSING -eq 1 ]; then
	echo "\nSome dependencies are missing. Please install them before proceeding."
    echo "Missing dependencies:$MISSING_LIST"
	echo "Auto install: ./requirements.sh install"
	exit 1
else
	echo "\nAll dependencies are installed. :D"
	exit 0
fi