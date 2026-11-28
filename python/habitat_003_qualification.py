#!/usr/bin/env python3
"""Preregistered CPU qualification runner and independent endpoint analysis.

Raw directories are immutable. Endpoint budgets are recomputed from leaf payloads,
not accepted from Rust telemetry. No canonical runtime or observation authority.
"""
import argparse
import json
import math
from pathlib import Path
import statistics
import struct
import subprocess


def decode(x):
    if isinstance(x,dict):
        if set(x)=={'$f64'}:
            v=struct.unpack('>d',bytes.fromhex(x['$f64']))[0]
            if not math.isfinite(v):raise ValueError('nonfinite checkpoint')
            return v
        return {k:decode(v) for k,v in x.items()}
    if isinstance(x,list):return [decode(v) for v in x]
    return x


def audit_checkpoint(path):
    wrapper=json.loads(Path(path).read_text());s=decode(wrapper['state']);c=s['config'];cells={int(k):v for k,v in s['cells'].items()}
    roots=math.prod(c['roots']);h=c['h'];assert len(cells)<=c['capacity'];total=[0.]*8;volume=0.;negatives=0
    for root in range(roots):
        key=root<<4;children=[key|i for i in range(1,9)]
        assert (key in cells and not any(k in cells for k in children)) or (key not in cells and all(k in cells for k in children)), 'overlapping or missing inventory'
    assert all(0<=key>>4<roots and key&15<=8 for key in cells)
    for key,p in sorted(cells.items()):
        v=(h/(2 if key&15 else 1))**3;volume+=v
        if 'Fv' in p:q=p['Fv'];assert len(q)==8;negatives+=sum(a<0 for a in [q[0],*q[4:]])
        else:
            f=p['Lbm'];assert len(f)==19;negatives+=sum(a<0 for a in f);rho=sum(f)
            directions=[(0,0,0),(1,0,0),(-1,0,0),(0,1,0),(0,-1,0),(0,0,1),(0,0,-1),(1,1,0),(-1,-1,0),(1,-1,0),(-1,1,0),(1,0,1),(-1,0,-1),(1,0,-1),(-1,0,1),(0,1,1),(0,-1,-1),(0,1,-1),(0,-1,1)]
            q=[rho*v]+[sum(a*d[k]for a,d in zip(f,directions))*v*h/c['dt'] for k in range(3)]+[0.]*4
        assert all(math.isfinite(a)for a in q)
        for k,a in enumerate(q):total[k]+=a
    assert abs(volume-roots*h**3)<=1e-12*max(1.,volume)
    assert not s['bodies'],'unqualified body payload'
    for k in range(4):total[k+4]+=sum(s['ledger']['wall_scalar'][k])
    residual=[a-b for a,b in zip(total,s['ledger']['initial'])]
    for k in range(3):residual[k+1]-=sum(s['ledger'][name][k] for name in ['gravity_impulse','walls_impulse','hydrostatic_support'])
    assert negatives==0
    assert abs(residual[0])<=1e-14+1e-10*abs(s['ledger']['initial'][0])
    for k in range(4,8):assert abs(residual[k])<=1e-14+1e-10*abs(s['ledger']['initial'][k])
    impulse=sum(math.sqrt(sum(x*x for x in s['ledger'][name])) for name in ['gravity_impulse','walls_impulse','hydrostatic_support'])
    for k in range(1,4):assert abs(residual[k])<=1e-10+1e-9*(abs(s['ledger']['initial'][k])+impulse)
    return {'leaves':len(cells),'root_cells':roots,'full_finest_cells':roots*8,'volume_m3':volume,'total':total,'residual':residual,'physical_time_s':s['cycle']*c['dt'],'cycle':s['cycle'],'duplicate_inventory':False}


