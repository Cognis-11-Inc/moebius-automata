"""Authored scalar-only single-resolution-interface falsification probe.
Not a Chamber backend, pressure solver or coupled multiresolution LBM.
"""
import json,math,time,sys
from pathlib import Path

def initial(edges):
 return [0.5*(math.erf((b-.4)/(.025*math.sqrt(2)))-math.erf((a-.4)/(.025*math.sqrt(2)))) for a,b in zip(edges,edges[1:])]
def evolve(edges,ticks):
 h=[b-a for a,b in zip(edges,edges[1:])];mass=initial(edges);start=sum(mass);rows=[];clock=time.perf_counter_ns()
 for tick in range(ticks):
  delta=[0.]*len(h)
  for i in range(len(h)):
   j=(i+1)%len(h);a=mass[i]/h[i];b=mass[j]/h[j];flux=.016666667*(.05*a+.0005*(a-b)/((h[i]+h[j])*.5));delta[i]-=flux;delta[j]+=flux
  mass=[m+d for m,d in zip(mass,delta)];assert min(mass)>=0
  rows.append({'tick':tick+1,'mass':sum(mass),'residual':sum(mass)-start,'min_kg_per_unit_area':min(mass)})
 return mass,rows,time.perf_counter_ns()-clock

def main(out):
 out.mkdir();fine=[i/80 for i in range(81)];mixed=[i/40 for i in range(21)]+[.5+i/80 for i in range(1,41)];a,ar,at=evolve(fine,240);b,br,bt=evolve(mixed,240)
 # Conservative overlap average onto fine cells; scalar mass has explicit volume.
 sampled=[]
 for lo,hi in zip(fine,fine[1:]):sampled.append(sum(m*max(0.,min(hi,y)-max(lo,x))/(y-x) for m,x,y in zip(b,mixed,mixed[1:])))
 l1=sum(abs(x-y) for x,y in zip(a,sampled));masserr=max(abs(r['residual']) for r in br);assert masserr<=1e-12
 # Independent local restriction/prolongation exact-inventory demonstration.
 children=[.25,.75];parent=sum(children);restored=[parent*.5,parent*.5];assert sum(restored)==sum(children)
 report={'status':'SCALAR_INTERFACE_ONLY','ticks':240,'time_s':240*.016666667,'fine_cells':80,'mixed_cells':60,'max_mass_drift':masserr,'l1_mass_error_against_uniform_fine':l1,'fine_elapsed_ns':at,'mixed_elapsed_ns':bt,'restriction_mass':parent,'prolonged_mass':sum(restored),'lost_subcell_detail_l1':sum(abs(x-y) for x,y in zip(children,restored)),'scope':'Shared conservative scalar flux across one coarse/fine interface; no D3Q19 remapping, carrier viscosity, acoustic reflection/transmission, temporal subcycling or adopted backend. Timing is Python probe, not environmental performance.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n');(out/'rows.json').write_text(json.dumps({'uniform':ar,'interface':br},indent=2)+'\n');(out/'final.json').write_text(json.dumps({'fine_edges':fine,'fine_mass':a,'mixed_edges':mixed,'mixed_mass':b,'projected_mass':sampled},indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main(Path(sys.argv[1]))
