#!/usr/bin/env bash
set -euo pipefail
root=$(dirname "$(dirname "$(dirname "$(realpath "$0")")")")
binary=${1:-"$root/dropship/target/release/dropship"}
if [[ ! -x "$binary" ]]; then
    printf 'Build first: cargo build --release --manifest-path "%s/dropship/Cargo.toml"\n' "$root" >&2
    exit 1
fi
for dependency in /usr/bin/python3 /usr/bin/pkexec /usr/bin/ping; do
    [[ -x "$dependency" ]] || { printf 'Missing dependency: %s\n' "$dependency" >&2; exit 1; }
done
if [[ ! -x /usr/sbin/nft ]]; then
    for dependency in /usr/sbin/iptables /usr/sbin/ip6tables /usr/sbin/iptables-restore /usr/sbin/ip6tables-restore; do
        [[ -x "$dependency" ]] || { printf 'Install nftables or iptables first. Missing: %s\n' "$dependency" >&2; exit 1; }
    done
fi
# Installation needs one elevation; subsequent GUI runs are ordinary user processes.
if [[ $EUID -ne 0 ]]; then
    exec sudo bash "$0" "$binary"
fi
install -Dm755 "$binary" /usr/bin/dropship
install -Dm755 "$root/packaging/linux/dropship-firewall" /usr/libexec/dropship-firewall
install -Dm644 "$root/packaging/linux/io.github.durkaebanaya.dropship.firewall.policy" /usr/share/polkit-1/actions/io.github.durkaebanaya.dropship.firewall.policy
install -Dm644 "$root/packaging/linux/dropship-restore.service" /usr/lib/systemd/system/dropship-restore.service
install -Dm644 "$root/packaging/linux/dropship.desktop" /usr/share/applications/dropship.desktop
install -Dm644 "$root/dropship/assets/white-bolts.png" /usr/share/icons/hicolor/256x256/apps/dropship.png
install -Dm644 "$root/LICENSE" /usr/share/licenses/dropship/LICENSE
install -d -m755 /var/lib/dropship
if [[ ! -e /etc/dropship/firewall.json ]]; then
    install -Dm644 "$root/packaging/linux/firewall.json" /etc/dropship/firewall.json
fi
systemctl daemon-reload
systemctl enable dropship-restore.service
printf 'Installed. Start Dropship from the application menu, or run: dropship\n'
