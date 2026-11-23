"""Independent linear-skin edge audit against recorded native motor poses.
Usage: python3 scripts/check_worm_a.py RESTORE/heldout-v1/episodes.jsonl.gz
Reports visual deformation only; this does not test or alter physics.
"""
import sys,gzip,json,math,struct
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent));from rig_worm_a import read,values
p=Path(__file__).resolve().parents[1]/'assets/creatures/worm_a';doc,blob=read(Path(sys.argv[2]) if len(sys.argv)>2 else p/'Worm_A_rigged.glb');meta=json.loads((p/'rig.json').read_text());bind=meta['bind_positions']
def mat(a,b):return [[sum(a[i][k]*b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]
def apply(m,v):return [sum(m[a][b]*v[b] for b in range(3)) for a in range(3)]
def rotation(z,y):
 cz,sz,cy,sy=math.cos(z),math.sin(z),math.cos(y),math.sin(y)
 return mat([[cz,-sz,0],[sz,cz,0],[0,0,1]],[[cy,0,sy],[0,1,0],[-sy,0,cy]])
def transforms(angles):
 matrices=[];points=[]
 for i in range(14):
  parent=i-1 if 0<i<12 else 9 if i>=12 else None
  m=matrices[parent] if parent is not None else [[1,0,0],[0,1,0],[0,0,1]]
  base=points[parent] if parent is not None else [0,0,0];delta=[bind[i][a]-(bind[parent][a] if parent is not None else 0) for a in range(3)]
  offset=apply(m,delta);points.append([base[a]+offset[a] for a in range(3)])
  local=rotation(angles[i*2]/1024,angles[i*2+1]/1024) if i<12 else rotation((-1 if i==12 else 1)*angles[24+i-12]/1024,0)
  matrices.append(mat(m,local))
 return matrices,points
poses={}
with gzip.open(sys.argv[1],'rt') as f:
 for line in f:
  v=json.loads(line)
  if v['type']=='end':break
  if v['type']=='tick' and v['tick'] in [0,15,31,63,127,255]:poses[v['tick']]=v['neural']['body']['angles']
results=[]
for primitive in doc['meshes'][0]['primitives']:
 ps,_,_=values(doc,blob,primitive['attributes']['POSITION']);js,_,_=values(doc,blob,primitive['attributes']['JOINTS_0']);ws,_,_=values(doc,blob,primitive['attributes']['WEIGHTS_0']);idx,_,_=values(doc,blob,primitive['indices']);idx=[i[0] for i in idx]
 assert all(abs(sum(w)-1)<1e-5 for w in ws)
 assert all(0<=j<14 for row in js for j in row)
 edges=set()
 for i in range(0,len(idx),3):
  a,b,c=idx[i:i+3]
  for x,y in [(a,b),(b,c),(a,c)]:edges.add(tuple(sorted((x,y))))
 for tick,angles in poses.items():
  ms,bs=transforms(angles);deformed=[]
  for p,j,w in zip(ps,js,ws):
   accum=[0,0,0]
   for bone,weight in zip(j,w):
    v=apply(ms[bone],[p[a]-bind[bone][a] for a in range(3)])
    for a in range(3):accum[a]+=weight*(v[a]+bs[bone][a])
   deformed.append(accum)
  ratios=[]
  for a,b in edges:
   rest=math.dist(ps[a],ps[b])
   if rest>1e-4:ratios.append(math.dist(deformed[a],deformed[b])/rest)
  ratios.sort();results.append({'primitive':primitive['material'],'tick':tick,'max_edge_stretch':max(ratios),'p99':ratios[int(len(ratios)*.99)],'p999':ratios[int(len(ratios)*.999)]})
print(json.dumps(results,indent=2))
