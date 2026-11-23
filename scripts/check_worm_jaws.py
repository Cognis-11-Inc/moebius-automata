"""Isolate real sculpt shape-key motion under native jaw telemetry.
Usage: python3 scripts/check_worm_jaws.py JAW_PROBE_JSONL
Retired bone-only audit is preserved with Worm_A v2 evidence.
"""
import json,math,sys
from pathlib import Path
from rig_worm_a import read,values,OUT
D,B=read(OUT/'Worm_A_rigged.glb');P=D['meshes'][0]['primitives'][0]
targets=[values(D,B,t['POSITION'])[0] for t in P['targets']]
rows=[json.loads(s) for s in Path(sys.argv[1]).read_text().splitlines()]
report={'scope':'Actual sculpt morph displacements in head-relative frame; no body movement credited.','controls':{}}
for control in ['selected','untrained']:
 sample=[r for r in rows if r['control']==control]
 first=[max(0,min(1,(768-a)/1024)) for a in sample[0]['angles']];peak=0
 for row in sample:
  weights=[max(0,min(1,(768-a)/1024)) for a in row['angles']]
  for a,b in zip(*targets):peak=max(peak,math.sqrt(sum(((weights[0]-first[0])*a[q]+(weights[1]-first[1])*b[q])**2 for q in range(3))))
 report['controls'][control]={'samples':len(sample),'max_morph_displacement':peak,'jaw_angle_range':[[min(r['angles'][j] for r in sample),max(r['angles'][j] for r in sample)] for j in range(2)]}
assert report['controls']['selected']['max_morph_displacement']>.03
print(json.dumps(report,indent=2))
