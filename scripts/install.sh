#!/usr/bin/env sh

set -eu

REPO="${DSVIEW_REPO:-LISTENAI/dsview-cli}"
PREFIX="${DSVIEW_PREFIX:-$HOME/.local/opt/dsview-cli}"
BIN_DIR="${DSVIEW_BIN_DIR:-$HOME/.local/bin}"
VERSION=""
FORCE=0
SKIP_CHECKSUM=0
DRY_RUN=0

usage() {
    cat <<EOF
Install DSView CLI from GitHub release bundles.

Usage:
  install.sh [options]

Options:
  --version <tag>       Install a specific release tag (default: latest)
  --prefix <path>       Bundle install root (default: $PREFIX)
  --bin-dir <path>      Wrapper script directory (default: $BIN_DIR)
  --repo <owner/name>   GitHub repository (default: $REPO)
  --force               Replace an existing installation for the same version
  --skip-checksum       Skip SHA-256 verification
  --dry-run             Print the resolved install plan without downloading
  --help                Show this help message
EOF
}

log() {
    printf '==> %s\n' "$*"
}

warn() {
    printf 'warning: %s\n' "$*" >&2
}

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

download_to() {
    url="$1"
    destination="$2"

    if command -v curl >/dev/null 2>&1; then
        if curl -fsSL --retry 3 --retry-delay 1 --connect-timeout 20 "$url" -o "$destination"; then
            return
        fi
        warn "curl download failed for $url, trying wget"
    fi
    if command -v wget >/dev/null 2>&1; then
        wget -q --tries=3 -O "$destination" "$url"
        return
    fi

    die "either curl or wget is required"
}

fetch_text() {
    url="$1"

    if command -v curl >/dev/null 2>&1; then
        if curl -fsSL --retry 3 --retry-delay 1 --connect-timeout 20 "$url"; then
            return
        fi
        warn "curl request failed for $url, trying wget"
    fi
    if command -v wget >/dev/null 2>&1; then
        wget -qO- "$url"
        return
    fi

    die "either curl or wget is required"
}

sha256_file() {
    file="$1"

    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{print $1}'
        return
    fi
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{print $1}'
        return
    fi

    die "either sha256sum or shasum is required"
}

detect_target() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            case "$arch" in
                x86_64|amd64) printf '%s\n' "x86_64-unknown-linux-gnu" ;;
                aarch64|arm64) printf '%s\n' "aarch64-unknown-linux-gnu" ;;
                *) die "unsupported Linux architecture: $arch" ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64|amd64) printf '%s\n' "x86_64-apple-darwin" ;;
                arm64|aarch64) printf '%s\n' "aarch64-apple-darwin" ;;
                *) die "unsupported macOS architecture: $arch" ;;
            esac
            ;;
        *)
            die "unsupported operating system: $os (supported: Linux, macOS)"
            ;;
    esac
}

resolve_latest_version() {
    api_url="https://api.github.com/repos/$REPO/releases/latest"
    tag_name="$(
        fetch_text "$api_url" \
            | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
            | head -n 1
    )"

    [ -n "$tag_name" ] || die "unable to determine the latest release from $api_url"
    printf '%s\n' "$tag_name"
}

verify_checksum() {
    archive_path="$1"
    checksum_path="$2"
    asset_name="$3"

    expected="$(
        awk -v asset="$asset_name" '
            {
                entry = $2
                sub(/^.*\//, "", entry)
                if (entry == asset) {
                    print $1
                    exit
                }
            }
        ' "$checksum_path"
    )"
    [ -n "$expected" ] || die "checksum entry not found for $asset_name"

    actual="$(sha256_file "$archive_path")"
    [ "$actual" = "$expected" ] || die "checksum mismatch for $asset_name"
}

write_wrapper() {
    wrapper_path="$1"
    target_path="$2"

    cat >"$wrapper_path" <<EOF
#!/usr/bin/env sh
set -eu
exec "$target_path" "\$@"
EOF
    chmod 755 "$wrapper_path"
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version)
            [ "$#" -ge 2 ] || die "--version requires a value"
            VERSION="$2"
            shift 2
            ;;
        --prefix)
            [ "$#" -ge 2 ] || die "--prefix requires a value"
            PREFIX="$2"
            shift 2
            ;;
        --bin-dir)
            [ "$#" -ge 2 ] || die "--bin-dir requires a value"
            BIN_DIR="$2"
            shift 2
            ;;
        --repo)
            [ "$#" -ge 2 ] || die "--repo requires a value"
            REPO="$2"
            shift 2
            ;;
        --force)
            FORCE=1
            shift
            ;;
        --skip-checksum)
            SKIP_CHECKSUM=1
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            die "unknown argument: $1"
            ;;
    esac
