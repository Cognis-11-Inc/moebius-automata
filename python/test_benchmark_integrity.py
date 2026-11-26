import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import subprocess
from benchmark_integrity import build_capture, verify_artifact, validate_pair, validate_run_pair, run_snapshot, sha, source_manifest

class IntegrityTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def artifact(self, name):
        p = self.root / name
        p.mkdir()
        identity = {'schema':1,'sources_blake3':{'../../Cargo.lock':'locked'},'settings':{'rustc':'fixed'},'features':{}}
        exe = p/'probe'
        exe.write_text('#!/usr/bin/env python3\nimport json\nprint('+repr(json.dumps(identity))+')\n')
        exe.chmod(0o755)
        (p/'build.json').write_text(json.dumps({'status':'BUILT','executable':'probe','sha256':sha(exe),'identity':identity,'instrumentation':'ordinary','source_sha256':{},'dependency_signature':{'test':'same'}}))
        return p

    def test_changed_executable_rejected(self):
        p=self.artifact('a')
        verify_artifact(p)
        (p/'probe').write_text('stale binary')
        with self.assertRaisesRegex(ValueError,'changed executable'):verify_artifact(p)

    def test_embedded_identity_mismatch_rejected(self):
        p=self.artifact('a');r=json.loads((p/'build.json').read_text());r['identity']['schema']=2;(p/'build.json').write_text(json.dumps(r))
        with self.assertRaisesRegex(ValueError,'embedded'):verify_artifact(p)

    def test_mixed_instrumentation_rejected(self):
        a=self.artifact('a');b=self.artifact('b');r=json.loads((b/'build.json').read_text());r['instrumentation']='allocation';(b/'build.json').write_text(json.dumps(r))
        with self.assertRaisesRegex(ValueError,'Instrumented'):validate_pair(a,b)

    def test_changed_dependency_features_rejected(self):
        a=self.artifact('a');b=self.artifact('b');r=json.loads((b/'build.json').read_text())
        r['dependency_signature']={'test':'different'};(b/'build.json').write_text(json.dumps(r))
        with self.assertRaisesRegex(ValueError,'Dependency features'):validate_pair(a,b)

    def test_shared_build_rejected(self):
        a=self.artifact('a')
        with self.assertRaisesRegex(ValueError,'isolated'):validate_pair(a,a)

    def test_source_changed_after_build_rejected(self):
        p=self.artifact('a')
        with patch('benchmark_integrity.source_manifest',return_value={'new.rs':'changed'}):
            with self.assertRaisesRegex(ValueError,'Source changed'):verify_artifact(p,self.root)

    def test_incomplete_or_mismatched_runs_rejected(self):
        a=self.artifact('a');b=self.artifact('b')
        left=self.root/'left';right=self.root/'right'
        base={'artifact':str(a),'input_sha256':'same','requested_ticks':3,
              'completed_ticks':3,'mode':'ordinary','stack_sampling':False,'status':'COMPLETE'}
        left.with_suffix('.run.json').write_text(json.dumps(base))
        for change,error in [({'completed_ticks':2},'completed_ticks'),
                              ({'requested_ticks':4},'requested_ticks'),
                              ({'input_sha256':'different'},'input_sha256'),
                              ({'status':'INCOMPLETE'},'Incomplete'),
                              ({'stack_sampling':True},'stack_sampling')]:
            other=dict(base,artifact=str(b));other.update(change)
            right.with_suffix('.run.json').write_text(json.dumps(other))
            with self.assertRaisesRegex(ValueError,error):validate_run_pair(left,right)

    def test_timeout_is_explicit_and_cannot_be_complete(self):
        snapshot=self.root/'snapshot.json';snapshot.write_text('{}')
        output=self.root/'output'
        artifact={'executable':'probe','instrumentation':'habitat_probe'}
        with patch('benchmark_integrity.verify_artifact',return_value=artifact), patch(
                'benchmark_integrity.subprocess.run',side_effect=subprocess.TimeoutExpired(['probe'],0.1)):
            receipt=run_snapshot(self.root,snapshot,output,3600,timeout_seconds=0.1)
        self.assertEqual(receipt['status'],'TIMED_OUT')
        self.assertIsNone(receipt['completed_ticks'])
        self.assertEqual(receipt,json.loads(output.with_suffix('.run.json').read_text()))

    def test_compile_failure_never_captures_stale_binary(self):
        repo=self.root/'repo';(repo/'crates/moebius-morph').mkdir(parents=True)
        for name in ['Cargo.lock','Cargo.toml','crates/moebius-morph/Cargo.toml']:(repo/name).write_text('')
        out=self.root/'build'
        def failed_compile(*args,**kwargs):
            stale=out/'target/release/examples/habitat_probe'
            stale.parent.mkdir(parents=True);stale.write_text('stale executable')
            raise subprocess.CalledProcessError(1,['cargo'])
        with patch('benchmark_integrity.subprocess.run',side_effect=failed_compile):
            with self.assertRaises(subprocess.CalledProcessError):build_capture(repo,out)
        self.assertEqual(json.loads((out/'build.json').read_text())['status'],'BUILD_FAILED')
        self.assertFalse((out/'habitat_probe.binary').exists())

if __name__=='__main__':unittest.main()
