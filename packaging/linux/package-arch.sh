#!/usr/bin/env bash
set -euo pipefail
root=$(dirname "$(dirname "$(dirname "$(realpath "$0")")")")
version=$(python3 -c 'import sys,tomllib; print(tomllib.load(open(sys.argv[1],"rb"))["package"]["version"])' "$root/dropship/Cargo.toml")
archive="$root/dist/dropship-linux-$version.tar.gz"
[[ -f "$archive" ]] || { printf 'Run bash packaging/linux/package-rpm.sh first.\n' >&2; exit 1; }
output="$root/dist"
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
cp "$archive" "$root/packaging/linux/PKGBUILD" "$root/packaging/linux/dropship.install" "$temporary/"
# The source archive is generated locally, so pin its digest rather than SKIP.
python3 - "$temporary/PKGBUILD" "$archive" <<'PY'
import hashlib
import pathlib
import sys
pkgbuild = pathlib.Path(sys.argv[1])
sha = hashlib.sha256(pathlib.Path(sys.argv[2]).read_bytes()).hexdigest()
pkgbuild.write_text(pkgbuild.read_text().replace("sha256sums=('SKIP')", f"sha256sums=('{sha}')"))
PY
if command -v makepkg >/dev/null; then
    [[ $EUID -ne 0 ]] || { printf 'Run makepkg as a regular Arch user.\n' >&2; exit 1; }
    (cd "$temporary" && PKGDEST="$temporary" makepkg --nodeps --noconfirm --clean --force)
elif command -v docker >/dev/null; then
    chmod 755 "$temporary"
    docker run --rm -v "$temporary:/build" -w /build archlinux:base-devel sh -ec \
      'useradd -m builder; chown -R builder:builder /build; su builder -c "cd /build && PKGDEST=/build makepkg --nodeps --noconfirm --clean"'
else
    printf 'Arch makepkg or Docker is required.\n' >&2
    exit 1
fi
cp "$temporary/dropship-$version-3-x86_64.pkg.tar.zst" "$output/"
