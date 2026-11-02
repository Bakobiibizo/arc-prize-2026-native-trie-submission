"""Build hash-pinned offline submission assets. Never uploads, publishes, logs in or submits."""
import argparse
import hashlib
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'contest'))
from arc_submission import sha256, write_json
from notebook_support import elf_info


def build(output_dir):
    output_dir = pathlib.Path(output_dir).resolve()
    if output_dir.exists():
        raise FileExistsError('Bundle output exists; preserve it and use a new directory')
    package = ROOT / 'experiments/native-trie'
    if output_dir.is_relative_to(ROOT / 'contest') or output_dir.is_relative_to(package):
        raise ValueError('Output cannot be inside packaged source directories')
    sources = [p for p in package.rglob('*') if p.is_file() and '__pycache__' not in p.parts]
    sources += [ROOT / 'rust-toolchain.toml', ROOT / 'references/arc-transfer/arc/native/curriculum.ts']
    sources += [p for p in (ROOT / 'contest').rglob('*')
                if p.is_file() and '__pycache__' not in p.parts
                and 'private-data' not in p.parts and 'staging' not in p.parts]
    pinned = {str(p.relative_to(ROOT)): sha256(p) for p in sources}
    offline_env = os.environ.copy()
    offline_env['RUSTUP_AUTO_INSTALL'] = '0'
    output_dir.mkdir(parents=True)
    commands = []

    def execute(label, argv, env=None):
        row = dict(label=label, argv=argv)
        commands.append(row)
        with (output_dir / f'{label}.stdout').open('wb') as so, (output_dir / f'{label}.stderr').open('wb') as se:
            result = subprocess.run(argv, cwd=ROOT, env=env or offline_env, stdout=so, stderr=se)
        row['status'] = result.returncode
        write_json(output_dir / f'{label}.json', row)
        if result.returncode:
            raise RuntimeError(f'{label} failed; logs retained, no retry/download')

    base = ['cargo', 'build', '--offline', '--locked', '--release', '--manifest-path',
            'experiments/native-trie/Cargo.toml', '--target-dir', 'target/native-trie', '--bin', 'sparse_probe']
    execute('build-host', base)
    rust_version = subprocess.check_output(['rustc', '-vV'], cwd=ROOT, env=offline_env, text=True)
    host = next(line.split(': ', 1)[1] for line in rust_version.splitlines() if line.startswith('host:'))
    sysroot = pathlib.Path(subprocess.check_output(['rustc', '--print', 'sysroot'], cwd=ROOT, env=offline_env, text=True).strip())
    linker = sysroot / 'lib/rustlib' / host / 'bin/rust-lld'
    if not linker.is_file():
        raise RuntimeError('Rust bundled cross linker missing; no automatic installation')
    env = offline_env.copy()
    env['CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER'] = str(linker)
    execute('build-x86-musl', base + ['--target', 'x86_64-unknown-linux-musl'], env)
    bundle = output_dir / 'native-arc-submission'
    bundle.mkdir()
    # Complete package plus the external compile-time fixture; no benchmark,
    # archives, credentials or LDGR data copied. Sources pinned BEFORE builds.
    for path in sources:
        target = bundle / path.relative_to(ROOT)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
    workers = {}
    paths = {'linux-x86_64-musl': ROOT / 'target/native-trie/x86_64-unknown-linux-musl/release/sparse_probe'}
    if platform.machine().lower() in ('aarch64', 'arm64'):
        paths['linux-aarch64'] = ROOT / 'target/native-trie/release/sparse_probe'
    for key, source in paths.items():
        target = bundle / 'workers' / key / 'sparse_probe'
        target.parent.mkdir(parents=True)
        shutil.copyfile(source, target)
        target.chmod(0o700)
        info = elf_info(target)
        if key == 'linux-x86_64-musl' and (info['machine'] != 62 or info['has_interpreter']):
            raise ValueError('Cross-built worker is not static x86_64 ELF')
        if key == 'linux-aarch64' and info['machine'] != 183:
            raise ValueError('Host worker is not ARM64 ELF')
        workers[key] = dict(path=str(target.relative_to(bundle)), elf=info,
                            kaggle_execution_verified=False,
                            cross_compiled=key == 'linux-x86_64-musl' and 'x86_64' not in host)
    (bundle / 'PRE-SUBMISSION.txt').write_text(
        'Private code-submission asset prepared locally. Upload, publication, and submission require a separate explicit operator instruction.\n'
    )
    for name, digest in pinned.items():
        if sha256(ROOT / name) != digest:
            raise RuntimeError(f'Source changed during packaging: {name}')
    manifest = dict(pre_submission=True, source_revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                    rustc=rust_version, commands=commands, workers=workers,
                    files={str(p.relative_to(bundle)): sha256(p) for p in sorted(bundle.rglob('*')) if p.is_file()},
                    native_workspace_limit=286720000, learner='sparse_probe --cyclic-lattice-actions',
                    kaggle_runtime_verified=False, uploaded=False, published=False, submitted=False,
                    scope='Offline private code-submission preparation; no upload, publication, submission, or hidden-output accuracy evidence')
    write_json(bundle / 'bundle-manifest.json', manifest)
    archive_path = output_dir / 'native-arc-submission.zip'
    with zipfile.ZipFile(archive_path, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(bundle.rglob('*')):
            if path.is_file():
                archive.write(path, str(path.relative_to(output_dir)))
    write_json(output_dir / 'bundle-receipt.json', dict(archive_sha256=sha256(archive_path),
                manifest_sha256=sha256(bundle / 'bundle-manifest.json'), source_revision=manifest['source_revision'],
                archive_bytes=archive_path.stat().st_size, uploaded=False, published=False, submitted=False))
    return bundle


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', required=True)
    args = parser.parse_args()
    print(build(args.output_dir))
