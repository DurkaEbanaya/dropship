Name:           dropship
Version:        3.0.6
Release:        2
Summary:        Native Linux Overwatch server selector
License:        GPL-3.0-only
URL:            https://github.com/DurkaEbanaya/dropship
# Bundle the locally built binary and GPL sources using package-rpm.sh.
Source0:        dropship-linux-%{version}.tar.gz
%global debug_package %{nil}
BuildArch:      x86_64
Requires:       (nftables >= 1.0.9 or iptables)
Requires:       polkit
Requires:       python3-base
Requires:       iputils
Requires:       libxkbcommon0
Requires:       libwayland-client0
Requires:       libX11-6
Requires:       libXcursor1
Requires:       libXrandr2
Requires:       libXi6
Requires:       Mesa-libEGL1
Requires:       Mesa-libGL1

%description
Dropship's native Linux port selects Overwatch servers by filtering the user's
outgoing UDP game traffic with nftables or iptables/ip6tables. Supports Steam/Proton and Wine.

%prep
%setup -q -n dropship-linux-%{version}

%build
# Prebuilt on the target distribution; complete corresponding source is included.

%install
install -Dm755 bin/dropship %{buildroot}%{_bindir}/dropship
install -Dm755 packaging/linux/dropship-firewall %{buildroot}%{_libexecdir}/dropship-firewall
install -Dm644 packaging/linux/io.github.durkaebanaya.dropship.firewall.policy %{buildroot}%{_datadir}/polkit-1/actions/io.github.durkaebanaya.dropship.firewall.policy
install -Dm644 packaging/linux/dropship-restore.service %{buildroot}%{_unitdir}/dropship-restore.service
install -Dm644 packaging/linux/dropship.desktop %{buildroot}%{_datadir}/applications/dropship.desktop
install -Dm644 dropship/assets/white-bolts.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/dropship.png
install -d -m755 %{buildroot}%{_localstatedir}/lib/dropship
install -Dm644 packaging/linux/firewall.json %{buildroot}%{_sysconfdir}/dropship/firewall.json

%post
systemctl daemon-reload
systemctl enable dropship-restore.service >/dev/null 2>&1 || :

%preun
if [ "$1" = 0 ]; then
    for state in /var/lib/dropship/*.json /var/lib/dropship/*.lock; do
        [ -e "$state" ] || continue
        uid=${state##*/}; uid=${uid%%.*}
        case "$uid" in *[!0-9]*|'') continue ;; esac
        /usr/libexec/dropship-firewall --clear-user "$uid" || :
    done
    systemctl disable --now dropship-restore.service >/dev/null 2>&1 || :
    rm -f /var/lib/dropship/*.json /var/lib/dropship/*.lock
fi

%postun
systemctl daemon-reload

%files
%license LICENSE
%doc README.md docs/linux.md
%{_bindir}/dropship
%{_libexecdir}/dropship-firewall
%{_datadir}/polkit-1/actions/io.github.durkaebanaya.dropship.firewall.policy
%{_unitdir}/dropship-restore.service
%{_datadir}/applications/dropship.desktop
%{_datadir}/icons/hicolor/256x256/apps/dropship.png
%dir %{_localstatedir}/lib/dropship
%dir %{_sysconfdir}/dropship
%config(noreplace) %{_sysconfdir}/dropship/firewall.json
