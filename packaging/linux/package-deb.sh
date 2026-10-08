#!/usr/bin/env bash
set -euo pipefail
root=$(dirname "$(dirname "$(dirname "$(realpath "$0")")")")
version=$(python3 -c 'import sys,tomllib; print(tomllib.load(open(sys.argv[1],"rb"))["package"]["version"])' "$root/dropship/Cargo.toml")
binary="$root/dropship/target/release/dropship"
[[ -x "$binary" ]] || { printf 'Run cargo build --release first.\n' >&2; exit 1; }
output="$root/dist"
mkdir -p "$output"
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
stage="$temporary/stage"
mkdir -p "$stage/DEBIAN"
bash "$root/packaging/linux/stage.sh" "$stage" "$binary"
cat > "$stage/DEBIAN/control" <<EOF
Package: dropship
Version: ${version}-4
Section: games
Priority: optional
Architecture: amd64
Maintainer: Dropship Linux fork <https://github.com/DurkaEbanaya/dropship>
Depends: libc6 (>= 2.35), python3, polkitd | policykit-1, iputils-ping, nftables (>= 1.0.9) | iptables, libx11-6, libxcursor1, libxrandr2, libxi6, libwayland-client0, libxkbcommon0, libegl1, libgl1
Description: Native Linux Overwatch server selector
 Per-user UDP game traffic filtering using nftables or iptables/ip6tables.
 Runs with Steam/Proton and Wine on X11 and Wayland desktops.
EOF
install -m755 "$root/packaging/linux/deb-postinst" "$stage/DEBIAN/postinst"
install -m755 "$root/packaging/linux/deb-prerm" "$stage/DEBIAN/prerm"
install -m755 "$root/packaging/linux/deb-postrm" "$stage/DEBIAN/postrm"
printf '/etc/dropship/firewall.json\n' > "$stage/DEBIAN/conffiles"
dpkg-deb --build --root-owner-group "$stage" "$output/dropship_${version}-4_amd64.deb"
