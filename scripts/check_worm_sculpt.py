"""Verify intact sculpt and vertical closure. Usage: check_worm_sculpt.py PRE_V3_GLB
The saved basis must equal the accepted pre-v3 mesh; closure is authored render geometry.
"""
import json,sys,math
from collections import Counter
from pathlib import Path
from rig_worm_a import read,values,OUT
D,B=read(OUT/'Worm_A_rigged.glb');old,ob=read(Path(sys.argv[1]));P=D['meshes'][0]['primitives'][0];OP=old['meshes'][0]['primitives'][0]
def attribute(d,b,p,name):return values(d,b,p['attributes'][name])[0]
points=attribute(D,B,P,'POSITION');baseline=attribute(old,ob,OP,'POSITION');assert points==baseline
indices=[i[0] for i in values(D,B,P['indices'])[0]]
if len(D['meshes'][0]['primitives'])==3:
    dental=D['meshes'][0]['primitives'][2]
    dental_indices=[i[0] for i in values(D,B,dental['indices'])[0]]
    metadata=json.loads((OUT/'rig.json').read_text()).get('teeth_paint',{})
    if 'vertex_source_ids' in metadata:
        ids=metadata['vertex_source_ids']
        indices += [ids[j] for i in metadata['cap_triangle_ids'] for j in dental_indices[i*3:i*3+3]]
    else:
        assert dental['attributes']['POSITION']==P['attributes']['POSITION']
        indices += dental_indices
original=[i[0] for i in values(old,ob,OP['indices'])[0]]
if len(old['meshes'][0]['primitives'])==3:original += [i[0] for i in values(old,ob,old['meshes'][0]['primitives'][2]['indices'])[0]]
assert Counter(tuple(indices[i:i+3]) for i in range(0,len(indices),3))==Counter(tuple(original[i:i+3]) for i in range(0,len(original),3))
assert attribute(D,B,P,'TEXCOORD_0')==attribute(old,ob,OP,'TEXCOORD_0')
targets=[values(D,B,t['POSITION'])[0] for t in P['targets']];closed=[[p[q]+sum(t[i][q] for t in targets) for q in range(3)] for i,p in enumerate(points)]
assert all(v[0]==0 for t in targets for v in t)
fixed=sum(math.dist(p,c)<1e-7 for p,c in zip(points,closed))
assert fixed>len(points)*.75
assert all(math.dist(p,c)<1e-7 for p,c in zip(points,closed) if p[1]<.30 or math.sqrt((p[0]/.22)**2+((p[2]-.28)/.17)**2)>1.8)
def depths(ps):
 result=[]
 for x in [-.12,-.08,-.04,0,.04,.08,.12]:
  for z in [.17,.20,.23,.28,.33,.36,.39]:
   nearest=-999
   for k in range(0,len(indices),3):
    a,b,c=[ps[i] for i in indices[k:k+3]];ux,uz=b[0]-a[0],b[2]-a[2];vx,vz=c[0]-a[0],c[2]-a[2];det=ux*vz-uz*vx
    if abs(det)<1e-12:continue
    dx,dz=x-a[0],z-a[2];u=(dx*vz-dz*vx)/det;v=(ux*dz-uz*dx)/det
    if u>=0 and v>=0 and u+v<=1:nearest=max(nearest,a[1]+u*(b[1]-a[1])+v*(c[1]-a[1]))
   result.append(nearest)
 return result
open_depths=depths(points);closed_depths=depths(closed)
report={'status':'PASS','basis_positions_exactly_equal_pre_v3':True,'triangle_connectivity_and_count_exactly_equal_baseline':True,'source_uv_exact':True,'vertices':len(points),'triangles':len(indices)//3,'fixed_vertices':fixed,'fixed_fraction':fixed/len(points),'max_horizontal_displacement':0,'max_vertical_displacement':max(abs(c[2]-p[2]) for p,c in zip(points,closed)),'frontal_rays':49,'open_rays_reaching_behind_y_0_52':sum(y<.52 for y in open_depths),'closed_rays_reaching_behind_y_0_52':sum(y<.52 for y in closed_depths),'geometry_only':'Original sculpt projection; raised dental collars excluded. Not feeding collision, learning or automatic chewing.'}
assert report['closed_rays_reaching_behind_y_0_52']<report['open_rays_reaching_behind_y_0_52']
print(json.dumps(report,indent=2))
