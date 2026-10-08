"""Run: python3 -m unittest discover -s packaging/linux -p 'test_*.py'.
Kernel integration: unshare --user --map-current-user --keep-caps --net \
    python3 packaging/linux/test_firewall.py --integration
"""
import importlib.machinery
import importlib.util
import ipaddress
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

loader = importlib.machinery.SourceFileLoader("helper", str(Path(__file__).with_name("dropship-firewall")))
spec = importlib.util.spec_from_loader(loader.name, loader)
helper = importlib.util.module_from_spec(spec)
loader.exec_module(helper)


class ValidationTests(unittest.TestCase):
    def test_disable_is_explicit_and_cannot_include_blocks(self):
        self.assertEqual(helper.validate_request({"networks": [], "persistent": True, "disable_all": True}), [])
        for flag in ["true", 1]:
            with self.assertRaises(ValueError):
                helper.validate_request({"networks": [], "persistent": True, "disable_all": flag})
        with self.assertRaises(ValueError):
            helper.validate_request({"networks": ["127.0.0.1/32"], "persistent": True, "disable_all": True})

    def test_invalid_input_never_becomes_nft_syntax(self):
        for value in ["0.0.0.0/0", "::/0", "1.2.3.4; flush ruleset", "example.com", None]:
            with self.assertRaises(ValueError):
                helper.validate_request({"networks": [value], "persistent": True})
        for request in [{"networks": [], "persistent": "false"},
                        {"networks": [], "persistent": True, "uid": 0},
                        {"networks": ["127.0.0.1"] * 4097, "persistent": True}]:
            with self.assertRaises(ValueError):
                helper.validate_request(request)

    def test_deduplication_and_normalization(self):
        result = helper.validate_request({"networks": ["192.0.2.3/24", "192.0.2.0/24", "2001:db8::1/32"], "persistent": False})
        self.assertEqual([str(n) for n in result], ["192.0.2.0/24", "2001:db8::/32"])

    def test_state_is_atomic_and_dynamic_is_not_persisted(self):
        with tempfile.TemporaryDirectory() as temporary:
            original = helper.STATE
            helper.STATE = Path(temporary)
            try:
                networks = [ipaddress.ip_network("192.0.2.0/24")]
                helper.save_state(1000, networks, True)
                self.assertEqual(json.loads((helper.STATE / "1000.json").read_text())["networks"], ["192.0.2.0/24"])
                helper.save_state(1000, networks, False)
                self.assertFalse((helper.STATE / "1000.json").exists())
            finally:
                helper.STATE = original

    def test_backend_selection_and_fallback(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(helper, "CONFIG", Path(temporary) / "config.json"):
            with patch.object(helper.os, "access", return_value=True):
                self.assertEqual(helper.select_backend(), "nftables")
                helper.CONFIG.write_text('{"backend":"iptables"}')
                self.assertEqual(helper.select_backend(), "iptables")
            helper.CONFIG.unlink()
            with patch.object(helper.os, "access", side_effect=lambda path, mode: path != helper.NFT):
                self.assertEqual(helper.select_backend(), "iptables")
            with patch.object(helper.os, "access", return_value=False), self.assertRaises(RuntimeError):
                helper.select_backend()

    def test_ipv6_failure_rolls_back_only_our_ipv4_chain(self):
        original = {"exists": False, "rules": [], "hook": [], "attached": False}
        def transaction(uid, version, desired, current):
            if version == 6:
                raise RuntimeError("IPv6 failure")
        with patch.object(helper, "iptables_snapshot", return_value=original), \
             patch.object(helper, "write_iptables", side_effect=transaction) as write:
            with self.assertRaisesRegex(RuntimeError, "IPv6 failure"):
                helper.apply_iptables(1000, [ipaddress.ip_network("192.0.2.0/24")])
            self.assertEqual([call.args[1] for call in write.call_args_list], [4, 6, 4])
            self.assertEqual(write.call_args_list[-1].args[2], original)


def integration(backend="nftables", prefix=None, variant="nft"):
    """Real kernel rules and packets, in a disposable network namespace only."""
    uid = os.getuid()
    assert uid > 0, "map your current UID, not root"
    subprocess.run(["/usr/bin/ip", "link", "set", "lo", "up"], check=True)
    nft = [helper.NFT]
    if prefix:
        helper.IPTABLES = {4: str(Path(prefix) / f"iptables-{variant}"), 6: str(Path(prefix) / f"ip6tables-{variant}")}
        helper.RESTORE = {4: str(Path(prefix) / f"iptables-{variant}-restore"), 6: str(Path(prefix) / f"ip6tables-{variant}-restore")}
    def apply(uid, networks):
        helper.apply_backend(backend, uid, networks)
    # A foreign table must survive every update / reset.
    subprocess.run(nft + ["add", "table", "inet", "unrelated_test"], check=True)
    if backend == "iptables":
        for tool in helper.IPTABLES.values():
            subprocess.run([tool, "-N", "UNRELATED_TEST"], check=True)
            subprocess.run([tool, "-A", "UNRELATED_TEST", "-p", "tcp", "-j", "RETURN"], check=True)
    networks = helper.validate_request({"networks": ["127.0.0.0/8", "127.0.0.1/32", "::1/128"], "persistent": True})
    apply(uid, networks)
    apply(uid, networks)  # replace existing rules, including overlapping CIDRs

    def udp(family, host, port, blocked):
        with socket.socket(family, socket.SOCK_DGRAM) as listener, socket.socket(family, socket.SOCK_DGRAM) as sender:
            listener.bind((host, port))
            listener.settimeout(0.15)
            try:
                sender.sendto(b"dropship-test", (host, port))
            except PermissionError:
                assert blocked, f"unexpected local rejection on {host}:{port}"
                return
            try:
                data = listener.recv(128)
            except socket.timeout:
                assert blocked, f"unexpected UDP block on {host}:{port}"
            else:
                assert not blocked and data == b"dropship-test", "blocked datagram was delivered"

    for family, host in [(socket.AF_INET, "127.0.0.1"), (socket.AF_INET6, "::1")]:
        for port in [12000, 35000, 64000]:
            udp(family, host, port, True)
        for port in [11999, 64001]:
            udp(family, host, port, False)
        with socket.socket(family, socket.SOCK_STREAM) as listener:
            listener.bind((host, 35000))
            listener.listen()
            with socket.create_connection((host, 35000), timeout=1):
                connection, _ = listener.accept()
                connection.close()
    apply(uid, [])
    udp(socket.AF_INET, "127.0.0.1", 35000, False)
    # xt_owner rejects UIDs unmapped in a user namespace. CI's root-created
    # network namespace maps every UID; map-current-user locally maps only one.
    mapped_other_uid = any(int(start) <= uid + 1 < int(start) + int(count)
                          for start, outside, count in (line.split() for line in Path("/proc/self/uid_map").read_text().splitlines()))
    if backend == "nftables" or mapped_other_uid:
        apply(uid + 1, networks)
        udp(socket.AF_INET, "127.0.0.1", 35000, False)  # a different UID is unaffected
        apply(uid + 1, [])
    else:
        print("SKIP other-UID packet test: this user namespace maps only the current UID")
    subprocess.run(nft + ["list", "table", "inet", "unrelated_test"], check=True)
    if backend == "iptables":
        for tool in helper.IPTABLES.values():
            result = subprocess.run([tool, "-S", "UNRELATED_TEST"], text=True, capture_output=True, check=True)
            assert "-p tcp -j RETURN" in result.stdout, "unrelated rule was modified"

    # Exercise the JSON-line protocol, EOF cleanup, persistence and boot restore.
    with tempfile.TemporaryDirectory() as temporary:
        helper.STATE = Path(temporary)
        helper.CONFIG = helper.STATE / "config"
        helper.TERMINAL_STATE = helper.STATE / "terminal"
        helper.TERMINAL_STATE.mkdir()
        source = "import runpy,sys,json; from pathlib import Path; m=runpy.run_path(sys.argv[1]); g=m['serve'].__globals__; g['STATE']=Path(sys.argv[2]); g['TERMINAL_STATE']=g['STATE']/'terminal'; g['CONFIG']=g['STATE']/'config'; g['IPTABLES']={int(k):v for k,v in json.loads(sys.argv[4]).items()}; g['RESTORE']={int(k):v for k,v in json.loads(sys.argv[5]).items()}; m['serve'](int(sys.argv[3]))"

        def session(persistent, selected=backend):
            helper.CONFIG.write_text(json.dumps({"backend": selected}))
            child = subprocess.Popen([sys.executable, "-c", source, loader.path, temporary, str(uid), json.dumps(helper.IPTABLES), json.dumps(helper.RESTORE)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
            child.stdin.write(json.dumps({"networks": ["127.0.0.1/32"], "persistent": persistent}) + "\n")
            child.stdin.flush()
            assert json.loads(child.stdout.readline()) == {"ok": True, "backend": selected}
            udp(socket.AF_INET, "127.0.0.1", 35000, True)
            # Invalid request must preserve the existing selection.
            child.stdin.write(json.dumps({"networks": ["0.0.0.0/0"], "persistent": persistent}) + "\n")
            child.stdin.flush()
            assert json.loads(child.stdout.readline())["ok"] is False
            udp(socket.AF_INET, "127.0.0.1", 35000, True)
            child.stdin.close()
            assert child.wait(timeout=5) == 0

        session(False)
        udp(socket.AF_INET, "127.0.0.1", 35000, False)
        assert not (helper.STATE / f"{uid}.json").exists()
        session(True)
        udp(socket.AF_INET, "127.0.0.1", 35000, True)
        apply(uid, [])  # simulate a reboot's empty firewall
        helper.restore()
        udp(socket.AF_INET, "127.0.0.1", 35000, True)
        apply(uid, [])
        if backend == "iptables":
            session(True, "nftables")
            assert not helper.iptables_snapshot(uid, 4)["exists"], "old iptables chain survived migration"
            session(True, "iptables")
            result = subprocess.run(nft + ["list", "table", "inet", f"dropship_{uid}"], capture_output=True)
            assert result.returncode != 0, "old nftables table survived migration"
            session(False)
            udp(socket.AF_INET, "127.0.0.1", 35000, False)
        helper.clear(uid)
        udp(socket.AF_INET, "127.0.0.1", 35000, False)
        # GUI Disable must clear an orphan terminal table/state even if its own
        # selection was already empty, preserving other UIDs and foreign tables.
        terminal = f"dropshit_{uid}"
        subprocess.run(nft + ["-f", "-"], input=f"table inet {terminal} {{\n chain output {{\n type filter hook output priority -10; policy accept;\n meta skuid {uid} ip daddr 127.0.0.1 udp dport 35000 reject\n }}\n }}\n", text=True, check=True)
        subprocess.run(nft + ["add", "table", "inet", f"dropshit_{uid + 1}"], check=True)
        (helper.TERMINAL_STATE / f"{uid}.json").write_text(json.dumps({"backend": "nftables", "mode": "allowlist", "region": "gen1", "networks": ["34.88.0.0/16"]}))
        if backend == "iptables":
            for tool in helper.IPTABLES.values():
                subprocess.run([tool, "-N", f"DSHT_{uid}"], check=True)
                subprocess.run([tool, "-A", "OUTPUT", "-p", "udp", "-m", "owner", "--uid-owner", str(uid), "--dport", "12000:64000", "-j", f"DSHT_{uid}"], check=True)
        udp(socket.AF_INET, "127.0.0.1", 35000, True)
        helper.CONFIG.write_text(json.dumps({"backend": backend}))
        child = subprocess.Popen([sys.executable, "-c", source, loader.path, temporary, str(uid), json.dumps(helper.IPTABLES), json.dumps(helper.RESTORE)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        for _ in range(2):
            child.stdin.write(json.dumps({"networks": [], "persistent": True, "disable_all": True}) + "\n")
            child.stdin.flush()
            assert json.loads(child.stdout.readline())["ok"] is True
        child.stdin.close()
        assert child.wait(timeout=5) == 0
        udp(socket.AF_INET, "127.0.0.1", 35000, False)
        assert not (helper.TERMINAL_STATE / f"{uid}.json").exists()
        assert not (helper.STATE / f"{uid}.json").exists()
        for table in [f"dropship_{uid}", terminal]:
            assert subprocess.run(nft + ["list", "table", "inet", table], capture_output=True).returncode != 0
        for table in ["unrelated_test", f"dropshit_{uid + 1}"]:
            assert subprocess.run(nft + ["list", "table", "inet", table], capture_output=True).returncode == 0
        if backend == "iptables":
            for tool in helper.IPTABLES.values():
                assert subprocess.run([tool, "-S", f"DSHT_{uid}"], capture_output=True).returncode == 1
    other_uid = "other UID checked" if backend == "nftables" or mapped_other_uid else "other UID skipped (unmapped)"
    print(f"PASS ({backend}/{variant}): IPv4/IPv6 UDP blocks, port boundaries, TCP unaffected, {other_uid}, replacement, foreign rules preserved, JSON protocol, invalid input rollback, dynamic EOF cleanup, persistent restore and backend migration")


if __name__ == "__main__":
    if "--integration" in sys.argv:
        import argparse
        parser = argparse.ArgumentParser()
        parser.add_argument("--integration", action="store_true")
        parser.add_argument("--backend", choices=["nftables", "iptables"], default="nftables")
        parser.add_argument("--iptables-prefix")
        parser.add_argument("--variant", choices=["nft", "legacy"], default="nft")
        args = parser.parse_args()
        integration(args.backend, args.iptables_prefix, args.variant)
    else:
        unittest.main()
