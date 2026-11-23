"""Build a flesh-first render rig from the preserved user sculpt.
Local glTF transformation: original sculpt connectivity, UV and image retained. Rear contour adapted; a separate tissue layer and vertex paint added.
The mouth landmark is authored from inspected front/side renders; physical anchor is joint9.
"""
import hashlib,json,math,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'assets/creatures/worm_a'

def read(path):
    data=path.read_bytes(); size=struct.unpack_from('<I',data,12)[0]
    doc=json.loads(data[20:20+size]); offset=20+size
    count,kind=struct.unpack_from('<II',data,offset);assert kind==0x004e4942
    return doc,bytearray(data[offset+8:offset+8+count])

def values(doc,blob,index):
    acc=doc['accessors'][index];view=doc['bufferViews'][acc['bufferView']]
    width={'VEC3':3,'VEC2':2,'VEC4':4,'MAT4':16,'SCALAR':1}[acc['type']];fmt={5126:'f',5125:'I',5123:'H'}[acc['componentType']]
    item=struct.calcsize('<'+fmt*width);stride=view.get('byteStride',item)
    offset=view.get('byteOffset',0)+acc.get('byteOffset',0)
    return [struct.unpack_from('<'+fmt*width,blob,offset+i*stride) for i in range(acc['count'])],offset,stride

