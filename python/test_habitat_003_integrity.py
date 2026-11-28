"""Falsification checks for independent checkpoint auditor, not numerical formulas."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from habitat_003_qualification import audit_checkpoint

class AuditTests(unittest.TestCase):
    def setUp(self):
        self.state={'state':{'config':{'roots':[1,1,1],'h':1.,'dt':.1,'capacity':8},'cycle':2,'cells':{'0':{'Fv':[20.,0.,0.,0.,0.,0.,0.,0.]}},'bodies':[],'ledger':{'initial':[20.,0.,0.,0.,0.,0.,0.,0.],'wall_scalar':[[0.]*6 for _ in range(4)],'gravity_impulse':[0.]*3,'walls_impulse':[0.]*3,'hydrostatic_support':[0.]*3}}}
        self.temp=tempfile.TemporaryDirectory();self.path=Path(self.temp.name)/'checkpoint.json';self.addCleanup(self.temp.cleanup)
    def audit(self):self.path.write_text(json.dumps(self.state));return audit_checkpoint(self.path)
    def test_physical_time_and_volume(self):
        result=self.audit();self.assertEqual(result['physical_time_s'],.2);self.assertEqual(result['volume_m3'],1.)
    def test_overlapping_inventory_rejected(self):
        self.state['state']['cells']['1']=copy.deepcopy(self.state['state']['cells']['0'])
        with self.assertRaises(AssertionError):self.audit()
    def test_unaccounted_mass_rejected(self):
        self.state['state']['cells']['0']['Fv'][0]+=1e-5
        with self.assertRaises(AssertionError):self.audit()
    def test_unaccounted_momentum_rejected(self):
        self.state['state']['cells']['0']['Fv'][1]=1e-5
        with self.assertRaises(AssertionError):self.audit()
    def test_negative_scalar_rejected(self):
        self.state['state']['cells']['0']['Fv'][4]=-1e-5
        with self.assertRaises(AssertionError):self.audit()
if __name__=='__main__':unittest.main()
