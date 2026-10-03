#!/usr/bin/env bash
set -euo pipefail
root=$(dirname "$(dirname "$(dirname "$(realpath "$0")")")")
stage=$1
binary=$2
install -Dm755 "$binary" "$stage/usr/bin/dropship"
install -Dm755 "$root/packaging/linux/dropship-firewall" "$stage/usr/libexec/dropship-firewall"
install -Dm755 "$root/packaging/linux/clear-installed-rules.sh" "$stage/usr/share/dropship/clear-installed-rules.sh"
install -Dm644 "$root/packaging/linux/io.github.durkaebanaya.dropship.firewall.policy" "$stage/usr/share/polkit-1/actions/io.github.durkaebanaya.dropship.firewall.policy"
install -Dm644 "$root/packaging/linux/dropship-restore.service" "$stage/usr/lib/systemd/system/dropship-restore.service"
install -Dm644 "$root/packaging/linux/dropship.desktop" "$stage/usr/share/applications/dropship.desktop"
install -Dm644 "$root/dropship/assets/white-bolts.png" "$stage/usr/share/icons/hicolor/256x256/apps/dropship.png"
install -Dm644 "$root/packaging/linux/firewall.json" "$stage/etc/dropship/firewall.json"
install -Dm644 "$root/LICENSE" "$stage/usr/share/licenses/dropship/LICENSE"
install -Dm644 "$root/README.md" "$stage/usr/share/doc/dropship/README.md"
install -Dm644 "$root/docs/linux.md" "$stage/usr/share/doc/dropship/linux.md"
install -d -m755 "$stage/var/lib/dropship"
