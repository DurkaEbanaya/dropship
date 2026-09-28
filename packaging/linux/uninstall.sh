#!/usr/bin/env bash
set -euo pipefail
if [[ $EUID -ne 0 ]]; then
    exec sudo bash "$0"
fi
# Only remove Dropship rules. Never flush the machine's firewall.
for state in /var/lib/dropship/*.json /var/lib/dropship/*.lock; do
    [[ -e "$state" ]] || continue
    uid=${state##*/}
    uid=${uid%%.*}
    [[ "$uid" =~ ^[0-9]+$ ]] || continue
    /usr/libexec/dropship-firewall --clear-user "$uid"
done
systemctl disable --now dropship-restore.service
rm -f /usr/bin/dropship /usr/libexec/dropship-firewall \
    /usr/share/polkit-1/actions/io.github.durkaebanaya.dropship.firewall.policy \
    /usr/lib/systemd/system/dropship-restore.service \
    /usr/share/applications/dropship.desktop \
    /usr/share/icons/hicolor/256x256/apps/dropship.png
rm -rf /var/lib/dropship /usr/share/licenses/dropship
rm -f /etc/dropship/firewall.json
rmdir /etc/dropship 2>/dev/null || true
systemctl daemon-reload
printf 'Dropship removed. User interface preferences were retained.\n'
