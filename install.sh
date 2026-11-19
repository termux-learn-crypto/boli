#!/bin/sh
# BOLI installer - Termux aur Linux dono me kaam karta hai
#
# Use: ./install.sh
# Custom dir ke liye: BOLI_INSTALL_DIR=/path/to/bin ./install.sh

set -eu

# Src se binary dhundo (pehle release build banaya ho)
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BIN_SRC="$SCRIPT_DIR/target/release/boli"

echo "=== BOLI Installer ==="

if [ ! -x "$BIN_SRC" ]; then
    echo "Galti: $BIN_SRC nahi mila."
    echo "Pehle release build banao:  cargo build --release"
    exit 1
fi

# Install directory chunno
if [ -n "${BOLI_INSTALL_DIR:-}" ]; then
    # User ne khud dir diya
    INSTALL_DIR="$BOLI_INSTALL_DIR"
elif [ -n "${PREFIX:-}" ] && [ -d "$PREFIX/bin" ] && [ -w "$PREFIX/bin" ]; then
    # Termux (ya koi bhi environment jisme PREFIX/bin writable ho)
    INSTALL_DIR="$PREFIX/bin"
elif [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
    # Linux system-wide
    INSTALL_DIR=/usr/local/bin
elif [ -n "${HOME:-}" ]; then
    # User-local fallback
    INSTALL_DIR="$HOME/.local/bin"
else
    echo "Galti: install karne ke liye koi directory nahi mili."
    echo "       BOLI_INSTALL_DIR set karo, ya PREFIX/bin, /usr/local/bin, HOME available karo."
    exit 1
fi

mkdir -p "$INSTALL_DIR"
cp "$BIN_SRC" "$INSTALL_DIR/boli"
chmod 755 "$INSTALL_DIR/boli"

echo "boli install ho gaya: $INSTALL_DIR/boli"

# Kya dir PATH me hai?
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo "Note: $INSTALL_DIR tumhare PATH me nahi hai."
        echo "      PATH me jodo:  export PATH=\"$INSTALL_DIR:\$PATH\""
        ;;
esac

echo "Ab bina cargo ke chal sakte ho:"
echo "  boli run examples/namaste.hl"
echo "  boli"
