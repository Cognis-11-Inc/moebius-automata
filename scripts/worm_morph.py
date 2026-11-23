"""Localized jaw-driven shape keys on the intact, pre-v3 sculpt.
No triangle deletion, head reshaping, replacement lips, teeth or throat meshes.
"""
import math

def build(doc,append,positions,normals,indices):
    primitive=doc['meshes'][0]['primitives'][0]
    def smooth(a,b,t):
        t=max(0,min(1,(t-a)/(b-a)));return t*t*(3-2*t)
    deltas=[[],[]];affected=0
    for p in positions:
        x,y,z=p
        radius=math.sqrt((x/.22)**2+((z-.28)/.17)**2)
        field=smooth(.30,.50,y)*(1-smooth(1.05,1.80,radius))
        upper=smooth(.255,.305,z)
        deltas[0].append([0.,.0108*field*upper,-(z-.28)*.44325*field*upper])
        deltas[1].append([0.,.024*field*(1-upper),(.36-z)*.985*field*(1-upper)])
        affected+=field>1e-5
    def normal_map(ps):
        ns=[[0.,0.,0.] for p in ps]
        for k in range(0,len(indices),3):
            a,b,c=indices[k:k+3];u=[ps[b][q]-ps[a][q] for q in range(3)];v=[ps[c][q]-ps[a][q] for q in range(3)]
            n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
            for i in [a,b,c]:
                for q in range(3):ns[i][q]+=n[q]
        merged={}
        for p,n in zip(ps,ns):
            m=merged.setdefault(tuple(round(v,6) for v in p),[0.,0.,0.])
            for q in range(3):m[q]+=n[q]
        out=[]
        for p in ps:
            n=merged[tuple(round(v,6) for v in p)];length=math.sqrt(sum(v*v for v in n)) or 1;out.append([v/length for v in n])
        return out
    targets=[]
    for delta in deltas:
        changed=[[p[q]+d[q] for q in range(3)] for p,d in zip(positions,delta)];newnorm=normal_map(changed)
        targets.append({'POSITION':append(delta,5126,3,'f',34962),'NORMAL':append([[n[q]-old[q] for q in range(3)] for n,old in zip(newnorm,normals)],5126,3,'f',34962)})
    primitive['targets']=targets
    count=doc['accessors'][doc['meshes'][0]['primitives'][1]['attributes']['POSITION']]['count']
    zero=append([[0.,0.,0.] for _ in range(count)],5126,3,'f',34962)
    doc['meshes'][0]['primitives'][1]['targets']=[{'POSITION':zero},{'POSITION':zero}]
    doc['meshes'][0]['weights']=[0,0];doc['meshes'][0]['extras']={'targetNames':['existing_upper_lip_close','existing_lower_lip_close']}
    return {'method':'Localized vertical upper/lower rim closure on intact pre-v3 sculpt; existing jaw signals drive shape keys; oral skin anchored to head9','affected_vertices':affected,'original_triangles':len(indices)//3,'primitive_count':2,'closed_angle':-256,'open_angle':768,'mouth_view_offset':[0,0,.28]}