def main():
    source=ROOT/'Worm_A.glb';doc,blob=read(source);primitive=doc['meshes'][0]['primitives'][0]
    # Only editor caches are removed; no remesh, smoothing, decimation or texture repaint.
    primitive.pop('extras',None)
    scale=.3/.48/1024
    poses=[[0,(-1720+i*297)*scale,0] for i in range(12)]
    mouth=poses[9];poses += [[-.08,mouth[1]+.14,0],[.08,mouth[1]+.14,0]]
    landmark=[0,.32,.80] # source glTF coordinates: original+Z is front, original+Y is dorsal
    asset_scale=(mouth[1]-poses[0][1])/(.80-(-.9803120493888855))
    positions,offset,stride=values(doc,blob,primitive['attributes']['POSITION'])
    # Fit the rear hood into a supple worm contour; preserve the recognizable face.
    original=[]
    for x,y,z in positions:
        original.append([-x*asset_scale,(z-landmark[2])*asset_scale+mouth[1],(y-landmark[1])*asset_scale])
    def smooth(a,b,t):
        t=max(0,min(1,(t-a)/(b-a)));return t*t*(3-2*t)
    sections=[]
    for i in range(28):
        y=-1.1+i*.075;near=[p for p in original if abs(p[1]-y)<.09]
        center=(min(p[2] for p in near)+max(p[2] for p in near))/2 if near else 0
        radius=max((math.hypot(p[0],p[2]-center) for p in near),default=1)
        sections.append((center,max(radius,.1)))
    transformed=[]
    for x,y,z in original:
        t=max(0,min(27,(y+1.1)/.075));i=int(t);j=min(i+1,27);f=t-i
        center=sections[i][0]*(1-f)+sections[j][0]*f;radius=sections[i][1]*(1-f)+sections[j][1]*f
        shape=smooth(.05,.42,y)
        target=.16+.015*math.cos(y*9)
        body=[x*target/radius,y,(z-center)*target/radius]
        transformed.append([body[0]*(1-shape)+x*shape,y,body[2]*(1-shape)+z*shape])
    for i,p in enumerate(transformed):struct.pack_into('<3f',blob,offset+i*stride,*p)
    acc=doc['accessors'][primitive['attributes']['POSITION']]
    acc['min']=[min(p[a] for p in transformed) for a in range(3)];acc['max']=[max(p[a] for p in transformed) for a in range(3)]
    index_rows,_,_=values(doc,blob,primitive['indices']);indices=[v[0] for v in index_rows]
    normals=[[0.,0.,0.] for _ in transformed];neighbors=[set() for _ in transformed]
    for i in range(0,len(indices),3):
        a,b,c=indices[i:i+3];pa,pb,pc=transformed[a],transformed[b],transformed[c]
        u=[pb[k]-pa[k] for k in range(3)];v=[pc[k]-pa[k] for k in range(3)]
        n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        for idx in [a,b,c]:
            for k in range(3):normals[idx][k]+=n[k]
        neighbors[a].update([b,c]);neighbors[b].update([a,c]);neighbors[c].update([a,b])
    merged={}
    for p,n in zip(transformed,normals):
        key=tuple(round(v,6) for v in p);acc=merged.setdefault(key,[0.,0.,0.])
        for k in range(3):acc[k]+=n[k]
    for i,p in enumerate(transformed):
        n=merged[tuple(round(v,6) for v in p)];length=math.sqrt(sum(v*v for v in n)) or 1;normals[i]=[v/length for v in n]
    _,offset,stride=values(doc,blob,primitive['attributes']['NORMAL'])
    for i,n in enumerate(normals):struct.pack_into('<3f',blob,offset+i*stride,*n)
    def append(rows,kind,width,fmt,target=None):
        while len(blob)%4:blob.append(0)
        start=len(blob)
        for row in rows:blob.extend(struct.pack('<'+fmt*width,*row))
        view={'buffer':0,'byteOffset':start,'byteLength':len(blob)-start}
        if target:view['target']=target
        doc['bufferViews'].append(view)
        a={'bufferView':len(doc['bufferViews'])-1,'componentType':kind,'count':len(rows),'type':{1:'SCALAR',2:'VEC2',3:'VEC3',4:'VEC4',16:'MAT4'}[width]}
        doc['accessors'].append(a);return len(doc['accessors'])-1
    def binding(p,mouth_skin=True):
        x,y,z=p;axis=max(0,min(11,(y-poses[0][1])/(297*scale)));base=int(axis);t=axis-base
        spline=[((1-t)**3)/6,(3*t*t*t-6*t*t+4)/6,(-3*t*t*t+3*t*t+3*t+1)/6,t*t*t/6]
        h=smooth(poses[8][1],mouth[1],y)*math.exp(-(z/.30)**4) if mouth_skin else 0
        jaw=.80*math.exp(-((y-mouth[1])/.22)**2-(z/.23)**2)*h
        influence={}
        body_influences=zip([base,base+1],[1-t,t]) if mouth_skin else zip([base-1,base,base+1,base+2],spline)
        for idx,w in body_influences:
            idx=max(0,min(11,idx));influence[idx]=influence.get(idx,0)+w*(1-h)
        influence[9]=influence.get(9,0)+h-jaw
        left=max(0,min(1,.5-x/.12));influence[12]=jaw*left;influence[13]=jaw*(1-left)
        ranked=sorted(influence.items(),key=lambda item:(-item[1],item[0]))[:4]
        while len(ranked)<4:ranked.append((0,0.))
        total=sum(w for _,w in ranked)
        return [j for j,_ in ranked],[w/total for _,w in ranked]
    joints=[];weights=[];colors=[];jaw_vertices=[0,0]
    for i,p in enumerate(transformed):
        js,ws=binding(p)
        # The visible mouth closes vertically through shape keys. Anchor its
        # tissue to the head instead of rotating left/right skin lobes.
        x,y,z=p
        region=smooth(.27,.49,y)*(1-smooth(.30,.46,abs(x)))*(1-smooth(.28,.44,abs(z-.28)))
        influence={9:region}
        for j,w in zip(js,ws):
            j=9 if j>=12 else j
            influence[j]=influence.get(j,0)+w*(1-region)
        ranked=sorted(influence.items(),key=lambda a:-a[1])[:4]
        while len(ranked)<4:ranked.append((0,0.))
        js=[j for j,w in ranked];ws=[w for j,w in ranked]
        total=sum(ws);ws=[w/total for w in ws]
        joints.append(js);weights.append(ws)
        for side in range(2):jaw_vertices[side]+=sum(w for j,w in zip(js,ws) if j==12+side)>.01
        x,y,z=p;near=neighbors[i]
        mean=[sum(transformed[j][k] for j in near)/len(near) for k in range(3)] if near else p
        relief=sum(normals[i][k]*(p[k]-mean[k]) for k in range(3))
        ridge=smooth(.003,.018,relief)
        # Thin muted cuticle only on convex ridges, living tissue between them.
        vein=.5+.5*math.sin(30*y+7*math.sin(13*x+4*z))
        tissue=[.20+.025*vein,.045+.012*vein,.065+.015*vein]
        bone=[.39,.31,.20];cuticle=ridge*.70
        col=[tissue[k]*(1-cuticle)+bone[k]*cuticle for k in range(3)]
        cavity=math.exp(-((x/.20)**4+(z/.20)**4))*(1-smooth(mouth[1]-.12,mouth[1]+.03,y))*smooth(.22,.42,y)
        col=[v*(1-.94*cavity) for v in col];colors.append(col+[1.])
    primitive['attributes']['COLOR_0']=append(colors,5126,4,'f',34962)
    primitive['attributes']['JOINTS_0']=append(joints,5123,4,'H',34962)
    primitive['attributes']['WEIGHTS_0']=append(weights,5126,4,'f',34962)
    # Supple tissue under the sculpted cuticle, built in the same rig coordinates.
    flesh=[];fnorm=[];fuv=[];fcolors=[];fjs=[];fws=[];findices=[]
    rings=65;sides=24
    for ring in range(rings):
        t=ring/(rings-1);y=poses[0][1]+t*(mouth[1]-poses[0][1])
        radius=(.07+.065*math.sin(math.pi*t/2))*(.97+.03*math.cos(t*math.pi*22))
        for side in range(sides):
            angle=side/sides*math.tau;x=math.cos(angle)*radius;z=math.sin(angle)*radius
            p=[x,y,z];flesh.append(p);fnorm.append([math.cos(angle),0,math.sin(angle)]);fuv.append([side/sides,t])
            col=[.22+.025*math.sin(t*60),.046,.065]
            if t>.85:col=[v*(1-.8*(t-.85)/.15) for v in col]
            fcolors.append(col+[1.]);js,ws=binding(p,False);fjs.append(js);fws.append(ws)
    for ring in range(rings-1):
        for side in range(sides):
            a=ring*sides+side;b=ring*sides+(side+1)%sides;c=a+sides;d=b+sides
            findices += [(a,c,b),(b,c,d)]
    flesh_primitive={'mode':4,'material':1,'attributes':{
        'POSITION':append(flesh,5126,3,'f',34962),'NORMAL':append(fnorm,5126,3,'f',34962),
        'TEXCOORD_0':append(fuv,5126,2,'f',34962),'COLOR_0':append(fcolors,5126,4,'f',34962),
        'JOINTS_0':append(fjs,5123,4,'H',34962),'WEIGHTS_0':append(fws,5126,4,'f',34962)},
        'indices':append([(v,) for tri in findices for v in tri],5123,1,'H',34963)}
    for key in ['POSITION']:
        a=doc['accessors'][flesh_primitive['attributes'][key]];a['min']=[min(p[k] for p in flesh) for k in range(3)];a['max']=[max(p[k] for p in flesh) for k in range(3)]
    doc['meshes'][0]['primitives'].append(flesh_primitive)
    from worm_morph import build
    mouth_deformation=build(doc,append,transformed,normals,indices)
    matrices=[]
    for x,y,z in poses:matrices.append([1,0,0,0,0,1,0,0,0,0,1,0,-x,-y,-z,1])
    inverse=append(matrices,5126,16,'f')
    names=[f'spine_{i:02}' for i in range(12)]+['jaw_left','jaw_right']
    nodes=[{'name':'Worm_A','mesh':0,'skin':0},{'name':'Worm_A_Armature','children':[2]}]
    for i,p in enumerate(poses):
        parent=i-1 if 0<i<12 else 9 if i>=12 else None
        translation=[p[a]-(poses[parent][a] if parent is not None else 0) for a in range(3)]
        node={'name':names[i],'translation':translation}
        children=([i+3] if i<11 else [])+([14,15] if i==9 else [])
        if children:node['children']=children
        nodes.append(node)
    doc['nodes']=nodes;doc['scenes']=[{'name':'Worm_A_rigged','nodes':[0,1]}];doc['scene']=0
    doc['skins']=[{'name':'Worm_A_14','inverseBindMatrices':inverse,'skeleton':2,'joints':list(range(2,16))}]
    doc['materials'][0]['name']='Worm_A subtle cuticle on painted living tissue'
    doc['materials'][0]['pbrMetallicRoughness'].update(baseColorFactor=[1,1,1,1],metallicFactor=0,roughnessFactor=.42)
    doc['materials'].append({'name':'Worm_A wet muscle','pbrMetallicRoughness':{'baseColorFactor':[1,1,1,1],'metallicFactor':0,'roughnessFactor':.28},'doubleSided':True})
    from worm_teeth import build as paint_teeth
    teeth_paint=paint_teeth(doc,append,transformed,indices,colors,blob)
    doc['buffers']=[{'byteLength':len(blob)}];doc['asset']['generator']='Moebius Automata rig_worm_a.py v6; preserved user sculpt'
    raw=json.dumps(doc,separators=(',',':')).encode();raw+=b' '*((-len(raw))%4);blob+=b'\0'*((-len(blob))%4)
    result=struct.pack('<III',0x46546c67,2,12+8+len(raw)+8+len(blob))+struct.pack('<II',len(raw),0x4e4f534a)+raw+struct.pack('<II',len(blob),0x004e4942)+blob
    OUT.mkdir(parents=True,exist_ok=True);dest=OUT/'Worm_A_rigged.glb';dest.write_bytes(result)
    report={'source':'Worm_A.glb','source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'rigged_sha256':hashlib.sha256(result).hexdigest(),'mouth_deformation':mouth_deformation,'teeth_paint':teeth_paint,'primitive_count':3,'style':'Flexible flesh with subtle cuticle. Rear contour reprofiled; face/mouth retained. Original source preserved.','flesh_vertices':len(flesh),'vertices':len(positions),'indices':doc['accessors'][primitive['indices']]['count'],'joints':names,'bind_positions':poses,'mouth_joint':9,'mouth_source_landmark':landmark,'mouth_bind_anchor':mouth,'jaw_weighted_vertices':jaw_vertices,'asset_uniform_scale':asset_scale,'transform':'x=-source.x; y=source.z-mouth.z+joint9; z=source.y-mouth.y; all uniformly scaled; proper rigid rotation, not reflection','body_physics':'unchanged; collision companion preserved, not installed as new physics','jaw_mapping':'Existing motor24 drives upper-lip close shape; motor25 lower-lip close shape. Oral skin anchored to head9; no left/right lip rotation, clock or baked gait.','max_weight_error':max(abs(sum(w)-1) for w in weights)}
    (OUT/'rig.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
