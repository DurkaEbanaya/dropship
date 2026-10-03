Native Linux Dropship with manual/continuous server ping and per-user nftables/iptables firewall support.

**Packages (x86_64):**

- Debian 12+, Ubuntu 24.04+: download `dropship_*_amd64.deb`, install with `sudo apt install ./dropship_*_amd64.deb`.
- Fedora: download `dropship-*-3.fc.x86_64.rpm` and install with `sudo dnf install ./dropship-*-3.fc.x86_64.rpm`.
- openSUSE (including Tumbleweed): download `dropship-*-3.suse.x86_64.rpm` and install with `sudo zypper --no-gpg-checks install ./dropship-*-3.suse.x86_64.rpm` (published packages are unsigned).
- Arch Linux / Manjaro: download `dropship-*.pkg.tar.zst`, install with `sudo pacman -U ./dropship-*.pkg.tar.zst`.

The Linux executable in all three packages was built on Debian 12 (glibc 2.36). The complete corresponding sources are provided as `dropship-linux-3.0.6.tar.gz` and an RPM source package. `SHA256SUMS` lists release file digests. Run `dropship` as a normal desktop user after installation. Firewall changes require polkit authentication. These packages are **not** the upstream Windows executable.

See [Linux documentation](https://github.com/DurkaEbanaya/dropship/blob/main/docs/linux.md) for usage, firewall behavior and building from source.
