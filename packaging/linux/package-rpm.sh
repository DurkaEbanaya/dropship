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
name="dropship-linux-$version"
mkdir -p "$temporary/$name/bin" "$temporary/rpmbuild/SOURCES"
# Package current sources (including uncommitted port changes), excluding build products.
tar -C "$root" --exclude=.git --exclude=target --exclude=dist --exclude=__pycache__ -cf - . | tar -C "$temporary/$name" -xf -
install -m755 "$binary" "$temporary/$name/bin/dropship"
tar -C "$temporary" -czf "$output/$name.tar.gz" "$name"
cp "$output/$name.tar.gz" "$temporary/rpmbuild/SOURCES/"
for distro in suse fedora; do
    rm -rf "$temporary/rpmbuild/BUILD" "$temporary/rpmbuild/BUILDROOT"
    rpmbuild -ba --define "_topdir $temporary/rpmbuild" --define "dropship_distro $distro" "$root/packaging/linux/dropship.spec"
done
cp "$temporary"/rpmbuild/RPMS/x86_64/*.rpm "$temporary"/rpmbuild/SRPMS/*.rpm "$output/"
printf 'RPMs and corresponding source bundle: %s\n' "$output"
