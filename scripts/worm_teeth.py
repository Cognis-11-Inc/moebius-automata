"""Expose and finish the existing incisor/canine crowns, not generic replacement teeth.
Source face patches become raised caps with rooted collars; lip geometry/morphs stay intact.
"""
import math
SEEDS=[[-.151883,.607912,.296552],[-.127626,.637325,.324926],[-.106891,.657757,.344010],[-.078517,.683189,.362532],[-.046549,.680587,.366418],[-.025275,.657618,.077836],[.003792,.705002,.079371],[.028720,.657785,.076701],[.051613,.652139,.089007]]
def build(doc,append,positions,indices,colors,blob):
    from rig_worm_a import values
    seeds=[]
    for candidate in SEEDS+[[-p[0],p[1],p[2]] for p in SEEDS if abs(p[0])>.012]:
        if all(math.dist(candidate,s)>.015 for s in seeds):seeds.append(candidate)
    def metric(p,s):return ((p[0]-s[0])/.016)**2+((p[1]-s[1])/.028)**2+((p[2]-s[2])/.024)**2
    patches={};skin=[];selected=[]
    for k in range(0,len(indices),3):
        tri=indices[k:k+3];center=[sum(positions[i][q] for i in tri)/3 for q in range(3)]
        distance,seed=min((metric(center,s),j) for j,s in enumerate(seeds))
        if distance<1.25:
            patches.setdefault(seed,[]).append(tuple(tri));selected.append(k//3)
        else:skin.extend(tri)
    # Keep only the connected crown surface nearest each picked tip. Thin
    # mouth folds can otherwise contribute unrelated, floating triangle shards.
    clean={};selected=[]
    face_by_tri={tuple(indices[k:k+3]):k//3 for k in range(0,len(indices),3)}
    for seed,tris in patches.items():
        edge_faces={}
        for i,t in enumerate(tris):
            for a,b in zip(t,t[1:]+t[:1]):edge_faces.setdefault(tuple(sorted((a,b))),[]).append(i)
        start=min(range(len(tris)),key=lambda i:metric([sum(positions[v][q] for v in tris[i])/3 for q in range(3)],seeds[seed]))
        seen={start};stack=[start]
        while stack:
            i=stack.pop();t=tris[i]
            for a,b in zip(t,t[1:]+t[:1]):
                for j in edge_faces[tuple(sorted((a,b)))]:
                    if j not in seen:seen.add(j);stack.append(j)
        clean[seed]=[tris[i] for i in sorted(seen)]
        for i,t in enumerate(tris):
            if i not in seen:skin.extend(t)
        selected.extend(face_by_tri[t] for t in clean[seed])
    patches=clean
    assert patches
    original=doc['meshes'][0]['primitives'][0]
    original_normals=values(doc,blob,original['attributes']['NORMAL'])[0]
    js=values(doc,blob,original['attributes']['JOINTS_0'])[0];ws=values(doc,blob,original['attributes']['WEIGHTS_0'])[0]
    morphs=[values(doc,blob,t['POSITION'])[0] for t in original['targets']]
    morph_normals=[values(doc,blob,t['NORMAL'])[0] for t in original['targets']]
    ps=[];joints=[];weights=[];deltas=[[],[]];triangles=[];provenance=[];cap_triangle_ids=[];wall_triangle_ids=[]
    for seed,tris in sorted(patches.items()):
        edges={};vertices=set(i for t in tris for i in t)
        for tri in tris:
            for a,b in zip(tri,tri[1:]+tri[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
        # Raising the same source crown patch exposes its existing form above
        # the gum. A collar closes every boundary back to the original root.
        zsign=-1 if seeds[seed][2]>.20 else 1
        offset=[0.,.006,zsign*.028]
        root={};cap={}
        for i in sorted(vertices):
            for table,move in [(root,[0.,0.,0.]),(cap,offset)]:
                table[i]=len(ps);ps.append([positions[i][q]+move[q] for q in range(3)]);joints.append(js[i]);weights.append(ws[i]);provenance.append(i)
                for m in range(2):deltas[m].append(morphs[m][i])
        for tri in tris:
            cap_triangle_ids.append(len(triangles));triangles.append(tuple(cap[i] for i in tri))
        for edge,uses in edges.items():
            if len(uses)!=1:continue
            a,b=uses[0];wall_triangle_ids.extend([len(triangles),len(triangles)+1]);triangles.extend([(root[a],root[b],cap[b]),(root[a],cap[b],cap[a])])
    # Collar faces need their own normals: inheriting gum normals makes raised
    # teeth read as painted stickers. Keep crown normals and crisp root edges.
    normals=[original_normals[i] for i in provenance]
    for ti in wall_triangle_ids:
        ids=triangles[ti];a,b,c=[ps[i] for i in ids]
        u=[b[q]-a[q] for q in range(3)];v=[c[q]-a[q] for q in range(3)]
        n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        length=math.sqrt(sum(q*q for q in n)) or 1;n=[q/length for q in n]
        new=[]
        for i in ids:
            new.append(len(ps));ps.append(ps[i]);normals.append(n);joints.append(joints[i]);weights.append(weights[i]);provenance.append(provenance[i])
            for m in range(2):deltas[m].append(deltas[m][i])
        triangles[ti]=tuple(new)
    attrs={}
    for name,rows,width,kind,fmt in [('POSITION',ps,3,5126,'f'),('NORMAL',normals,3,5126,'f'),('JOINTS_0',joints,4,5123,'H'),('WEIGHTS_0',weights,4,5126,'f')]:attrs[name]=append(rows,kind,width,fmt,34962)
    a=doc['accessors'][attrs['POSITION']];a['min']=[min(p[q] for p in ps) for q in range(3)];a['max']=[max(p[q] for p in ps) for q in range(3)]
    targets=[{'POSITION':append(d,5126,3,'f',34962),'NORMAL':append([morph_normals[m][i] for i in provenance],5126,3,'f',34962)} for m,d in enumerate(deltas)]
    original['indices']=append([(i,) for i in skin],5125,1,'I',34963)
    doc['meshes'][0]['primitives'].append({'mode':4,'material':2,'attributes':attrs,'indices':append([(i,) for t in triangles for i in t],5125,1,'I',34963),'targets':targets})
    doc['materials'].append({'name':'Existing raised crowns - opaque brushed gunmetal','pbrMetallicRoughness':{'baseColorFactor':[.24,.28,.32,1],'metallicFactor':.86,'roughnessFactor':.34},'alphaMode':'OPAQUE','doubleSided':True})
    return {'purpose':'Slightly exposed existing crown patches with root collars; no replacement cone teeth','seeds':seeds,'patches':len(patches),'source_face_ids':selected,'source_cap_triangles':len(selected),'finished_triangles':len(triangles),'new_surface_vertices':len(ps),'vertex_source_ids':provenance,'cap_triangle_ids':cap_triangle_ids,'crown_offset':[0,.006,.028],'material':'opaque brushed gunmetal','metallic':.86,'roughness':.34,'closing_morph_deltas':'exact copies by original vertex ID'}
