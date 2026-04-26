#!/usr/bin/env sh
set -e

REPO="Kodaskills/oplint"
BIN="oplint"
RELEASES="https://github.com/${REPO}/releases/latest/download"

# ── Detect OS ────────────────────────────────────────────────────────────────
os=$(uname -s)
case "$os" in
  Linux)  os_name="linux"  ;;
  Darwin) os_name="macos"  ;;
  *)
    echo "Unsupported OS: $os"
    echo "Download manually: https://github.com/${REPO}/releases/latest"
    exit 1
    ;;
esac

# ── Detect arch ──────────────────────────────────────────────────────────────
arch=$(uname -m)
case "$arch" in
  x86_64)          arch_name="x86_64"  ;;
  arm64 | aarch64) arch_name="aarch64" ;;
  *)
    echo "Unsupported architecture: $arch"
    echo "Download manually: https://github.com/${REPO}/releases/latest"
    exit 1
    ;;
esac

# ── Resolve install dir ───────────────────────────────────────────────────────
if [ -w /usr/local/bin ]; then
  install_dir="/usr/local/bin"
elif [ -d "$HOME/.local/bin" ]; then
  install_dir="$HOME/.local/bin"
else
  install_dir="$HOME/.local/bin"
  mkdir -p "$install_dir"
fi

# ── Download & install ────────────────────────────────────────────────────────
artifact="${BIN}-${os_name}-${arch_name}.tar.gz"
url="${RELEASES}/${artifact}"
tmp=$(mktemp -d)

echo "Downloading ${artifact}..."
if command -v curl > /dev/null 2>&1; then
  curl -fsSL "$url" -o "${tmp}/${artifact}"
elif command -v wget > /dev/null 2>&1; then
  wget -q "$url" -O "${tmp}/${artifact}"
else
  echo "curl or wget required"
  exit 1
fi

tar -xzf "${tmp}/${artifact}" -C "$tmp"
install -m 755 "${tmp}/${BIN}" "${install_dir}/${BIN}"
rm -rf "$tmp"

echo "Installed ${BIN} to ${install_dir}/${BIN}"

# ── PATH hint if needed ───────────────────────────────────────────────────────
case ":$PATH:" in
  *":${install_dir}:"*) ;;
  *)
    echo ""
    echo "Add ${install_dir} to your PATH:"
    echo "  export PATH=\"${install_dir}:\$PATH\""
    ;;
esac

echo ""
${install_dir}/${BIN} --version