done

need_cmd tar

TARGET="$(detect_target)"
if [ -z "$VERSION" ]; then
    log "Resolving latest release for $REPO"
    VERSION="$(resolve_latest_version)"
fi

ASSET_NAME="dsview-cli-$VERSION-$TARGET.tar.gz"
CHECKSUM_NAME="dsview-cli-$VERSION-SHA256SUMS.txt"
RELEASE_BASE_URL="https://github.com/$REPO/releases/download/$VERSION"
ARCHIVE_URL="$RELEASE_BASE_URL/$ASSET_NAME"
CHECKSUM_URL="$RELEASE_BASE_URL/$CHECKSUM_NAME"
INSTALL_DIR="$PREFIX/$VERSION"
CURRENT_LINK="$PREFIX/current"
WRAPPER_PATH="$BIN_DIR/dsview-cli"
RUNTIME_TARGET="$CURRENT_LINK/dsview-cli"

if [ "$DRY_RUN" -eq 1 ]; then
    printf 'repo=%s\n' "$REPO"
    printf 'version=%s\n' "$VERSION"
    printf 'target=%s\n' "$TARGET"
    printf 'archive_url=%s\n' "$ARCHIVE_URL"
    printf 'checksum_url=%s\n' "$CHECKSUM_URL"
    printf 'install_dir=%s\n' "$INSTALL_DIR"
    printf 'wrapper_path=%s\n' "$WRAPPER_PATH"
    exit 0
fi

if [ -e "$INSTALL_DIR" ] || [ -L "$INSTALL_DIR" ]; then
    if [ "$FORCE" -ne 1 ]; then
        die "installation already exists at $INSTALL_DIR (use --force to replace it)"
    fi
    rm -rf "$INSTALL_DIR"
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/dsview-cli-install.XXXXXX")"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

mkdir -p "$PREFIX" "$BIN_DIR" "$TMP_DIR/extract"

log "Downloading $ASSET_NAME"
download_to "$ARCHIVE_URL" "$TMP_DIR/$ASSET_NAME"

if [ "$SKIP_CHECKSUM" -eq 0 ]; then
    log "Downloading checksum file"
    download_to "$CHECKSUM_URL" "$TMP_DIR/$CHECKSUM_NAME"
    log "Verifying checksum"
    verify_checksum "$TMP_DIR/$ASSET_NAME" "$TMP_DIR/$CHECKSUM_NAME" "$ASSET_NAME"
else
    warn "Skipping checksum verification"
fi

log "Extracting release bundle"
tar -xzf "$TMP_DIR/$ASSET_NAME" -C "$TMP_DIR/extract"

BUNDLE_ROOT="$TMP_DIR/extract/dsview-cli-$VERSION-$TARGET"
[ -d "$BUNDLE_ROOT" ] || die "expected extracted bundle directory missing: $BUNDLE_ROOT"

log "Installing bundle to $INSTALL_DIR"
mv "$BUNDLE_ROOT" "$INSTALL_DIR"

if [ -e "$CURRENT_LINK" ] || [ -L "$CURRENT_LINK" ]; then
    rm -rf "$CURRENT_LINK"
fi
ln -s "$INSTALL_DIR" "$CURRENT_LINK"

log "Writing launcher to $WRAPPER_PATH"
write_wrapper "$WRAPPER_PATH" "$RUNTIME_TARGET"

log "Running smoke checks"
"$WRAPPER_PATH" --version >/dev/null
"$WRAPPER_PATH" devices list --help >/dev/null

log "Installed DSView CLI $VERSION for $TARGET"
log "Run: $WRAPPER_PATH --help"

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
        warn "$BIN_DIR is not on PATH"
        warn "add this to your shell profile: export PATH=\"$BIN_DIR:\$PATH\""
        ;;
esac

if [ "$(uname -s)" = "Linux" ]; then
    cat <<EOF

Linux note:
  Accessing DSLogic hardware may require a udev rule. To install the bundled rule:
    printf '%s\n' 'SUBSYSTEM=="usb", ATTRS{idVendor}=="2a0e", MODE="0666"' | \\
      sudo tee /etc/udev/rules.d/99-dsview-cli.rules >/dev/null
    sudo udevadm control --reload-rules
    sudo udevadm trigger

  Then unplug and reconnect the device.
EOF
fi
