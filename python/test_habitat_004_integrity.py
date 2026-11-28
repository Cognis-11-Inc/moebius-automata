"""Independent force-ledger/ownership and analytic-initialization falsification."""
import json
from pathlib import Path
import tempfile
import unittest
from habitat_004_qualification import audit,analytic_errors,recipes

class AuditTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.p=Path(self.temp.name)/'checkpoint.json';self.s={'config':{'roots':[1,1,1],'h':1.,'dt':.1,'capacity':8},'cycle':0,'cells':{'0':{'Fv':[20.,1e-6,0.,0.,0.,0.,0.,0.]}},'bodies':[],'ledger':{'initial':[20.,0.,0.,0.,0.,0.,0.,0.],'wall_scalar':[[0.]*6 for _ in range(4)],'gravity_impulse':[0.]*3,'walls_impulse':[0.]*3,'hydrostatic_support':[0.]*3,'manufactured_impulse':[1e-6,0.,0.]}}
    def run_audit(self):self.p.write_text(json.dumps({'state':self.s}));return audit(self.p)
    def test_external_force_is_accounted(self):self.assertEqual(self.run_audit()['residual'][1],0.)
    def test_removed_external_force_is_rejected(self):
        self.s['ledger']['manufactured_impulse'][0]=0.
        with self.assertRaises(AssertionError):self.run_audit()
    def test_duplicate_leaf_rejected(self):
        self.s['cells']['1']={'Fv':[2.5,0.,0.,0.,0.,0.,0.,0.]}
        with self.assertRaises(AssertionError):self.run_audit()
    def test_unit_box_scalar_average(self):
        spec=next(x['spec']for x in recipes()if x['name']=='Diffusion3-8-reference-uniform');spec['config'].update(roots=[1]*3,h=1.);self.s.update(config=spec['config']);self.s['cells']['0']['Fv']=[20.,0.,0.,0.,1e-4,0.,0.,0.];self.p.write_text(json.dumps({'state':self.s}));errors=analytic_errors(spec,self.p);self.assertLess(errors['scalar_L1_normalized'],1e-14)
    def test_fixed_three_dimensions(self):
        for case in recipes():
            if case['name'].startswith(('Diffusion3-','Diagonal-','Shear3-','Oblique-','Vortex-')):self.assertEqual([n*case['spec']['config']['h']for n in case['spec']['config']['roots']],[1.]*3)
if __name__=='__main__':unittest.main()
