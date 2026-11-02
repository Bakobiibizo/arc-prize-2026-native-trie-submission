"""Offline asset checks. No network, build, credential or Kaggle API operations."""
import json
import pathlib
import platform
import shutil
import struct
import sys
from arc_submission import sha256


def elf_info(path):
    with pathlib.Path(path).open('rb') as f:
        header = f.read(64)
        if len(header) != 64 or header[:6] != b'\x7fELF\x02\x01':
            raise ValueError('Expected little-endian 64-bit ELF worker')
        machine = struct.unpack_from('<H', header, 18)[0]
        offset = struct.unpack_from('<Q', header, 32)[0]
        size, count = struct.unpack_from('<HH', header, 54)
        if size < 4:
            raise ValueError('Malformed ELF program headers')
        interpreter = False
        for i in range(count):
            f.seek(offset + i * size)
            entry = f.read(4)
            if len(entry) != 4:
                raise ValueError('Truncated ELF program headers')
            interpreter |= struct.unpack('<I', entry)[0] == 3
    return dict(machine=machine, has_interpreter=interpreter)


def verify_bundle(bundle):
    bundle = pathlib.Path(bundle).resolve()
    manifest = json.loads((bundle / 'bundle-manifest.json').read_text())
    for name, digest in manifest['files'].items():
        path = (bundle / name).resolve()
        if not path.is_relative_to(bundle) or not path.is_file():
            raise ValueError(f'Invalid/missing bundle file: {name}')
        if sha256(path) != digest:
            raise ValueError(f'Bundle hash mismatch: {name}')
    return manifest


def prepare_worker(bundle, workdir, machine=None):
    if sys.platform != 'linux':
        raise RuntimeError('Only Linux worker artifacts are packaged; no automatic download/build')
    bundle, workdir = pathlib.Path(bundle).resolve(), pathlib.Path(workdir).resolve()
    manifest = verify_bundle(bundle)
    machine = (machine or platform.machine()).lower()
    keys = {'aarch64': 'linux-aarch64', 'arm64': 'linux-aarch64',
            'x86_64': 'linux-x86_64-musl', 'amd64': 'linux-x86_64-musl'}
    key = keys.get(machine)
    if key is None or key not in manifest['workers']:
        raise RuntimeError(f'No offline worker for {machine}; supply an approved compatible artifact')
    item = manifest['workers'][key]
    source = (bundle / item['path']).resolve()
    if not source.is_relative_to(bundle) or item['path'] not in manifest['files']:
        raise ValueError('Worker must be a hashed in-bundle file')
    info = elf_info(source)
    wanted_machine = 183 if key == 'linux-aarch64' else 62
    if info['machine'] != wanted_machine or (key.endswith('musl') and info['has_interpreter']):
        raise ValueError('Offline worker ELF architecture/linkage mismatch')
    workdir.mkdir(parents=True, exist_ok=True)
    destination = workdir / 'native-sparse-worker'
    if destination.exists():
        if sha256(destination) != manifest['files'][item['path']]:
            raise FileExistsError('Existing worker differs; refusing overwrite')
    else:
        shutil.copyfile(source, destination)
    destination.chmod(0o700)
    return destination
