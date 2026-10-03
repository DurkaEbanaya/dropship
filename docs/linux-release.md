Native Linux Dropship with manual/continuous server ping and per-user nftables/iptables firewall support.

**Packages (x86_64):**

- Debian 12+, Ubuntu 24.04+: download `dropship_*_amd64.deb`, install with `sudo apt install ./dropship_*_amd64.deb`.
- Fedora and openSUSE (including Tumbleweed): download `dropship-*.x86_64.rpm`. Install on Fedora with `sudo dnf install ./dropship-*.x86_64.rpm`; on openSUSE with `sudo zypper --no-gpg-checks install ./dropship-*.x86_64.rpm` (locally published packages are unsigned).
- Arch Linux / Manjaro: download `dropship-*.pkg.tar.zst`, install with `sudo pacman -U ./dropship-*.pkg.tar.zst`.

The Linux executable in all three packages was built on Debian 12 (glibc 2.36). The complete corresponding sources are provided as `dropship-linux-3.0.6.tar.gz` and an RPM source package. `SHA256SUMS` lists release file digests. Run `dropship` as a normal desktop user after installation. Firewall changes require polkit authentication. These packages are **not** the upstream Windows executable.

See [Linux documentation](https://github.com/DurkaEbanaya/dropship/blob/main/docs/linux.md) for usage, firewall behavior and building from source.
