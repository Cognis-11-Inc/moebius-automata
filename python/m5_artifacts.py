"""Verify or restore versioned M5 evidence from Mermogon.

Authoritative summaries, protocols and checksums live in Git; bulky raw evidence
is external. Use --archive-root to restore a verified copy on another machine.
"""
import argparse
import hashlib
import json
import shutil
import subprocess
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / 'docs/m5/artifacts'
STORAGE = ROOT.parent / 'ARTIFACT_STORAGE.json'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def verify_archive(archive, manifest):
    expected = {row['name']: row for row in manifest}
    with tarfile.open(archive, 'r:gz') as bundle:
        require(set(bundle.getnames()) == set(expected), f'archive membership mismatch: {archive}')
        for member in bundle:
            require(member.isfile() and '/' not in member.name, f'unsafe archive member: {member.name}')
            row = expected[member.name]
            require(member.size == row['bytes'], f'wrong size: {member.name}')
            h = hashlib.sha256()
            with bundle.extractfile(member) as stream:
                for chunk in iter(lambda: stream.read(1024 * 1024), b''):
                    h.update(chunk)
            require(h.hexdigest() == row['sha256'], f'wrong SHA256: {member.name}')


def storage_root(override=None):
    record = json.loads(STORAGE.read_text())
    if override is not None:
        return record, override.expanduser().resolve()
    mount = Path(record['mount_path'])
    require(mount.is_mount(), f'Mermogon is not mounted at {mount}')
    uuid = subprocess.check_output(['findmnt', '-n', '-o', 'UUID', '-T', str(mount)], text=True).strip()
    require(uuid == record['volume_uuid'], f'Mermogon UUID mismatch: {uuid}')
    return record, Path(record['archive_root'])


def verify(override=None):
    record, external = storage_root(override)
    for entry in record['records']:
        path = external / entry['relative_path'] if entry['storage'] == 'mermogon' else Path(__file__).resolve().parents[1] / entry['repository_path']
        require(path.is_file(), f'missing evidence: {path}')
        require(path.stat().st_size == entry['bytes'] and sha(path) == entry['sha256'], f'wrong evidence hash/size: {path}')
        if entry['storage'] == 'mermogon':
            manifest = json.loads((ROOT / Path(entry['relative_path']).parent / 'RAW_MANIFEST.json').read_text())
            verify_archive(path, manifest)
    for entry in record.get('git_document_inventory', []):
        path = Path(__file__).resolve().parents[1] / entry['path']
        require(path.is_file() and path.stat().st_size == entry['bytes'] and sha(path) == entry['sha256'], f'Git metadata mismatch: {path}')
    if record.get('external_index_sha256'):
        require(sha(external / 'ARCHIVE_INDEX.json') == record['external_index_sha256'], 'external archive index hash mismatch')
    print(f"Verified {record['raw_archives']} external archives and Git reproducibility metadata", flush=True)
    return record, external


def unpack(output, override=None):
    require(not output.exists(), f'choose a new directory: {output}')
    record, external = verify(override)
    by_dataset = {Path(e['relative_path']).parent.name: e for e in record['records'] if e['storage'] == 'mermogon'}
    output.mkdir(parents=True)
    for dataset in sorted(p for p in ROOT.iterdir() if p.is_dir()):
        destination = output / dataset.name
        destination.mkdir()
        entry = by_dataset.get(dataset.name)
        if entry:
            archive = external / entry['relative_path']
            manifest = json.loads((dataset / 'RAW_MANIFEST.json').read_text())
            with tarfile.open(archive, 'r:gz') as bundle:
                bundle.extractall(destination, filter='data')
            for row in manifest:
                restored = destination / row['name']
                require(restored.stat().st_size == row['bytes'] and sha(restored) == row['sha256'], f'restored evidence mismatch: {restored}')
        for name in ('results.json', 'protocol.json', 'RAW_MANIFEST.json'):
            source = dataset / name
            if source.exists():
                shutil.copy2(source, destination / name)
        print(dataset.name, flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('verify', 'unpack'))
    parser.add_argument('output', nargs='?', type=Path)
    parser.add_argument('--archive-root', type=Path, help='explicit versioned archive copy; hashes remain mandatory')
    args = parser.parse_args()
    if args.mode == 'unpack':
        if args.output is None:
            parser.error('unpack requires a new output directory')
        unpack(args.output.expanduser().resolve(), args.archive_root)
    else:
        if args.output is not None:
            parser.error('verify does not take an output directory')
        verify(args.archive_root)
