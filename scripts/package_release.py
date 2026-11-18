#!/usr/bin/env python3
"""Deterministic, offline Arcxiom prerelease archive packager.

Used by both pull-request qualification and the maintainer-only release job.
Does not tag, publish, upload, or contact external services.
"""

import argparse
import hashlib
import pathlib
import platform
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
RELEASE_FILES = ("LICENSE", "spec/archyaml-v0.1-compat.md", "docs/releases/v0.2.0a1.md")


def create_archive(output: pathlib.Path) -> pathlib.Path:
    system = platform.system().lower()
    machine = platform.machine().lower()
    machine = {"amd64": "x86_64", "x64": "x86_64", "aarch64": "arm64"}.get(machine, machine)
    if system not in {"linux", "darwin", "windows"}:
        raise ValueError(f"Unsupported release platform: {system}")
    binary_name = "arcxiom.exe" if system == "windows" else "arcxiom"
    binary = ROOT / "rust" / "arcxiom-ir" / "target" / "release" / binary_name
    if not binary.is_file():
        raise FileNotFoundError(f"Native CLI missing: {binary}")
    for name in RELEASE_FILES:
        if not (ROOT / name).is_file():
            raise FileNotFoundError(f"Required release metadata missing: {name}")
    output.mkdir(parents=True, exist_ok=True)
    archive = output / f"arcxiom-v0.2.0a1-{system}-{machine}.zip"
    digest_file = output / f"SHA256SUMS.{system}"
    if archive.exists() or digest_file.exists():
        raise FileExistsError("Release archive/checksum already exists; refuse overwrite")
    members = [(binary_name, binary.read_bytes(), 0o100755)]
    members.extend((name, (ROOT / name).read_bytes(), 0o100644) for name in RELEASE_FILES)
    with zipfile.ZipFile(archive, "x", compression=zipfile.ZIP_DEFLATED) as z:
        for name, data, mode in members:
            entry = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            entry.create_system = 3
            entry.external_attr = mode << 16
            entry.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(entry, data)
    checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
    digest_file.write_text(f"{checksum}  {archive.name}\n", encoding="ascii")
    with zipfile.ZipFile(archive) as z:
        assert z.testzip() is None, "ZIP member CRC failure"
        assert z.namelist() == [name for name, _, _ in members], "Unexpected ZIP contents"
        for name, data, _ in members:
            assert z.read(name) == data, f"ZIP member differs: {name}"
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == checksum
    print(f"{archive.name} sha256:{checksum}")
    return archive


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", default="dist", type=pathlib.Path)
    options = parser.parse_args()
    create_archive(options.output)


if __name__ == "__main__":
    main()
