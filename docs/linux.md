# Native Linux port / openSUSE

This fork of [stowmyy/dropship](https://github.com/stowmyy/dropship) retains the
original Rust/egui interface and GPL-3.0 license. It runs natively on Linux;
Overwatch itself can run through Steam/Proton or Battle.net/Wine.

## openSUSE Tumbleweed: build and install

Rust stable 1.96 or newer is required (the egui dependency may raise this minimum).
Use `rustup` if your distribution's Rust package is older.

```sh
sudo zypper install gcc gcc-c++ cmake pkgconf-pkg-config \
  libX11-devel libXcursor-devel libXrandr-devel libXi-devel \
  wayland-devel libxkbcommon-devel Mesa-libEGL-devel Mesa-libGL-devel \
  python3-base polkit nftables iptables iputils
git clone https://github.com/DurkaEbanaya/dropship.git
cd dropship
cargo build --release --locked --manifest-path dropship/Cargo.toml
bash packaging/linux/install.sh
dropship
```

The GUI can also be opened from the desktop application menu. An authentication
dialog appears on the first firewall change per application session. A desktop
polkit authentication agent (normally supplied by KDE/GNOME) must be running.
Do not launch the GUI with `sudo` or through Proton.

To build an RPM on openSUSE after compiling:

```sh
sudo zypper install rpm-build
bash packaging/linux/package-rpm.sh
sudo zypper --no-gpg-checks install ./dist/dropship-3.0.6-2.x86_64.rpm
```

The `dist/` directory also contains the source RPM and a complete project source
bundle with the compiled binary. Build this RPM on the target distribution;
the binary links against its glibc. Leap and other distributions need their own
builds and have not been verified with this Tumbleweed binary.

Locally built RPMs are unsigned. Discover/PackageKit on openSUSE can reject them
with `Internal error: Installation has been aborted as directed.` Its backend
log (`/var/log/pk_backend_zypp`) reports `Signature verification failed` and
`File is unsigned`. Use the terminal command above for this locally built RPM;
`--no-gpg-checks` applies to that invocation, without changing system settings.
After installation, launch `dropship` as your normal user.

## Use

1. Close Overwatch and open Dropship.
2. Block servers you do **not** want to use; right-click toggles the other servers.
3. Authorize the firewall change and wait for the selection to finish applying.
4. Start Overwatch normally in Steam or your Wine launcher.
5. `disable dropship` removes all blocks for your user, including while the game
   is open. Changes to ordinary selections wait until the game closes.

Adding an `.exe` is unnecessary on Linux. The application watches process names
and command-line basenames to recognize `Overwatch.exe` under Wine/Proton.

Each server row shows its average ping in milliseconds alongside the signal
icon. `... ms` means measurement is in progress; `n/a` means the test failed
(hover for details). This is the average of four ICMP probes to the region's
test IP. Use **refresh all pings** above the list to measure all regions again,
or the refresh icon beside one server to measure only that region. A running
measurement disables its refresh button; previous results remain visible with
a spinner while being refreshed. Refresh controls do not change server blocks.

Enable **continuous ping** to repeat each region's measurement five seconds
after it completes, including retrying failed tests. Disable it to stop future
automatic measurements; any running measurement finishes normally. The setting
is saved between launches and works in compact mode too. Measurements also run
once at startup and when a new test IP appears. The displayed result is a
regional estimate, not live in-game latency. Linux uses `/usr/bin/ping` from
`iputils`.

## Firewall behavior

The root-owned `/etc/dropship/firewall.json` selects the backend:

```json
{"backend": "auto"}
```

`auto` uses nftables when `/usr/sbin/nft` is installed; otherwise it requires
`iptables`, `ip6tables` and their `*-restore` tools. Set `"backend": "iptables"`
to force iptables, or `"backend": "nftables"` to force nftables. To edit it:

```sh
sudoedit /etc/dropship/firewall.json
```

Close and reopen Dropship after changing this setting. On its next successful
application it removes the previous backend's rules. Keep the old backend tools
installed until that migration is complete. Boot restoration uses the backend
recorded with each user's saved selection.

- nftables table: `inet dropship_<UID>`. Existing firewalld/nftables tables are
  preserved; Dropship never runs `flush ruleset` or resets the system firewall.
- iptables/ip6tables chain: `DSHIP_<UID>` in each family's `filter` table, reached
  by an owner/UDP/port-matched jump inserted at the start of `OUTPUT`. Works with
  the distribution's selected iptables-nft or iptables-legacy implementation.
  `*-restore --noflush --wait` changes only Dropship's chain and jump, preserving
  other chains, policies and rules. Each family commits atomically; IPv4 is
  rolled back if the IPv6 transaction fails. Both families' tools are required.
  This Tumbleweed host was packet-tested with nftables and iptables-nft;
  iptables-legacy requires the `ip_tables`/`ip6_tables` kernel modules and was
  not packet-tested here because those modules were not loaded.
- Rules match only sockets owned by the launching user, destinations in the
  selected upstream IPv4/IPv6 network lists, and UDP destination ports
  **12000–64000** (Overwatch game traffic). Other users, TCP/HTTPS, ICMP ping and
  UDP outside this range are unaffected. Other programs using matching UDP
  destinations/ports for the same user **are** affected; Linux does not provide
  Windows WFP's executable-path filtering here. The range is deliberately broad
  to cover game-server ports and can be adjusted in `dropship-firewall`.
- `always`: survives closing Dropship and is restored at boot by
  `dropship-restore.service`, after the system firewall services. State lives in
  `/var/lib/dropship/<UID>.json`, root-owned. Only successful changes are saved.
  A separate `<UID>.runtime.json` marker tracks applied runtime rules for cleanup
  and migration; it is never restored at boot.
- `only while dropship is open`: the root helper removes rules on stdin EOF,
  including when the GUI crashes. If the helper itself is forcibly killed,
  reopen Dropship and click `disable dropship` to clear remaining runtime rules.
- A firewall tool that explicitly flushes the selected backend's tables removes
  these rules too. Restore with `sudo systemctl restart dropship-restore.service` or
  reopen Dropship. Normal firewalld use keeps the separately owned table.
- nftables **1.0.9+** is required for atomic, idempotent table replacement using
  `destroy table`. There is one GUI instance and one helper per user.

The GUI runs unprivileged. The root-owned helper accepts bounded JSON requests
with CIDRs and a persistence boolean only. The caller UID comes from pkexec,
and IP addresses are parsed before constructing nft syntax. No GUI-supplied
shell, commands, nft scripts, filesystem paths or user IDs are executed.

Server data is still fetched from the upstream IP-list endpoint. On Linux the
Windows executable self-updater is disabled: update the binary and helper
together using the installer/RPM. Interface settings and cached server data are
stored by eframe under the user's XDG data directory (normally
`~/.local/share/dropship/`).

## Verification

```sh
cargo check --locked --manifest-path dropship/Cargo.toml
cargo test --locked --manifest-path dropship/Cargo.toml
python3 -m unittest discover -s packaging/linux -p 'test_*.py'
unshare --user --map-current-user --keep-caps --net \
  python3 packaging/linux/test_firewall.py --integration
```

Run the same kernel test with iptables/ip6tables installed:

```sh
unshare --user --map-current-user --keep-caps --net \
  python3 packaging/linux/test_firewall.py --integration --backend iptables
```

The integration test uses a disposable network namespace, real kernel firewall
rules and loopback IPv4/IPv6 packets. It checks blocked UDP, allowed TCP and
out-of-range UDP, table replacement, preservation of unrelated tables, invalid
requests, dynamic cleanup, persistent restoration and backend migration. It
requires nftables, `iproute2` and enabled unprivileged user namespaces. The
iptables test also needs iptables/ip6tables. It does not change the host firewall.

## Removal

Close Dropship first. For the script installation:

```sh
bash packaging/linux/uninstall.sh
```

For the RPM installation: `sudo zypper remove dropship`. Both remove the owned
tables and boot service; user interface preferences are retained.