def plan():
    cases=[]
    def add(name,scenario='Equilibrium',n=32,dt=.0002,ticks=100,refined='uniform',temporal='mono',species=0,h=None,build='ordinary'):
        cases.append(dict(name=name,scenario=scenario,model='FiniteVolume',n=n,dt=dt,ticks=ticks,refined=refined,temporal=temporal,species=species,h=h,build=build))
    modes=[('uniform','mono'),('refined','mono'),('refined','subcycle')]
    physical_modes=modes+[('fine','mono')]
    for scenario in ['Diffusion','Advection','Shear','Wave']:
        for n in [16,32,64]:
            for spatial,temporal in physical_modes:add(f'{scenario.lower()}-{n}-{spatial}-{temporal}',scenario,n,dt=.0001 if scenario=='Wave' else .0002,ticks=1000,refined=spatial,temporal=temporal,species=int(scenario in ['Diffusion','Advection']))
    for n in [16,32,64]:add(f'hydrostatic-{n}','Hydrostatic',n,ticks=1000)
    for spatial,temporal in modes:
        add(f'equilibrium-{spatial}-{temporal}',refined=spatial,temporal=temporal)
        for scenario in ['Disturbance','Interacting']:add(f'{scenario.lower()}-{spatial}-{temporal}',scenario,ticks=1000,refined=spatial,temporal=temporal)
    for dt,ticks in [(.0001,2000),(.0004,500)]:add(f'temporal-shear-{dt}','Shear',dt=dt,ticks=ticks,refined='refined',temporal='subcycle')
    for n in [32,64,128,256,512]:add(f'domain-{n}','Disturbance',n,refined='refined',h=1/32)
    for species in [1,2,4]:add(f'species-{species}','Diffusion',species=species,refined='refined')
    add('sustained','Advection',n=16,ticks=5000,refined='refined',temporal='subcycle',species=1,build='counted')
    for model in ['FiniteVolume','AdaptiveLbm']:
        name='replay-control' if model=='FiniteVolume' else 'lbm-interface-failure'
        add(name,'Wave',n=16,dt=.001 if model=='AdaptiveLbm' else .0001,ticks=100,refined='refined');cases[-1]['model']=model
    # Deliberately repeated identical runs: independent process and complete trace equality.
    add('replay-repeat','Wave',n=16,dt=.0001,ticks=100,refined='refined')
    return cases


