#!/bin/sh
# Called only for package removal, while the helper is still installed.
for state in /var/lib/dropship/*.json /var/lib/dropship/*.lock; do
    [ -e "$state" ] || continue
    uid=${state##*/}; uid=${uid%%.*}
    case "$uid" in *[!0-9]*|'') continue ;; esac
    /usr/libexec/dropship-firewall --clear-user "$uid" || :
done
systemctl disable --now dropship-restore.service >/dev/null 2>&1 || :
rm -f /var/lib/dropship/*.json /var/lib/dropship/*.lock
