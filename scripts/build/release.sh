#!/usr/bin/env bash
# scripts/release.sh — hardened release builder for count_lines
# Portable, reproducible-ish, and target-aware.

set -Eeuo pipefail
IFS=$'\n\t'

# -------- Pretty printers --------
RED=$'\033[0;31m'; GREEN=$'\033[0;32m'; YELLOW=$'\033[1;33m'; BLUE=$'\033[0;34m'; CYAN=$'\033[0;36m'; NC=$'\033[0m'
log()      { printf "%b[INFO]%b %s\n"   "$BLUE" "$NC" "$*"; }
ok()       { printf "%b[✓]%b %s\n"      "$GREEN" "$NC" "$*"; }
warn()     { printf "%b[!]%b %s\n"      "$YELLOW" "$NC" "$*"; }
err()      { printf "%b[✗]%b %s\n"      "$RED" "$NC" "$*"; }
step()     { printf "%b[STEP]%b %s\n"   "$CYAN" "$NC" "$*"; }

on_error() {
  err "Build failed on line ${BASH_LINENO[0]} (command: ${BASH_COMMAND})"
}
trap on_error ERR

# -------- cd to repo root --------
cd "$(dirname "$0")/../.."

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  count_lines Release Build (hardened)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# -------- Extract package name & version w/o jq --------
# Robust within [package] section
PKG_NAME=$(awk '
  BEGIN{ insec=0 }
  /^\[package\]/{ insec=1; next }
  /^\[/{ if(insec) exit; next }
  insec && $1 ~ /^name/ { sub(/^name *= *"/,""); sub(/".*$/,""); print; exit }
' Cargo.toml)

VERSION=$(awk '
  BEGIN{ insec=0 }
  /^\[package\]/{ insec=1; next }
  /^\[/{ if(insec) exit; next }
  insec && $1 ~ /^version/ { sub(/^version *= *"/,""); sub(/".*$/,""); print; exit }
' Cargo.toml)

: "${PKG_NAME:?failed to get package name}"
: "${VERSION:?failed to get version}"

log "Crate     : ${PKG_NAME}"
log "Version   : ${VERSION}"

# Optionally suffix version with commit if tree is dirty
if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  COMMIT=$(git rev-parse --short=9 HEAD)
  DIRTY=""
  git diff --quiet || DIRTY="-dirty"
  VERSION_FULL="${VERSION}+${COMMIT}${DIRTY}"
  # Reproducible-ish timestamps
  export SOURCE_DATE_EPOCH="$(git log -1 --pretty=%ct)"
else
  VERSION_FULL="${VERSION}"
fi
log "Build ID  : ${VERSION_FULL}"

# -------- Config knobs --------
# You can override via env:
# TARGET=x86_64-unknown-linux-musl RELEASE_FEATURES="--no-default-features --features minimal"
TARGET="${TARGET:-}"                 # e.g. x86_64-unknown-linux-musl
RELEASE_FEATURES="${RELEASE_FEATURES:---all-features}"
CARGO_BUILD_FLAGS="${CARGO_BUILD_FLAGS:---locked}"
CARGO_TEST_FLAGS="${CARGO_TEST_FLAGS:---all-features --locked --quiet}"
CARGO_CLIPPY_FLAGS="${CARGO_CLIPPY_FLAGS:---all-targets --all-features --locked}"

# Opportunistic size/speed tuning (safe defaults)
export CARGO_PROFILE_RELEASE_LTO="${CARGO_PROFILE_RELEASE_LTO:-fat}"
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="${CARGO_PROFILE_RELEASE_CODEGEN_UNITS:-1}"
export CARGO_PROFILE_RELEASE_STRIP="${CARGO_PROFILE_RELEASE_STRIP:-none}" # we call strip later if available

# Some linkers accept -s; do it only if supported and strip exists
STRIP_BIN="$(command -v strip || true)"

# -------- 1) Clean --------
step "1/8 Cleaning previous builds…"
cargo clean
ok "Clean complete"
echo ""

# -------- 2) Format check --------
step "2/8 Checking format…"
cargo fmt -- --check
ok "Format OK"
echo ""

# -------- 3) Tests --------
step "3/8 Running tests…"
cargo test ${CARGO_TEST_FLAGS}
ok "Tests passed"
echo ""

# -------- 4) Clippy (fail on warnings) --------
step "4/8 Running clippy…"
cargo clippy ${CARGO_CLIPPY_FLAGS} -- -D warnings
ok "Clippy passed (no warnings)"
echo ""

# -------- 5) Build (release, optional target) --------
step "5/8 Building optimized release…"
if [ -n "$TARGET" ]; then
  log "Target: ${TARGET}"
  cargo build --release ${CARGO_BUILD_FLAGS} --target "$TARGET" ${RELEASE_FEATURES}
  BIN_PATH="target/${TARGET}/release/${PKG_NAME}"
else
  cargo build --release ${CARGO_BUILD_FLAGS} ${RELEASE_FEATURES}
  BIN_PATH="target/release/${PKG_NAME}"
fi

[ -f "$BIN_PATH" ] || { err "Binary not found: $BIN_PATH"; exit 1; }
[ -x "$BIN_PATH" ] || { err "Binary not executable: $BIN_PATH"; exit 1; }
ok "Release build complete"
echo ""

# -------- 6) Post-process (strip / dsym) --------
step "6/8 Post-processing binary…"
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Normalize arch names
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
esac

# Strip if possible (Linux/Windows). On macOS, prefer dsymutil then strip -x
if [ -n "$STRIP_BIN" ]; then
  case "$OS" in
    darwin)
      if command -v dsymutil >/dev/null 2>&1; then
        dsymutil "$BIN_PATH" >/dev/null 2>&1 || true
      fi
      "$STRIP_BIN" -x "$BIN_PATH" || true
      ;;
    *)
      "$STRIP_BIN" "$BIN_PATH" || true
      ;;
  esac
  ok "Binary stripped (best effort)"
else
  warn "strip not found; skipping"
fi
echo ""

# Quick smoke
"$BIN_PATH" --version >/dev/null
ok "Binary verified: $("$BIN_PATH" --version)"
echo ""

# -------- 7) Package --------
step "7/8 Creating release package…"
RELEASE_DIR="release"
PKG_OS="$OS"
# WSL reports 'linux'; that's fine.
PKG_NAME_ARCHIVE="${PKG_NAME}-${VERSION_FULL}-${PKG_OS}-${ARCH}"
PKG_DIR="${RELEASE_DIR}/${PKG_NAME_ARCHIVE}"

mkdir -p "$PKG_DIR"
cp "$BIN_PATH" "$PKG_DIR/"

# Docs & licenses (tolerant)
cp -f README.md "$PKG_DIR/" 2>/dev/null || warn "README.md not found"
# pick any LICENSE variants present
found_license=0
for f in LICENSE LICENSE-* COPYING; do
  if [ -e "$f" ]; then cp -f "$f" "$PKG_DIR/"; found_license=1; fi
done
[ "$found_license" -eq 0 ] && warn "No LICENSE files found"

if [ -d docs ]; then
  mkdir -p "$PKG_DIR/docs"
  cp -R docs/* "$PKG_DIR/docs/" 2>/dev/null || true
fi

# Install guide
cat > "$PKG_DIR/INSTALL.txt" <<EOF
${PKG_NAME} v${VERSION_FULL}
Installation Instructions

1) Copy the '${PKG_NAME}' binary into your PATH:

   sudo install -m 0755 ${PKG_NAME} /usr/local/bin/${PKG_NAME}

   # or user-local:
   install -d "\$HOME/.local/bin"
   install -m 0755 ${PKG_NAME} "\$HOME/.local/bin/${PKG_NAME}"
   export PATH="\$HOME/.local/bin:\$PATH"

2) Verify:
   ${PKG_NAME} --version

3) Help:
   ${PKG_NAME} --help
EOF

# Build metadata
{
  echo "crate=${PKG_NAME}"
  echo "version=${VERSION_FULL}"
  echo "target=${TARGET:-host}"
  echo "os=${OS}"
  echo "arch=${ARCH}"
  echo "build_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  command -v rustc >/dev/null && echo "rustc=$(rustc -V)"
  command -v cargo >/dev/null && echo "cargo=$(cargo -V)"
} > "$PKG_DIR/BUILDINFO.txt"

if command -v git >/dev/null 2>&1; then
  git rev-parse HEAD 2>/dev/null > "$PKG_DIR/COMMIT.txt" || true
fi

# Archive: prefer .tar.zst, fallback .tar.gz
mkdir -p "$RELEASE_DIR"
pushd "$RELEASE_DIR" >/dev/null
ARCHIVE=""
if command -v zstd >/dev/null 2>&1; then
  ARCHIVE="${PKG_NAME_ARCHIVE}.tar.zst"
  tar -cf - "${PKG_NAME_ARCHIVE}" | zstd -19 -q -o "$ARCHIVE"
else
  ARCHIVE="${PKG_NAME_ARCHIVE}.tar.gz"
  tar -czf "$ARCHIVE" "${PKG_NAME_ARCHIVE}"
fi
SIZE=$(du -h "$ARCHIVE" | cut -f1)
popd >/dev/null
ok "Archive created: ${RELEASE_DIR}/${ARCHIVE} (${SIZE})"

# Checksums
if command -v sha256sum >/dev/null 2>&1; then
  ( cd "$RELEASE_DIR" && sha256sum "$ARCHIVE" > "${ARCHIVE}.sha256" )
  ok "SHA256 generated"
elif command -v shasum >/dev/null 2>&1; then
  ( cd "$RELEASE_DIR" && shasum -a 256 "$ARCHIVE" > "${ARCHIVE}.sha256" )
  ok "SHA256 generated (shasum)"
else
  warn "No sha256 tool found; skipping checksum"
fi
echo ""

# -------- 8) Summary --------
step "8/8 Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
log "Version : ${VERSION_FULL}"
log "Binary  : ${BIN_PATH}"
log "Package : ${PKG_DIR}/"
log "Archive : ${RELEASE_DIR}/${ARCHIVE}"
log "BinSize : $(du -h "$BIN_PATH" | cut -f1)"
if command -v file >/dev/null 2>&1; then
  printf "Type    : %s\n" "$(file "$BIN_PATH" | cut -d':' -f2-)"
fi
if command -v ldd >/dev/null 2>&1; then
  if ldd "$BIN_PATH" 2>&1 | grep -qi "statically linked"; then
    echo "Linking : statically linked"
  else
    echo "Linking : dynamically linked"
  fi
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ok "Release is ready for distribution!"
echo "Next:"
echo "  • Test : ${BIN_PATH} --help"
echo "  • Install : sudo install -m 0755 ${BIN_PATH} /usr/local/bin/${PKG_NAME}"
echo "  • Distribute : ${RELEASE_DIR}/${ARCHIVE}"
