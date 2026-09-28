"""The local-server client verifies TLS unless told not to.

A real HTTPS server on 127.0.0.1 with a self-signed certificate: the default
client must refuse it, a client given that certificate (`ca_cert`) must reach
it, and an explicitly insecure client reaches it with a warning.
"""
from __future__ import annotations

import asyncio
import datetime
import http.server
import ipaddress
import logging
import ssl
import threading
from pathlib import Path

import httpx
import pytest
from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.x509.oid import NameOID

from remarkable.local import LocalServerClient


def _self_signed(directory: Path) -> tuple[Path, Path]:
    key = ec.generate_private_key(ec.SECP256R1())
    name = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, "rm-local-test")])
    now = datetime.datetime.now(datetime.timezone.utc)
    cert = (
        x509.CertificateBuilder()
        .subject_name(name)
        .issuer_name(name)
        .public_key(key.public_key())
        .serial_number(x509.random_serial_number())
        .not_valid_before(now - datetime.timedelta(minutes=1))
        .not_valid_after(now + datetime.timedelta(days=1))
        .add_extension(
            x509.SubjectAlternativeName([x509.IPAddress(ipaddress.ip_address("127.0.0.1"))]),
            critical=False,
        )
        .add_extension(x509.BasicConstraints(ca=True, path_length=None), critical=True)
        .sign(key, hashes.SHA256())
    )
    cert_path, key_path = directory / "cert.pem", directory / "key.pem"
    cert_path.write_bytes(cert.public_bytes(serialization.Encoding.PEM))
    key_path.write_bytes(
        key.private_bytes(
            serialization.Encoding.PEM,
            serialization.PrivateFormat.PKCS8,
            serialization.NoEncryption(),
        )
    )
    return cert_path, key_path


class _Health(http.server.BaseHTTPRequestHandler):
    def do_GET(self) -> None:  # noqa: N802
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"ok")

    def log_message(self, *args: object) -> None:
        pass


@pytest.fixture
def https_server(tmp_path: Path):
    cert, key = _self_signed(tmp_path)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), _Health)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(cert, key)
    server.socket = context.wrap_socket(server.socket, server_side=True)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield f"https://127.0.0.1:{server.server_address[1]}", cert
    server.shutdown()


def _get(client: LocalServerClient, url: str) -> httpx.Response:
    async def run() -> httpx.Response:
        async with client:
            return await client.http.get(f"{url}/health")

    return asyncio.run(run())


def test_the_default_client_refuses_an_unverified_certificate(https_server) -> None:
    url, _ = https_server
    with pytest.raises(httpx.ConnectError):
        _get(LocalServerClient(url), url)


def test_a_trusted_self_signed_certificate_is_accepted(https_server) -> None:
    url, cert = https_server
    assert _get(LocalServerClient(url, ca_cert=cert), url).status_code == 200


def test_insecure_is_explicit_and_warns(https_server, caplog) -> None:
    url, _ = https_server
    with caplog.at_level(logging.WARNING, logger="remarkable.local"):
        assert _get(LocalServerClient(url, verify_ssl=False), url).status_code == 200
    assert "verification is OFF" in caplog.text


def test_trusting_a_certificate_and_skipping_verification_conflict(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="mutually exclusive"):
        LocalServerClient("https://127.0.0.1:1", ca_cert=tmp_path / "ca.pem", verify_ssl=False)


def test_the_cli_rejects_ca_cert_with_insecure(monkeypatch, capsys) -> None:
    from remarkable import cli

    for command in ("pair", "sync"):
        monkeypatch.setattr(
            "sys.argv",
            ["remarkable", command, "--local", "https://x", "--ca-cert", "ca.pem", "--insecure"],
        )
        with pytest.raises(SystemExit) as exit:
            cli.main()
        assert exit.value.code == 2
        assert "not allowed with argument" in capsys.readouterr().err
