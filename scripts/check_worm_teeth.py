"""Audit raised existing crowns against the previously accepted closing asset.
Usage: python3 scripts/check_worm_teeth.py PRECHANGE_GLB
"""
import json,sys,math
from collections import Counter
from pathlib import Path
from rig_worm_a import read,values,OUT,ROOT
D,B=read(OUT/'Worm_A_rigged.glb');old,ob=read(Path(sys.argv[1]));meta=json.loads((OUT/'rig.json').read_text())['teeth_paint'];p=D['meshes'][0]['primitives'][0];op=old['meshes'][0]['primitives'][0];teeth=D['meshes'][0]['primitives'][2]
def attr(d,b,p,key):return values(d,b,p['attributes'][key])[0]
for key in ['POSITION','NORMAL','TEXCOORD_0','COLOR_0','JOINTS_0','WEIGHTS_0']:assert attr(D,B,p,key)==attr(old,ob,op,key),key
for t,ot in zip(p['targets'],op['targets']):
 for key in ['POSITION','NORMAL']:assert values(D,B,t[key])[0]==values(old,ob,ot[key])[0],key
source,source_blob=read(ROOT/'Worm_A.glb');source_indices=[r[0] for r in values(source,source_blob,source['meshes'][0]['primitives'][0]['indices'])[0]]
base_indices=[r[0] for r in values(D,B,p['indices'])[0]];tooth_indices=[r[0] for r in values(D,B,teeth['indices'])[0]];ids=meta['vertex_source_ids'];cap_ids=meta['cap_triangle_ids']
projected=[tuple(ids[j] for j in tooth_indices[i*3:i*3+3]) for i in cap_ids]
assert Counter(tuple(base_indices[k:k+3]) for k in range(0,len(base_indices),3))+Counter(projected)==Counter(tuple(source_indices[k:k+3]) for k in range(0,len(source_indices),3))
points=attr(D,B,p,'POSITION');caps=attr(D,B,teeth,'POSITION');distances=[math.dist(q,points[i]) for q,i in zip(caps,ids)]
assert all(d<1e-7 or abs(d-math.hypot(.006,.028))<1e-6 for d in distances)
for key in ['JOINTS_0','WEIGHTS_0']:
 base_rows=attr(D,B,p,key);assert attr(D,B,teeth,key)==[base_rows[i] for i in ids]
for t,base in zip(teeth['targets'],p['targets']):
 for key in ['POSITION','NORMAL']:
  rows=values(D,B,base[key])[0];assert values(D,B,t[key])[0]==[rows[i] for i in ids]
mat=D['materials'][2];assert mat['alphaMode']=='OPAQUE' and mat['pbrMetallicRoughness']['baseColorFactor'][3]==1 and 'extensions' not in mat
report={'status':'PASS','all_original_skin_attributes_unchanged':True,'closing_position_and_normal_targets_exactly_unchanged':True,'all_source_faces_preserved_as_skin_or_crown_caps':True,'crown_caps':meta['patches'],'source_crown_triangles':len(projected),'finished_dental_triangles':len(tooth_indices)//3,'maximum_crown_exposure_mesh_units':max(distances),'root_vertices_at_original_positions':sum(d<1e-7 for d in distances),'crown_skinning_and_closing_targets_copied_exactly':True,'material':mat,'limitations':'Authored crown refinement; not new physical feeding geometry or user-approved final art.'}
print(json.dumps(report,indent=2))
