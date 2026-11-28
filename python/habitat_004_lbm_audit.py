#!/usr/bin/env python3
"""Read-only reproduction and interlevel-population inventory audit of original003.
No revised candidate, flux repair or normalization. Source is independently traced
from scientific/lbm.rs; the retained failed executable remains authoritative.
"""
import json
import math
from pathlib import Path
from habitat_003_qualification import decode
C=[(0,0,0),(1,0,0),(-1,0,0),(0,1,0),(0,-1,0),(0,0,1),(0,0,-1),(1,1,0),(-1,-1,0),(1,-1,0),(-1,1,0),(1,0,1),(-1,0,-1),(1,0,-1),(-1,0,1),(0,1,1),(0,-1,-1),(0,1,-1),(0,-1,1)]
def linear_sum(xs):
    total=0.
    for x in xs:total+=x
    return total

def audit(folder):
    folder=Path(folder);initial=decode(json.loads((folder/'initial.json').read_text())['state']);failure=json.loads((folder/'summary.json').read_text())['failure'];target=failure['candidate'];cfg=initial['config'];cells={int(k):p['Lbm']for k,p in initial['cells'].items()};tau=lambda level:.5+3*cfg['viscosity']/cfg['density']*(cfg['dt']/2**level)/(cfg['h']/2**level)**2
    def geometry(key):
        root=key>>4;id_=key&15;level=int(id_!=0);h=cfg['h']/2**level;child=max(0,id_-1);ijk=[root%cfg['roots'][0],root//cfg['roots'][0]%cfg['roots'][1],root//(cfg['roots'][0]*cfg['roots'][1])];p=[ijk[k]*cfg['h']+(((child>>k)&1)+.5)*h if level else(ijk[k]+.5)*h for k in range(3)];return level,h,p
    def locate(pos):
        xyz=[int(math.floor(p/cfg['h']))for p in pos];root=xyz[0]+cfg['roots'][0]*(xyz[1]+cfg['roots'][1]*xyz[2]);key=root<<4
        if key not in cells:key|=1+sum(int(pos[k]-xyz[k]*cfg['h']>=cfg['h']*.5)<<k for k in range(3))
        return key
    def eq(f):
        rho=linear_sum(f);inverse=1/rho;u=[linear_sum(f[q]*C[q][k]for q in range(19))*inverse for k in range(3)];u2=linear_sum(x*x for x in u);out=[]
        for q in range(19):
            cu=linear_sum(C[q][k]*u[k]for k in range(3));w=1/3 if q==0 else 1/18 if q<7 else 1/36;out.append(w*rho*(1+3*cu+4.5*cu*cu-1.5*u2))
        return out
    def rescale(f,a,b):
        equilibrium=eq(f);factor=tau(b)/tau(a)*2.**(a-b);return[e+factor*(x-e)for x,e in zip(f,equilibrium)]
    def restricted(state,root):
        average=[0.]*19
        for child in range(1,9):
            for q in range(19):average[q]+=state[root|child][q]/8
        return rescale(average,1,0)
    def inventory(state):
        total=[0.]*4
        for key,f in sorted(state.items()):
            _,h,_=geometry(key);v=h**3;total[0]+=linear_sum(f)*v
            for k in range(3):total[k+1]+=linear_sum(x*d[k]for x,d in zip(f,C))*v*cfg['h']/cfg['dt']
        return total
    def sweep(state,level,coarse=None):
        new={};incoming=[0.]*4;outgoing=[0.]*4;collision_roundoff=[0.]*4
        for key,old in sorted(state.items()):
            lvl,h,p=geometry(key)
            if lvl!=level:continue
            equilibrium=eq(old);post=[x+(e-x)/tau(level)for x,e in zip(old,equilibrium)];f=[0.]*19
            for q,direction in enumerate(C):
                dest=[(p[k]+direction[k]*h)%(cfg['roots'][k]*cfg['h'])for k in range(3)];dest_key=locate(dest)
                if geometry(dest_key)[0]!=level:
                    outgoing[0]+=post[q]*h**3
                    for k in range(3):outgoing[k+1]+=post[q]*C[q][k]*h**3*cfg['h']/cfg['dt']
                at=[(p[k]-direction[k]*h)%(cfg['roots'][k]*cfg['h'])for k in range(3)];source=locate(at);source_level=geometry(source)[0]
                if level==0 and source_level==1:ghost=restricted(state,source&~15)
                else:ghost=rescale((coarse if source_level==0 and coarse is not None else state)[source],source_level,level)
                equilibr=eq(ghost);value=ghost[q]+(equilibr[q]-ghost[q])/tau(level);f[q]=value
                if source_level!=level:
                    incoming[0]+=value*h**3
                    for k in range(3):incoming[k+1]+=value*C[q][k]*h**3*cfg['h']/cfg['dt']
            new[key]=f
        before=inventory({k:v for k,v in state.items()if geometry(k)[0]==level});after=inventory(new);change=[b-a for a,b in zip(before,after)];exchange=[a-b for a,b in zip(incoming,outgoing)];return new,{'level':level,'ghost_incoming_inventory':incoming,'outgoing_inventory':outgoing,'net_interlevel_inventory':exchange,'actual_level_inventory_change':change,'unexplained_within_level_roundoff':[a-b for a,b in zip(change,exchange)]}
    coarse,c0=sweep(cells,0);fine1,f1=sweep(cells,1);half=dict(cells);half.update(fine1);ghosts={k:[.5*(a+b)for a,b in zip(cells[k],f)]for k,f in coarse.items()};fine2,f2=sweep(half,1,ghosts);final=dict(cells);final.update(coarse);final.update(fine2)
    maximum=max(abs(x-y)for k,f in final.items()for x,y in zip(f,target['cells'][str(k)]['Lbm']));assert maximum<=1e-10,maximum
    net=[linear_sum(row['net_interlevel_inventory'][k]for row in [c0,f1,f2])for k in range(4)];change=[b-a for a,b in zip(inventory(cells),inventory(final))];assert all(abs(a-b)<1e-11 for a,b in zip(net,change)),(net,change)
    return {'status':'MECHANICAL_FAILURE_REPRODUCED','max_candidate_population_difference':maximum,'stages':[c0,f1,f2],'net_interlevel_inventory':net,'actual_global_inventory_change':change,'original_reported_residual':failure['candidate_diagnostics']['residual'][:4],'limits':'Only original planar-refinement nonuniform wave seed audited. Tiny differences from Rust reduction/transcendental ordering retained. Not a new interface or conservative repair.','mechanism':'Coarse step consumes a restricted whole-root old fine snapshot; fine half-steps consume old/interpolated coarse ghosts. Their actual direction/time-integrated interlevel transfers are unpaired and do not cancel. No common interface register supplies actual opposing exchange.'}
if __name__=='__main__':
    import sys
    result=audit(sys.argv[1]);Path(sys.argv[2]).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
