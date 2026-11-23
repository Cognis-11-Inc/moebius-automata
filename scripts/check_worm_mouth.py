# Historical v3 prototype, rejected by user; not used by the current rig.
"""Verify real lip triangles occlude a frontal aperture when controlled jaws close.
Usage: python3 scripts/check_worm_mouth.py EXERCISE_JSONL [NATIVE_JAW_JSONL]
Render geometry only: no claim about triangle-level feeding collision.
"""
import json,math,sys
from pathlib import Path
from rig_worm_a import read,values,OUT
D,B=read(OUT/'Worm_A_rigged.glb');meta=json.loads((OUT/'rig.json').read_text());oral=meta['oral_assembly'];bind=meta['bind_positions']
p=D['meshes'][0]['primitives'][oral['lip_primitive']]
points=values(D,B,p['attributes']['POSITION'])[0];joints=values(D,B,p['attributes']['JOINTS_0'])[0];indices=[x[0] for x in values(D,B,p['indices'])[0]]
seams={side:{r:i for s,r,i in oral['lip_seam_vertices'] if s==side} for side in [-1,1]}
def skin(angles):
 out=[]
 for p,j in zip(points,joints):
  bone=j[0];a=angles[bone-12]/1024*(-1 if bone==12 else 1);c,s=math.cos(a),math.sin(a);x,y=p[0]-bind[bone][0],p[1]-bind[bone][1]
  out.append([bind[bone][0]+c*x-s*y,bind[bone][1]+s*x+c*y,p[2]])
 return out
def ray_blocked(ps,x,z):
 # Project actual triangles onto XZ; a front-to-back Y ray intersects inside.
 for k in range(0,len(indices),3):
  a,b,c=[ps[i] for i in indices[k:k+3]];ux,uz=b[0]-a[0],b[2]-a[2];vx,vz=c[0]-a[0],c[2]-a[2];det=ux*vz-uz*vx
  if abs(det)<1e-12:continue
  dx,dz=x-a[0],z-a[2];u=(dx*vz-dz*vx)/det;v=(ux*dz-uz*dx)/det
  if u>=-1e-6 and v>=-1e-6 and u+v<=1+1e-6:return True
 return False
def measure(row):
 ps=skin(row['angles']);gaps=[max(0,ps[seams[1][r]][0]-ps[seams[-1][r]][0]) for r in seams[-1]]
 return {'tick':row['tick'],'angles':row['angles'],'mean_seam_gap':sum(gaps)/len(gaps),'max_seam_gap':max(gaps),'central_rays_blocked':sum(ray_blocked(ps,x,z) for x in [-.05,0,.05] for z in [-.15,0,.15])}
rows=[json.loads(s) for s in Path(sys.argv[1]).read_text().splitlines()];samples=[measure(r) for r in rows if r['tick']%32==0]
for r in rows:assert r['externally_imposed_motor_commands'] and int(r['energy_residual'])==int(r['material_residual'])==0
closed=[r for r in samples if r['tick']%64==32];opened=[r for r in samples if r['tick']%64==0]
assert all(r['max_seam_gap']<.001 and r['central_rays_blocked']==9 for r in closed)
assert all(r['mean_seam_gap']>.30 and r['central_rays_blocked']==0 for r in opened)
report={'status':'PASS','scope':'Actual GLB lip triangles;9 frontal rays. Not feeding collision.','controlled_samples':samples,'energy_spent_ticks_1_to_256':rows[0]['energy']-rows[-1]['energy'],'material_and_energy_residuals':'zero every tick','teeth_count':oral['teeth_count'],'throat_depth_mesh_units':oral['throat_depth'],'tongue_separate_mesh':True}
if len(sys.argv)>2:
 native=[json.loads(s) for s in Path(sys.argv[2]).read_text().splitlines()];native=[r for r in native if r['control']=='selected'];gaps=[measure(r)['mean_seam_gap'] for r in native];report['native_selected']={'samples':len(gaps),'minimum_mean_gap':min(gaps),'maximum_mean_gap':max(gaps),'full_closure_demonstrated':min(gaps)<.001}
print(json.dumps(report,indent=2))
