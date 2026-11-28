#!/usr/bin/env python3
"""HABITAT-004 immutable CPU recipes, independent endpoint budgets and scattering.

No physics or renderer authority. Old003 mathematical decoder reused unchanged.
"""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess
from habitat_003_qualification import decode


def recipes():
    base={'model':'FiniteVolume','origin':{'region':[0,0,0],'local':[0,0,0]},'roots':[8,8,8],'h':.125,'dt':.0005,'density':20.,'viscosity':.1,'sound_speed':math.sqrt(3)*2,'gravity':[0.,0.,0.],'species':[],'periodic':True,'capacity':800000,'indicators':None,'reconstruction':'Constant','integrator':'Euler','vortex_forcing':None}
    def make(control='Equilibrium',n=8,scheme='reference',mesh='uniform',dt=.0005,cycles=100,cache=True):
        c=copy.deepcopy(base);c.update(roots=[n]*3,h=1/n,dt=dt)
        c['reconstruction']='Minmod' if scheme in ['space','combined']else 'Constant';c['integrator']='Ssp2' if scheme in ['time','combined']else 'Euler'
        if control in ['Diffusion3','Diagonal','Steep']:
            c['species']=[{'material':'authored-004-scalar','radius_m':1e-5,'density_kg_m3':20.,'diffusivity_m2_s':.01 if control=='Diffusion3'else 0. if control=='Steep'else .001}]
        if control=='Vortex':c['vortex_forcing']={'amplitude':.005,'wavenumber':2*math.pi}
        if control=='Hydrostatic':c.update(periodic=False,gravity=[0.,-.0025,0.])
        return {'config':c,'control':control,'cycles':cycles,'refined':[[.25]*3,[.75]*3]if mesh=='refined'else None,'fine':mesh=='fine','cache':cache,'subcycle':False}
    out=[]
    def add(name,spec,build='ordinary'):out.append({'name':name,'spec':spec,'build':build})
    for control in ['Diffusion3','Diagonal','Shear3','Oblique','Vortex']:
        for n in [8,12,16]:
            for scheme in ['reference','combined']:add(f'{control}-{n}-{scheme}-uniform',make(control,n,scheme))
        for scheme in ['space','time']:add(f'{control}-12-{scheme}-uniform',make(control,12,scheme))
    for control in ['Diffusion3','Shear3','Oblique']:
        for n in [8,12,16]:add(f'{control}-{n}-combined-refined',make(control,n,'combined','refined'))
        add(f'{control}-12-reference-refined',make(control,12,'reference','refined'))
    for n in [32,64]:
        for direction in [-1,1]:
            for scheme in ['reference','combined']:
                for mesh in ['uniform','refined','fine']:
                    s=make('Equilibrium',n,scheme,dt=.0001,cycles=1200);s['config'].update(roots=[n,n//32,n//32],h=1/n);s.update(control={'Packet':{'direction':direction,'centre':.25 if direction==1 else .75,'width':.05,'amplitude':1e-4}},fine=mesh=='fine',refined=[[.5,0.,0.],[1.,1/32,1/32]]if mesh=='refined'else None)
                    add(f'packet-{n}-{direction}-{scheme}-{mesh}',s)
    for direction in [-1,1]:
        for mesh in ['refined','fine']:
            source=next(x['spec']for x in out if x['name']==f'packet-64-{direction}-combined-{mesh}');s=copy.deepcopy(source);s['control']['Packet']['width']=.025;add(f'narrow-64-{direction}-combined-{mesh}',s)
    for scheme in ['reference','combined']:
        s=copy.deepcopy(next(x['spec']for x in out if x['name']==f'packet-64-1-{scheme}-refined'));s['refined']=[[.4,0.,0.],[.6,1/32,1/32]];add(f'double-interface-64-{scheme}',s)
    for scheme in ['reference','space','combined']:add(f'steep-16-{scheme}',make('Steep',16,scheme,cycles=1000))
    for n in [8,12,16]:
        for scheme in ['reference','combined']:add(f'hydro-{n}-{scheme}',make('Hydrostatic',n,scheme))
    for repeat in range(3):
        for cache in [False,True]:add(f'cache-{repeat}-{cache}',make('Shear3',16,cache=cache))
    for repeat in range(3):
        for cache in [False,True]:add(f'cache-high-order-{repeat}-{cache}',make('Shear3',16,'combined',cache=cache))
    for dynamic in [False,True]:
        s=make('Diagonal',8,'combined',cycles=5000)
        if dynamic:s['config']['indicators']={'density':.001,'velocity':.01,'scalar':2e-5,'nonequilibrium':.01,'coarsen_fraction':.4,'quiet_cycles':8}
        add(f'physical-adaptation-{dynamic}',s)
    for name,dt,cycles,subcycle in [('coarse',.001,50,False),('fine',.0005,100,False),('subcycle',.001,50,True)]:
        s=make('Diffusion3',8,mesh='refined',dt=dt,cycles=cycles);s.update(subcycle=subcycle,cache=not subcycle);add(f'temporal-{name}',s)
    for dt in [.002,.001,.0005,.000125]:add(f'time-order-{dt}',make('Oblique',12,'combined',dt=dt,cycles=round(.05/dt)))
    for n in [8,16,24]:
        for mesh in ['uniform','refined']:
            s=make('Equilibrium',n,mesh=mesh,dt=.0001);s['config']['h']=.125;add(f'isotropic-{n}-{mesh}',s)
    add('sustained3D',make('Diagonal',8,'combined',cycles=5000),build='counted')
    s=make('Oblique',8,'combined');add('replay-A',s);add('replay-B',copy.deepcopy(s))
    return out


def read_state(p):return decode(json.loads(Path(p).read_text())['state'])

def audit(path):
    s=read_state(path);c=s['config'];cells={int(k):p['Fv']for k,p in s['cells'].items()};roots=math.prod(c['roots']);total=[0.]*8;volume=0.
    assert len(cells)<=c['capacity'] and not s['bodies']
    for root in range(roots):
        parent=root<<4;children=[parent|i for i in range(1,9)];assert (parent in cells and not any(x in cells for x in children))or(parent not in cells and all(x in cells for x in children))
    assert all(0<=k>>4<roots and k&15<=8 for k in cells)
    for key,u in sorted(cells.items()):
        assert len(u)==8 and all(math.isfinite(x)for x in u)and u[0]>0 and all(x>=0 for x in u[4:]);v=(c['h']/(2 if key&15 else 1))**3;volume+=v
        for k in range(8):total[k]+=u[k]
    assert abs(volume-roots*c['h']**3)<1e-11*max(1.,volume)
    for k in range(4):total[k+4]+=sum(s['ledger']['wall_scalar'][k])
    residual=[a-b for a,b in zip(total,s['ledger']['initial'])];sources=['gravity_impulse','walls_impulse','hydrostatic_support','manufactured_impulse']
    impulse=sum(math.sqrt(sum(x*x for x in s['ledger'].get(name,[0.]*3)))for name in sources)
    for k in range(3):residual[k+1]-=sum(s['ledger'].get(name,[0.]*3)[k]for name in sources)
    for k in [0,4,5,6,7]:assert abs(residual[k])<=1e-14+1e-10*abs(s['ledger']['initial'][k])
    for k in range(1,4):assert abs(residual[k])<=1e-10+1e-9*(abs(s['ledger']['initial'][k])+impulse)
    return {'leaves':len(cells),'faces_estimate_not_measured':None,'full_finest_cells':8*roots,'volume_m3':volume,'residual':residual,'total':total,'physical_time_s':s['cycle']*c['dt']}


def analytic_errors(spec,path):
    state=read_state(path);cfg=state['config'];t=state['cycle']*cfg['dt'];kind=spec['control'];k=2*math.pi;nu=cfg['viscosity']/cfg['density'];volume=math.prod(cfg['roots'])*cfg['h']**3;scalar=velocity=density=initial_scalar_error=0.
    def sinc(x):return math.sin(x)/x if x else 1.
    for raw,payload in state['cells'].items():
        key=int(raw);root=key>>4;child=max(0,(key&15)-1);h=cfg['h']/(2 if key&15 else 1);v=h**3;ijk=[root%cfg['roots'][0],root//cfg['roots'][0]%cfg['roots'][1],root//(cfg['roots'][0]*cfg['roots'][1])];p=[ijk[a]*cfg['h']+(((child>>a)&1)+.5)*h if key&15 else(ijk[a]+.5)*h for a in range(3)];u=payload['Fv'];rho=cfg['density'];target_rho=rho;target_velocity=[0.]*3;target_scalar=0.;average=sinc(k*h/2)
        if kind in ['Diffusion3','Diagonal']:
            target_velocity=[.03,.02,.01]if kind=='Diagonal'else[0.]*3;diff=cfg['species'][0]['diffusivity_m2_s'];mode=math.prod(math.cos(k*(p[a]-target_velocity[a]*t))*average for a in range(3));target_scalar=1e-4*(1+.5*mode*math.exp(-3*diff*k*k*t))
        elif kind=='Shear3':target_velocity=[a*math.sin(k*sum(p))*average**3*math.exp(-3*nu*k*k*t)for a in [.006,-.003,-.003]]
        elif kind=='Vortex':
            amplitude=.005*math.exp(-3*nu*k*k*t)*average**3;target_velocity=[amplitude*math.sin(k*p[0])*math.cos(k*p[1])*math.cos(k*p[2]),-amplitude*math.cos(k*p[0])*math.sin(k*p[1])*math.cos(k*p[2]),0.]
        elif kind=='Oblique':
            phase=k*sum(p);K=k*math.sqrt(3);relative=0.;momentum=.5e-8
            for harmonic,r0,v0 in [(1,1e-4,1e-4),(2,0.,.5e-8)]:
                frequency=K*harmonic;gamma=nu*frequency**2/2;omega=math.sqrt((cfg['sound_speed']*frequency)**2-gamma**2);E=math.exp(-gamma*t);C=math.cos(omega*t);S=math.sin(omega*t);mix=cfg['sound_speed']*frequency/omega*S;phase_h=phase*harmonic;aver=sinc(k*h*harmonic/2)**3
                relative+=E*((C+gamma/omega*S)*r0*math.cos(phase_h)+mix*v0*math.sin(phase_h))*aver
                momentum+=E*((C-gamma/omega*S)*v0*math.cos(phase_h)+mix*r0*math.sin(phase_h))*aver
            target_rho=rho*(1+relative);target_velocity=[rho*cfg['sound_speed']/math.sqrt(3)*momentum/target_rho]*3
        elif kind=='Hydrostatic':
            a=cfg['gravity'][1]/cfg['sound_speed']**2;z=a*h/2;target_rho=rho*math.exp(a*(p[1]-cfg['roots'][1]*cfg['h']/2))*(math.sinh(z)/z if z else 1.)
        elif isinstance(kind,dict)and'Packet'in kind:
            packet=kind['Packet'];centre=packet['centre']+packet['direction']*cfg['sound_speed']*t
            def avg_gauss(width):return width*math.sqrt(math.pi)/(2*h)*(math.erf((p[0]+h/2-centre)/width)-math.erf((p[0]-h/2-centre)/width))
            g=avg_gauss(packet['width']);g2=avg_gauss(packet['width']/math.sqrt(2));a=packet['amplitude'];target_rho=rho*(1+a*g);target_velocity[0]=rho*packet['direction']*cfg['sound_speed']*(a*g+a*a*g2)/target_rho
        elif kind=='Steep':target_velocity=[.03,0.,0.];lo=.25+.03*t;hi=.5+.03*t;target_scalar=1e-4*max(0.,min(p[0]+h/2,hi)-max(p[0]-h/2,lo))/h
        actual_velocity=[u[a+1]/u[0]for a in range(3)];scalar+=abs(u[4]/v-target_scalar)*v;velocity+=math.sqrt(sum((a-b)**2 for a,b in zip(actual_velocity,target_velocity)))*v;density+=(u[0]/v-target_rho)**2*v
    scale=math.sqrt(.006**2+2*.003**2)if kind=='Shear3'else cfg['sound_speed']*1e-4 if kind=='Oblique'else cfg['sound_speed']*kind['Packet']['amplitude']if isinstance(kind,dict)and'Packet'in kind else math.sqrt(.03**2+.02**2+.01**2)if kind=='Diagonal'else .005
    return {'density_L2_normalized':math.sqrt(density/volume)/(cfg['density']*1e-4),'scalar_L1_normalized':scalar/(5e-5*volume),'velocity_L1_normalized':velocity/(scale*volume)}


def profile1d(path):
    s=read_state(path);c=s['config'];points=[]
    for i in range(2*c['roots'][0]):
        pos=[(i+.5)*c['h']/2,c['roots'][1]*c['h']/2,c['roots'][2]*c['h']/2];ijk=[int(math.floor(x/c['h']))for x in pos];root=ijk[0]+c['roots'][0]*(ijk[1]+c['roots'][1]*ijk[2]);key=root<<4
        if str(key)not in s['cells']:key|=1+sum(int(pos[k]-ijk[k]*c['h']>=c['h']/2)<<k for k in range(3))
        u=s['cells'][str(key)]['Fv'];v=(c['h']/(2 if key&15 else 1))**3;delta=u[0]/v/c['density']-1;un=u[1]/u[0]/c['sound_speed'];points.append((pos[0],.5*(delta+un),.5*(delta-un)))
    return points


def scattering(native,fine,spec,coarse=None):
    a=profile1d(native);b=profile1d(fine);p=spec['control']['Packet'];direction=p['direction'];inc=1 if direction>0 else 2;ref=3-inc;amplitude=p['amplitude'];state=read_state(native);nx=spec['config']['roots'][0];h=spec['config']['h'];fine_x=[(int(k)>>4)%nx for k in state['cells']if int(k)&15];lo=min(fine_x)*h;hi=(max(fine_x)+1)*h;out_boundary=hi if direction>0 and hi<spec['config']['roots'][0]*h else lo;indices=[i for i,v in enumerate(a)if(v[0]>out_boundary if direction>0 else v[0]<lo)];peak_a=max(indices,key=lambda i:abs(a[i][inc]));peak_b=max(indices,key=lambda i:abs(b[i][inc]));T=abs(abs(a[peak_a][inc])-abs(b[peak_b][inc]))/max(abs(b[peak_b][inc]),1e-30);global_fine_R=max(abs(x[ref]-y[ref])for x,y in zip(a,b))/amplitude;coarse_profile=profile1d(coarse)if coarse is not None else b;upstream=coarse_profile if direction>0 else b;upstream_indices=[i for i,x in enumerate(a)if(x[0]<lo if direction>0 else x[0]>lo)];R=max(abs(a[i][ref]-upstream[i][ref])for i in upstream_indices)/amplitude if direction<0 or coarse is not None else None;shape=sum(abs(a[i][inc]-b[i][inc])for i in indices)/max(sum(abs(b[i][inc])for i in indices),1e-30);coarse_peak=max(abs(coarse_profile[i][inc])for i in indices);ordinary=abs(coarse_peak-abs(b[peak_b][inc]))/max(abs(b[peak_b][inc]),1e-30)if coarse is not None else None;phase=abs(a[peak_a][0]-b[peak_b][0]);h=spec['config']['h']
    def arrival(path):
        rows=[json.loads(line)for line in (path.parent/'ticks.jsonl').read_text().splitlines()];valid=[x for x in rows if x['metrics']['physical_time_s']>.02];row=max(valid,key=lambda x:abs(x['metrics']['probe_density_kg_m3']/spec['config']['density']-1));return row['metrics']['physical_time_s']
    arrival_error=abs(arrival(native)-arrival(fine));passed=R<=.10 and T<=.15 and phase<=h and arrival_error<=2*h/spec['config']['sound_speed']if R is not None else None
    return {'actual_refined_x_interval_m':[lo,hi],'status':'MEASURED'if R is not None else'UNRESOLVED_NO_MATCHED_UPSTREAM_REFERENCE','reflection_excess_relative_to_incident':R,'reflection_reference':'same upstream-resolution homogeneous control, upstream region only','global_reflected_characteristic_difference_vs_fine':global_fine_R,'outgoing_shape_L1_vs_fine':shape,'ordinary_coarse_vs_fine_peak_difference':ordinary,'transmission_peak_relative_error':T,'peak_displacement_m':phase,'arrival_error_s':arrival_error,'registered_bounds_pass':passed,'transmitted_peak_native':abs(a[peak_a][inc]),'transmitted_peak_fine':abs(b[peak_b][inc]),'interpretation':'linear characteristic diagnostics against homogeneous fine; not thermal-energy conservation or general oblique scattering'}


def run(root):
    root=Path(root);folder=root/'qualification';folder.mkdir(exist_ok=False);plan=recipes();(folder/'PLAN.json').write_text(json.dumps(plan,indent=2)+'\n');commands=[]
    for case in plan:
        spec_path=folder/(case['name']+'.spec.json');spec_path.write_text(json.dumps(case['spec'],indent=2)+'\n');selection=json.loads((root/'BUILD_SELECTION.json').read_text())if(root/'BUILD_SELECTION.json').exists()else{};binary=root/selection.get(case['build'],case['build']+'-certified')/'habitat_fidelity.binary';cmd=[str(binary),str(folder/case['name']),str(spec_path)];p=subprocess.run(cmd,capture_output=True,text=True,timeout=240)
        commands.append({'case':case['name'],'command':cmd,'exitcode':p.returncode,'stdout':p.stdout,'stderr':p.stderr});(folder/'COMMANDS.json').write_text(json.dumps(commands,indent=2)+'\n');print(case['name'],p.returncode,flush=True)
    return analyze(root)


def analyze(root):
    root=Path(root);folder=root/'qualification';plan=json.loads((folder/'PLAN.json').read_text());result=[]
    for case in plan:
        out=folder/case['name'];p=out/'summary.json'
        if not p.exists():result.append({'name':case['name'],'status':'FAILED EXECUTION'});continue
        summary=json.loads(p.read_text());rows=[json.loads(line)for line in(out/'ticks.jsonl').read_text().splitlines()];ms=[x['transaction_ns']/1e6 for x in rows];errors=summary['endpoint']['metrics']if summary['endpoint']else None
        c={'name':case['name'],'status':'COMPLETED'if summary['completed']==summary['requested']and summary['failure']is None else 'REJECTED','spec':case['spec'],'independent_endpoint':audit(out/'final.json'),'topology_counters':{k:read_state(out/'final.json')[k]for k in ['refinements','coarsenings']},'initial_metrics':summary['initial_metrics'],'metrics':errors,'independent_analytic_errors':analytic_errors(case['spec'],out/'final.json'),'timing':{'samples':len(ms),'mean_ms':statistics.mean(ms)if ms else None,'p99_floor_ms':sorted(ms)[int(.99*(len(ms)-1))]if ms else None},'restore_exact':summary['restore_exact'],'continuation_exact':summary['continuation_exact'],'cache_rebuilds':summary['cache_rebuilds'],'cache_hits':summary['cache_hits'],'cache_estimated_bytes':summary['cache_estimated_bytes'],'maximum_residual':[max((abs(row['diagnostics']['residual'][k])for row in rows),default=0.)for k in range(8)]}
        if errors:
            c['analytic_implementation_max_discrepancy']=max(abs(c['independent_analytic_errors'][k]-errors[k])for k in ['density_L2_normalized','scalar_L1_normalized','velocity_L1_normalized'])
            assert c['analytic_implementation_max_discrepancy']<1e-8,(case['name'],c['analytic_implementation_max_discrepancy'])
        if summary['failure']:c['failure']={k:v for k,v in summary['failure'].items()if k!='candidate'}
        if rows and rows[0]['allocation']['instrumented']:
            live=[x['allocation']['live_before']for x in rows[20:]];c['heap']={'steady_live_range':max(live)-min(live),'max_peak':max(x['allocation']['peak']for x in rows),'max_requested_per_transaction':max(x['allocation']['requested']for x in rows),'scope':'requested allocator only, not totalRSS/VRAM'}
        result.append(c)
    scatter=[]
    for case in plan:
        name=case['name']
        if name.startswith('packet-')and name.endswith('refined'):reference=name.removesuffix('refined')+'fine'
        elif name.startswith('narrow-')and name.endswith('refined'):reference=name.removesuffix('refined')+'fine'
        elif name.startswith('double-interface-'):reference='packet-64-1-'+('combined'if'combined'in name else'reference')+'-fine'
        else:continue
        try:scatter.append({'case':name,'reference':reference,**scattering(folder/name/'final.json',folder/reference/'final.json',case['spec'],folder/(reference.removesuffix('fine')+'uniform')/'final.json'if(reference.removesuffix('fine')+'uniform')in {x['name']for x in plan}else None)})
        except(Exception)as e:scatter.append({'case':name,'status':'ANALYSIS FAILED','error':str(e)})
    exact=[]
    for i in range(3):
        a=folder/f'cache-{i}-False';b=folder/f'cache-{i}-True';exact.append({'pair':i,'complete_final_bytes_equal':(a/'final.json').read_bytes()==(b/'final.json').read_bytes(),'every_tick_digest_equal':[json.loads(x)['digest']for x in(a/'ticks.jsonl').read_text().splitlines()]==[json.loads(x)['digest']for x in(b/'ticks.jsonl').read_text().splitlines()]})
    for i in range(3):
        a=folder/f'cache-high-order-{i}-False';b=folder/f'cache-high-order-{i}-True';exact.append({'pair':'higher-order-'+str(i),'complete_final_bytes_equal':(a/'final.json').read_bytes()==(b/'final.json').read_bytes(),'every_tick_digest_equal':[json.loads(x)['digest']for x in(a/'ticks.jsonl').read_text().splitlines()]==[json.loads(x)['digest']for x in(b/'ticks.jsonl').read_text().splitlines()]})
    replay=(folder/'replay-A/final.json').read_bytes()==(folder/'replay-B/final.json').read_bytes();data={'cases':result,'scattering':scatter,'cached_rebuilt_exact':exact,'process_replay_exact':replay,'limits':'Independent endpoint audits and characteristic comparison; per-tick residuals are Rust telemetry. No nativeWitness or GPU atmosphere qualified.'};(root/'FINAL_ANALYSIS.json').write_text(json.dumps(data,indent=2)+'\n');return data

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['plan','run','analyze']);p.add_argument('root',nargs='?');a=p.parse_args()
    if a.action=='plan':print(json.dumps(recipes(),indent=2))
    elif a.action=='run':run(a.root)
    else:analyze(a.root)