def run(root):
    root=Path(root);out=root/'qualification';out.mkdir(exist_ok=False)
    cases=plan();(out/'PLAN.json').write_text(json.dumps(cases,indent=2)+'\n');commands=[]
    for c in cases:
        directory=root/(c['build']+'-certified');binary=directory/'habitat_scientific.binary'
        command=[str(binary),str(out/c['name']),c['scenario'],c['model'],str(c['n']),str(c['dt']),str(c['ticks']),c['refined'],c['temporal'],str(c['species'])]
        if c['h'] is not None:command.append(str(c['h']))
        p=subprocess.run(command,capture_output=True,text=True,timeout=180)
        commands.append({'case':c['name'],'command':command,'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
        (out/'COMMANDS.json').write_text(json.dumps(commands,indent=2)+'\n')
        print(c['name'],p.returncode,flush=True)
    return analyze(root)


def compare_fields(native_path,fine_path,scenario):
    def state(path):return decode(json.loads(Path(path).read_text())['state'])
    native=state(native_path);fine=state(fine_path);c=native['config'];assert c==fine['config'] and native['cycle']==fine['cycle']
    def geometry(key):
        root=key>>4;id_=key&15;index=[root%c['roots'][0],root//c['roots'][0]%c['roots'][1],root//(c['roots'][0]*c['roots'][1])];child=max(0,id_-1);h=c['h']/(2 if id_ else 1)
        p=[v*c['h']+((((child>>k)&1)+.5)*h if id_ else .5*h)for k,v in enumerate(index)]
        return p,h**3
    cells={int(k):v['Fv']for k,v in native['cells'].items()};error=0.;scale=0.
    for raw,payload in fine['cells'].items():
        point,v=geometry(int(raw));ijk=[math.floor(x/c['h'])for x in point];root=ijk[0]+c['roots'][0]*(ijk[1]+c['roots'][1]*ijk[2]);key=root<<4
        if key not in cells:key|=1+sum(int(point[k]-ijk[k]*c['h']>=c['h']/2)<<k for k in range(3))
        _,nv=geometry(key);u=cells[key];f=payload['Fv']
        if scenario in ['diffusion','advection']:a=u[4]/nv;b=f[4]/v;denom=5e-5
        elif scenario=='shear':a=u[2]/u[0];b=f[2]/f[0];denom=.01
        else:a=u[0]/nv;b=f[0]/v;denom=c['density']*1e-4
        error+=abs(a-b)*v;scale+=denom*v
    return {'normalized_L1_against_uniform_fine':error/scale,'comparison':'Category II; piecewise constant diagnostic projection, no native finest shadow'}


def analyze(root):
    root=Path(root);out=root/'qualification';records=[]
    for c in json.loads((out/'PLAN.json').read_text()):
        p=out/c['name'];summary_path=p/'summary.json'
        if not summary_path.exists():records.append({'name':c['name'],'status':'FAILED EXECUTION'});continue
        summary=json.loads(summary_path.read_text());audit=audit_checkpoint(p/'final.json')
        rows=[json.loads(x)for x in (p/'profiles.jsonl').read_text().splitlines()];ms=[r['transaction_ns']/1e6 for r in rows]
        traces=[json.loads(x)for x in (p/'traces.jsonl').read_text().splitlines()]
        status='COMPLETED' if summary['failure'] is None and summary['completed_cycles']==c['ticks'] else 'REJECTED'
        data={'name':c['name'],'configuration':c,'status':status,'endpoint_audit':audit,'endpoint_errors':summary['endpoint']['errors'] if summary['endpoint'] else None,'restore_exact':summary['restore_exact'],'continuation_exact':summary['continuation_exact'],'pending_restoration':summary['pending_restoration'],'timing':{'observations':len(ms),'mean_ms':statistics.mean(ms) if ms else None,'p99_floor_ms':sorted(ms)[int(.99*(len(ms)-1))] if ms else None},'maximum_trace_residual':[max((abs(t['diagnostics']['residual'][k]) for t in traces),default=0.)for k in range(8)]}
        if summary['failure']:data['rejection']={k:v for k,v in summary['failure'].items()if k!='candidate'}
        if rows and rows[0]['allocation']['instrumented']:
            data['allocation']={'first_live_before':rows[0]['allocation']['live_before'],'last_live_before':rows[-1]['allocation']['live_before'],'steady_live_before_range':max(x['allocation']['live_before'] for x in rows[20:])-min(x['allocation']['live_before']for x in rows[20:]),'max_requested_per_transaction':max(x['allocation']['requested_delta']for x in rows),'max_peak':max(x['allocation']['peak']for x in rows)}
        records.append(data)
    a=out/'replay-control';b=out/'replay-repeat';replay={'full_trace_equal':(a/'traces.jsonl').read_bytes()==(b/'traces.jsonl').read_bytes(),'full_final_equal':(a/'final.json').read_bytes()==(b/'final.json').read_bytes()}
    comparisons=[]
    for scenario in ['diffusion','advection','shear','wave']:
        for n in [16,32,64]:
            for temporal in ['mono','subcycle']:
                name=f'{scenario}-{n}-refined-{temporal}';fine=f'{scenario}-{n}-fine-mono'
                comparisons.append({'case':name,'reference':fine,**compare_fields(out/name/'final.json',out/fine/'final.json',scenario)})
    result={'schema':1,'uniform_fine_comparisons':comparisons,'exactness':'same-source CPU replay Category I; refined/FV accuracy Category II; legacy LBM versus FV Category III; failed adaptive LBM Category IV','cases':records,'independent_replay':replay,'limits':'Independent endpoint inventory/partition audits; every-tick residuals from Rust telemetry. No independent per-tick payload archive, native Witness or GPU qualification.'}
    (root/'FINAL_ANALYSIS.json').write_text(json.dumps(result,indent=2)+'\n');return result

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['plan','run','analyze']);p.add_argument('root',nargs='?');a=p.parse_args()
    if a.action=='plan':print(json.dumps(plan(),indent=2))
    elif a.action=='run':run(a.root)
    else:analyze(a.root)
