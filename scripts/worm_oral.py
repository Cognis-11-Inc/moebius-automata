# Historical v3 prototype, rejected by user; not used by the current rig.
"""Authored render anatomy: closing lip lobes, silver dentition and recessed throat.
All movable surfaces bind to existing jaw bones. No motor/feeding/physics changes.
"""
import math

def build(doc, append, poses):
    mouth=poses[9]; front=mouth[1]+.135
    def rotate(p,j,angle):
        c,s=math.cos(angle),math.sin(angle);x,y=p[0]-poses[j][0],p[1]-poses[j][1]
        return [poses[j][0]+c*x-s*y,poses[j][1]+s*x+c*y,p[2]]
    def mesh(name,material):
        return {'name':name,'material':material,'p':[],'n':[],'c':[],'j':[],'w':[],'idx':[]}
    lips=mesh('bilateral muscular closing lips',2);teeth=mesh('interlocking silver recurved teeth',3)
    throat=mesh('recessed pharynx',4);tongue=mesh('separate tongue',5)
    def vertex(m,p,j,col,closed=False):
        if closed:p=rotate(p,j,.25 if j==13 else -.25) # inverse actual closed rotation
        m['p'].append(p);m['n'].append([0,0,0]);m['c'].append(col+[1]);m['j'].append([j,0,0,0]);m['w'].append([1.,0,0,0]);return len(m['p'])-1
    def tri(m,a,b,c):m['idx'].append((a,b,c))
    seam=[]
    # Two thick lips span the entire opening at jaw=-256. Stored in neutral bind pose.
    rows=25;cols=10
    for side,j in [(-1,12),(1,13)]:
        start=len(lips['p'])
        for back in [False,True]:
            for row in range(rows):
                z=-.25+row/(rows-1)*.50;half=.27*math.sqrt(max(.005,1-(z/.253)**2))
                for col in range(cols):
                    t=col/(cols-1);x=side*(-.006+(half+.006)*t)
                    y=front+.025*(1-t)**2-.035*t+.008*math.sin(z*85+3*t)*math.sin(math.pi*t)-(.035 if back else 0)
                    tone=.85+.15*math.sin(z*100+t*8)
                    i=vertex(lips,[x,y,z],j,[.19*tone,.033*tone,.052*tone],True)
                    if not back and col==0:seam.append((side,row,i))
        layer=rows*cols
        for back in range(2):
            for r in range(rows-1):
                for c in range(cols-1):
                    a=start+back*layer+r*cols+c;b=a+1;d=a+cols;e=d+1
                    tri(lips,a,d,b);tri(lips,b,d,e)
        for r in range(rows-1):
            for c in [0,cols-1]:
                a=start+r*cols+c;b=a+cols;tri(lips,a,a+layer,b);tri(lips,b,a+layer,b+layer)
        for r in [0,rows-1]:
            for c in range(cols-1):
                a=start+r*cols+c;tri(lips,a,a+1,a+layer);tri(lips,a+1,a+1+layer,a+layer)
        # Short recurved teeth interdigitate; silhouette remains readable when closed.
        for k in range(10):
            z=-.213+k*.046+(0 if side<0 else .020);z=min(z,.222)
            base=[side*.052,front+.010,z];tip=[side*(-.020),front+.052,z+.014*side]
            rings=[]
            for t,radius in [(0,.019),(.45,.014),(.8,.007),(1,.0008)]:
                ids=[]
                for a in range(8):
                    theta=a*math.tau/8;p=[base[q]*(1-t)+tip[q]*t for q in range(3)]
                    p[1]+=math.cos(theta)*radius;p[2]+=math.sin(theta)*radius
                    ids.append(vertex(teeth,p,j,[.65,.73,.82],True))
                rings.append(ids)
            for r in range(3):
                for a in range(8):
                    b=(a+1)%8;tri(teeth,rings[r][a],rings[r+1][a],rings[r][b]);tri(teeth,rings[r][b],rings[r+1][a],rings[r+1][b])
    # Funnel, not a brightly lit flat plug. Rings narrow deep inside the head.
    for r in range(19):
        t=r/18;y=front-.035-t*.43;radius=(.245*(1-t)+.045*t)*(1+.035*math.sin(t*math.pi*12))
        for a in range(48):
            theta=a*math.tau/48;col=[.035*(1-t)**2+.001,.003+.003*(1-t),.009*(1-t)+.001]
            vertex(throat,[math.cos(theta)*radius*(1+.035*math.sin(theta*5+t*13)),y,math.sin(theta)*radius*(1+.025*math.sin(theta*3-t*9))],9,col)
    for r in range(18):
        for a in range(48):
            b=(a+1)%48;i=r*48+a;tri(throat,i,(r+1)*48+a,r*48+b);tri(throat,r*48+b,(r+1)*48+a,(r+1)*48+b)
    center=vertex(throat,[0,front-.47,0],9,[.0005,.0002,.0003])
    for a in range(48):tri(throat,18*48+a,center,18*48+(a+1)%48)
    # Tongue is its own mesh/material, anchored to head, not independently animated.
    for r in range(18):
        t=r/17;y=front-.37+t*.26;width=.025+.033*math.sin(math.pi*t)
        for a in range(16):
            theta=a*math.tau/16
            vertex(tongue,[math.cos(theta)*width,y,-.12+math.sin(theta)*.016+.035*t],9,[.13,.018,.04])
    for r in range(17):
        for a in range(16):
            b=(a+1)%16;i=r*16+a;tri(tongue,i,i+16,r*16+b);tri(tongue,r*16+b,i+16,(r+1)*16+b)
    tip=vertex(tongue,[0,front-.100,-.085],9,[.13,.018,.04])
    for a in range(16):tri(tongue,17*16+a,tip,17*16+(a+1)%16)
    for m in [lips,teeth,throat,tongue]:
        for a,b,c in m['idx']:
            u=[m['p'][b][q]-m['p'][a][q] for q in range(3)];v=[m['p'][c][q]-m['p'][a][q] for q in range(3)]
            n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
            for i in [a,b,c]:
                for q in range(3):m['n'][i][q]+=n[q]
        for i,n in enumerate(m['n']):
            length=math.sqrt(sum(x*x for x in n)) or 1;m['n'][i]=[x/length for x in n]
        attrs={}
        for name,key,width,kind,fmt in [('POSITION','p',3,5126,'f'),('NORMAL','n',3,5126,'f'),('COLOR_0','c',4,5126,'f'),('JOINTS_0','j',4,5123,'H'),('WEIGHTS_0','w',4,5126,'f')]:attrs[name]=append(m[key],kind,width,fmt,34962)
        ac=doc['accessors'][attrs['POSITION']];ac['min']=[min(p[q] for p in m['p']) for q in range(3)];ac['max']=[max(p[q] for p in m['p']) for q in range(3)]
        doc['meshes'][0]['primitives'].append({'mode':4,'material':m['material'],'attributes':attrs,'indices':append([(i,) for t in m['idx'] for i in t],5123,1,'H',34963),'extras':{'purpose':m['name']}})
    return {'lip_primitive':2,'lip_seam_vertices':seam,'closed_angle':-256,'open_angle':768,'throat_depth':.43,'tongue_primitive':5,'teeth_count':20,'metallic':.85}
