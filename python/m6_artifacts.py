"""Verify/restore immutable M6 evidence. See docs/m6/ARTIFACT_STORAGE.json."""
import argparse
import hashlib
import json
import subprocess
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STORAGE = ROOT / 'docs/m6/ARTIFACT_STORAGE.json'


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024*1024), b''):h.update(chunk)
    return h.hexdigest()


def require(ok, message):
    if not ok:raise ValueError(message)


def external(record, override):
    if override:return override.expanduser().resolve()
    mount = Path(record['mount_path'])
    require(mount.is_mount(), 'Mermogon must be mounted')
    uuid = subprocess.check_output(['findmnt','-n','-o','UUID','-T',str(mount)],text=True).strip()
    require(uuid == record['volume_uuid'], 'Mermogon UUID mismatch')
    return Path(record['archive_root'])


def verify_bundle(path, row, destination=None):
    require(path.stat().st_size == row['bytes'] and sha(path) == row['sha256'], f'archive integrity: {path}')
    expected={m['name']:m for m in row['members']}
    with tarfile.open(path,'r:') as bundle:
        require(set(bundle.getnames()) == set(expected), f'archive membership: {path}')
        for member in bundle:
            require(member.isfile() and Path(member.name).name == member.name, 'unsafe member')
            entry=expected[member.name]
            require(member.size == entry['bytes'], 'member size')
            h=hashlib.sha256()
            output=None
            if destination is not None:output=(destination/member.name).open('xb')
            try:
                with bundle.extractfile(member) as stream:
                    for chunk in iter(lambda:stream.read(1024*1024), b''):
                        h.update(chunk)
                        if output:output.write(chunk)
            finally:
                if output:output.close()
            require(h.hexdigest()==entry['sha256'], f'member integrity: {member.name}')
            if destination is not None:require(sha(destination/member.name)==entry['sha256'], 'restored integrity')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['verify','unpack'])
    parser.add_argument('destination',type=Path,nargs='?')
    parser.add_argument('--archive-root',type=Path)
    args=parser.parse_args()
    record=json.loads(STORAGE.read_text()); archive=external(record,args.archive_root)
    require(sha(archive/record.get('external_index_path','ARCHIVE_INDEX.json'))==record['external_index_sha256'],'external index integrity')
    if args.command=='unpack':
        require(args.destination is not None,'new destination required')
        args.destination.mkdir(exist_ok=False,parents=True)
    count=0
    for row in record['records']:
        destination=None
        if args.command=='unpack':
            destination=args.destination/row['phase'];destination.mkdir()
        verify_bundle(archive/row['relative_path'],row,destination)
        count+=len(row['members'])
    print(json.dumps(dict(status='PASS',command=args.command,archives=len(record['records']),files=count,archive_root=str(archive))))

if __name__=='__main__':main()
