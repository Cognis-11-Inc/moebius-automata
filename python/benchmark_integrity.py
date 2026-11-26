"""Owned, isolated builds and verified execution for existing morph recorders.
No numerical authority: simulation remains in ShipRuntime.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tomllib


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1048576), b''):
            h.update(block)
    return h.hexdigest()


def source_manifest(repo):
    repo = Path(repo)
    paths = list((repo / 'crates').rglob('*.rs'))
    paths += [repo / name for name in ['Cargo.lock', 'Cargo.toml']]
    paths += list((repo / 'crates').glob('*/Cargo.toml'))
    paths += list((repo / '.cargo').glob('*.toml')) if (repo / '.cargo').exists() else []
    return {str(p.relative_to(repo)): sha(p) for p in sorted(paths)}


def verify_artifact(directory, repo=None):
    directory = Path(directory)
    receipt = json.loads((directory / 'build.json').read_text())
    if 'source_archive_sha256' in receipt and sha(directory/'source.tar.gz') != receipt['source_archive_sha256']:
        raise ValueError('Source archive changed')
    binary = directory / receipt['executable']
    if receipt['status'] != 'BUILT' or sha(binary) != receipt['sha256']:
        raise ValueError('Unbuilt or changed executable')
    identity = json.loads(subprocess.check_output([str(binary), '--build-identity'], text=True))
    if identity != receipt['identity'] or identity.get('schema') != 1:
        raise ValueError('Missing or mismatched embedded build identity')
    if repo is not None and source_manifest(repo) != receipt['source_sha256']:
        raise ValueError('Source changed after build; stale comparison rejected')
    return receipt


def build_capture(repo, directory, example='habitat_probe', features=(), package='moebius-morph', kind='example', seed_target=None, target_directory=None):
    repo, directory = Path(repo).resolve(), Path(directory).resolve()
    directory.mkdir(parents=True, exist_ok=False)
    if kind not in ('example','bin'): raise ValueError('Unsupported Cargo target kind')
    target = Path(target_directory).resolve() if target_directory else directory / 'target'
    if target.exists(): raise ValueError('Build target directory must be new and isolated')
    if seed_target:
        subprocess.run(['cp','--reflink=auto','-a',str(Path(seed_target).resolve()),str(target)],check=True)
        # Preserve reusable dependency caches, but force this package to rebuild.
        subprocess.run(['cargo','clean','--manifest-path',str(repo/'Cargo.toml'),'--target-dir',str(target),'-p',package],cwd=repo,check=True,stdout=subprocess.DEVNULL)
    command = ['cargo', 'build', '--manifest-path', str(repo / 'Cargo.toml'), '--target-dir', str(target),
               '--release', '--offline', '--locked', '-p', package, '--'+kind, example]
    if features:
        command += ['--features', ','.join(features)]
    before = source_manifest(repo)
    source_archive = directory / 'source.tar.gz'
    with tarfile.open(source_archive, 'w:gz') as archive:
        for name in before: archive.add(repo / name, arcname=name)
    receipt = {'source_archive_sha256':sha(source_archive), 'status': 'BUILD_FAILED', 'command': command, 'source_sha256': before,
               'executable': example + '.binary', 'instrumentation': example, 'target_kind':kind, 'package':package, 'seed_target':str(seed_target) if seed_target else None,
               'build_environment': {k: os.environ.get(k, '') for k in ['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC_WRAPPER']}}
    try:
        with (directory / 'build.log').open('w') as log:
            subprocess.run(command, cwd=repo, stdout=log, stderr=subprocess.STDOUT, check=True)
        if source_manifest(repo) != before:
            raise ValueError('Source changed during compilation')
        captured = directory / receipt['executable']
        shutil.copy2(target / ('release/examples' if kind == 'example' else 'release') / example, captured)
        identity = json.loads(subprocess.check_output([str(captured), '--build-identity'], text=True))
        if identity.get('schema') != 1 or not identity.get('sources_blake3') or not identity.get('settings'):
            raise ValueError('Unidentified executable')
        receipt.update(status='BUILT', sha256=sha(captured), identity=identity)
    finally:
        (directory / 'build.json').write_text(json.dumps(receipt, indent=2) + '\n')
    verify_artifact(directory, repo)
    return directory / receipt['executable']


def dependency_signature(directory, receipt):
    """Read dependency/feature/profile contracts from the captured build source.
    Ignore inactive feature declarations of the selected package; they cannot
    affect that executable. Cargo.lock identity is checked separately.
    """
    archive=Path(directory)/'source.tar.gz'
    if not archive.exists():return receipt.get('dependency_signature')
    result={}
    active=set(receipt['identity']['features'])
    with tarfile.open(archive) as source:
        for member in source.getmembers():
            if not member.name.endswith('Cargo.toml'):continue
            manifest=tomllib.loads(source.extractfile(member).read().decode())
            features=manifest.get('features',{})
            if manifest.get('package',{}).get('name')==receipt.get('package','moebius-morph'):
                features={k:v for k,v in features.items() if 'CARGO_FEATURE_'+k.upper().replace('-','_') in active}
            result[member.name]={k:manifest.get(k,{}) for k in ['dependencies','build-dependencies','dev-dependencies','target','patch','replace','profile']}
            result[member.name]['features']=features
            if 'workspace' in manifest:
                result[member.name]['workspace_dependencies']=manifest['workspace'].get('dependencies',{})
                result[member.name]['workspace_package']=manifest['workspace'].get('package',{})
    return result


def validate_pair(left, right):
    a, b = verify_artifact(left), verify_artifact(right)
    if Path(left).resolve() == Path(right).resolve():
        raise ValueError('Comparison requires isolated builds')
    if a.get('build_environment', {}) != b.get('build_environment', {}):
        raise ValueError('Build environments differ')
    for key in ['settings', 'features']:
        if a['identity'][key] != b['identity'][key]:
            raise ValueError('Compiler settings/features differ')
    if a['instrumentation'] != b['instrumentation']:
        raise ValueError('Instrumented and ordinary measurements cannot be paired')
    if a['identity']['sources_blake3']['../../Cargo.lock'] != b['identity']['sources_blake3']['../../Cargo.lock']:
        raise ValueError('Dependencies differ')
    da,db=dependency_signature(left,a),dependency_signature(right,b)
    if da is None or db is None or da!=db:
        raise ValueError('Dependency features/profiles differ or are unidentified')
    if a['source_sha256'] != b['source_sha256'] and a['sha256'] == b['sha256']:
        raise ValueError('Different sources supplied identical executable')


def run_snapshot(directory, snapshot, output, ticks, timeout_seconds=None, recorder_args=(), additional_inputs=()):
    """One explicit saved world; incomplete execution is never a valid pair."""
    directory, snapshot, output = map(lambda p: Path(p).resolve(), (directory, snapshot, output))
    artifact = verify_artifact(directory)
    binary = directory / artifact['executable']
    mode = artifact['instrumentation']
    if mode == 'coordinate_domain_bench':
        command = [str(binary), str(snapshot), str(ticks), str(output)]
    elif mode == 'habitat_probe':
        command = [str(binary), str(output), str(ticks), '--resume', str(snapshot)]
    else:
        raise ValueError('Unsupported recorder')
    command += list(recorder_args)
    extra_hashes = {str(Path(p).resolve()):sha(p) for p in additional_inputs}
    receipt = {'status':'FAILED', 'artifact':str(directory), 'input_sha256':sha(snapshot),
               'requested_ticks':ticks, 'mode':mode, 'command':command, 'timeout_seconds':timeout_seconds, 'recorder_args':list(recorder_args), 'additional_inputs':extra_hashes,
               'stack_sampling':bool(os.environ.get('MOEBIUS_ALLOCATION_STACK_DIR'))}
    try:
        with output.with_suffix('.log').open('w') as log:
            try:
                result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, timeout=timeout_seconds)
            except subprocess.TimeoutExpired:
                receipt.update(status='TIMED_OUT', completed_ticks=None)
                return receipt
        receipt['exit_code'] = result.returncode
        verify_artifact(directory)
        if any(sha(p) != h for p,h in extra_hashes.items()):
            raise ValueError('Additional input changed during execution')
        if sha(snapshot) != receipt['input_sha256']:
            raise ValueError('Input changed during execution')
        if result.returncode:
            raise RuntimeError('Recorder failed; comparison rejected')
        report = json.loads((output/'report.json').read_text())
        completed = report.get('ticks_advanced', report.get('ticks'))
        receipt['completed_ticks'] = completed
        receipt['status'] = 'COMPLETE' if completed == ticks and report.get('stop','completed') == 'completed' else 'INCOMPLETE'
        if mode == 'habitat_probe' and (report.get('status') != 'PASS' or report.get('replay_equal') is not True or report.get('material_gap') != 0):
            receipt['status'] = 'INVALID_RESULT'
        if mode == 'coordinate_domain_bench':
            receipt['digests'] = [row['digest'] for row in report['tick_rows']]
    finally:
        output.with_suffix('.run.json').write_text(json.dumps(receipt, indent=2)+'\n')
    return receipt


def validate_run_pair(left, right):
    left, right = Path(left), Path(right)
    a, b = [json.loads(p.with_suffix('.run.json').read_text()) for p in [left,right]]
    validate_pair(a['artifact'], b['artifact'])
    for key in ['input_sha256','requested_ticks','completed_ticks','mode','stack_sampling','timeout_seconds','recorder_args','additional_inputs']:
        if a.get(key) != b.get(key): raise ValueError('Mismatched execution: '+key)
    if a['status'] != 'COMPLETE' or b['status'] != 'COMPLETE' or a['stack_sampling']:
        raise ValueError('Incomplete or stack-sampled comparison')
    for name in ['initial.json','final.json','ticks.jsonl']:
        if name == 'initial.json' and a['mode'] == 'coordinate_domain_bench':continue
        if name == 'ticks.jsonl' and a['mode'] == 'coordinate_domain_bench':
            if a['digests'] != b['digests']:raise ValueError('Tick digests differ')
        elif (left/name).read_bytes() != (right/name).read_bytes():raise ValueError('Authoritative evidence differs: '+name)


def run_fixture(directory, snapshot, fixture, output, repeats=3, detailed_trace=False, timeout_seconds=120):
    """Authenticated isolated query run; traced and untraced timings stay distinct."""
    directory, snapshot, fixture, output = [Path(p).resolve() for p in [directory,snapshot,fixture,output]]
    artifact=verify_artifact(directory)
    if artifact['instrumentation'] != 'collision_fixture':raise ValueError('Wrong fixture recorder')
    command=[str(directory/artifact['executable']),str(snapshot),str(fixture),str(output),str(repeats)]
    if detailed_trace:command.append('--trace')
    receipt={'status':'FAILED','artifact':str(directory),'snapshot_sha256':sha(snapshot),
             'fixture_sha256':sha(fixture),'requested_repeats':repeats,'detailed_trace':detailed_trace,
             'command':command,'timeout_seconds':timeout_seconds}
    try:
        with output.with_suffix('.log').open('w') as log:
            try:result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=timeout_seconds)
            except subprocess.TimeoutExpired:
                receipt['status']='TIMED_OUT';return receipt
        receipt['exit_code']=result.returncode
        verify_artifact(directory)
        if sha(snapshot)!=receipt['snapshot_sha256'] or sha(fixture)!=receipt['fixture_sha256']:
            raise ValueError('Fixture input changed during execution')
        if result.returncode:raise RuntimeError('Fixture reproduction failed')
        report=json.loads((output/'report.json').read_text())
        receipt['completed_repeats']=len(report['rows'])
        receipt['status']='COMPLETE' if report['exact'] and len(report['rows'])==repeats else 'INCOMPLETE'
        receipt['report_sha256']=sha(output/'report.json')
    finally:output.with_suffix('.run.json').write_text(json.dumps(receipt,indent=2)+'\n')
    return receipt


def validate_fixture_pair(left,right):
    left,right=Path(left),Path(right)
    a,b=[json.loads(p.with_suffix('.run.json').read_text()) for p in [left,right]]
    validate_pair(a['artifact'],b['artifact'])
    for key in ['snapshot_sha256','fixture_sha256','requested_repeats','completed_repeats','detailed_trace']:
        if a.get(key)!=b.get(key):raise ValueError('Mismatched fixture execution: '+key)
    if a['status']!='COMPLETE' or b['status']!='COMPLETE':raise ValueError('Incomplete fixture comparison')
    for path,receipt in [(left,a),(right,b)]:
        if sha(path/'report.json')!=receipt['report_sha256']:raise ValueError('Changed fixture report')
    x,y=[json.loads((p/'report.json').read_text()) for p in [left,right]]
    for u,v in zip(x['rows'],y['rows']):
        if u['hit']!=v['hit']:raise ValueError('Fixture hits differ')
        if a['detailed_trace']:
            def samples(row):return [{k:s[k] for k in ['lo_bits','hi_bits','mid_bits','limit_bits','roundoff_bits','within']} for s in row['trace'] if s['phase']=='distance']
            if samples(u)!=samples(v):raise ValueError('Intermediate distance decisions differ')
