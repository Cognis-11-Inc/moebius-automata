"""HABITAT-002 detached FP64 carrier experiment, not a Chamber authority.
Uses installed OpenCL via ctypes; no imported solver code or new Python package.
Input checkpoints are fixtures verified against their captured executable identity.
"""
import array,ctypes as c,hashlib,json,math,os,struct,time
from pathlib import Path
KERNEL = r'''
#pragma OPENCL EXTENSION cl_khr_fp64 : enable
#pragma OPENCL FP_CONTRACT OFF
__constant int vx[19]={0,1,-1,0,0,0,0,1,-1,1,-1,1,-1,1,-1,0,0,0,0};
__constant int vy[19]={0,0,0,1,-1,0,0,1,-1,-1,1,0,0,0,0,1,-1,1,-1};
__constant int vz[19]={0,0,0,0,0,1,-1,0,0,0,0,1,-1,-1,1,1,-1,-1,1};
__constant int opp[19]={0,2,1,4,3,6,5,8,7,10,9,12,11,14,13,16,15,18,17};
__kernel void collide(__global const double* f,__global double* post,double tau,int total){
 int i=get_global_id(0);if(i>=total)return;double rho=0.,ux=0.,uy=0.,uz=0.;
 for(int q=0;q<19;q++){double v=f[i*19+q];rho+=v;ux+=v*(double)vx[q];uy+=v*(double)vy[q];uz+=v*(double)vz[q];}
 double inverse=1./rho;ux*=inverse;uy*=inverse;uz*=inverse;
 double u2=0.;u2+=ux*ux;u2+=uy*uy;u2+=uz*uz;
 for(int q=0;q<19;q++){double cu=0.;cu+=(double)vx[q]*ux;cu+=(double)vy[q]*uy;cu+=(double)vz[q]*uz;
 double w=q==0?1./3.:(q<7?1./18.:1./36.);double eq=w*rho*(1.+3.*cu+4.5*cu*cu-1.5*u2);double value=f[i*19+q];post[i*19+q]=value+(eq-value)/tau;}
}
__kernel void stream(__global const double* post,__global double* out,int n,int sealed){
 int i=get_global_id(0);if(i>=n*n*n)return;int x=i%n,y=i/n%n,z=i/(n*n);
 for(int q=0;q<19;q++){int a=x-vx[q],b=y-vy[q],d=z-vz[q];
 if(sealed&&(a<0||b<0||d<0||a>=n||b>=n||d>=n))out[i*19+q]=0.+post[i*19+opp[q]];
 else {int src=(a+n)%n+n*((b+n)%n+n*((d+n)%n));out[i*19+q]=0.+post[src*19+q];}}
}
__kernel void validate(__global const double* f,__global int* bad,int total,double rho0){
 int i=get_global_id(0);if(i>=total)return;double rho=0.,u[3]={0.,0.,0.};int invalid=0;
 for(int q=0;q<19;q++){double v=f[i*19+q];invalid|=!isfinite(v)||v<0.;rho+=v;u[0]+=v*vx[q];u[1]+=v*vy[q];u[2]+=v*vz[q];}
 double inverse=1./rho;for(int k=0;k<3;k++)u[k]*=inverse;double v2=0.;for(int k=0;k<3;k++)v2+=u[k]*u[k];
 bad[i]=invalid||!isfinite(rho)||fabs(rho/rho0-1.)>0.05||sqrt(v2)*sqrt(3.)>0.1;
}
'''
P=c.c_void_p; I=c.c_int; U=c.c_uint; Z=c.c_size_t; Q=c.c_ulonglong

def decode(x):
 if isinstance(x,dict):
  if set(x)=={'$f64'}:return struct.unpack('>d',bytes.fromhex(x['$f64']))[0]
  return {k:decode(v) for k,v in x.items()}
 if isinstance(x,list):return [decode(v) for v in x]
 return x

def check(code):
 if code:raise RuntimeError(f'OpenCL error {code}')

class Device:
 def __init__(self):
  self.cl=c.CDLL('libOpenCL.so.1');self.handles=[]
  signatures={
   'clGetPlatformIDs':(I,[U,P,P]),'clGetDeviceIDs':(I,[P,Q,U,P,P]),'clGetDeviceInfo':(I,[P,U,Z,P,P]),
   'clCreateContext':(P,[P,U,P,P,P,P]),'clCreateCommandQueue':(P,[P,P,Q,P]),
   'clCreateProgramWithSource':(P,[P,U,P,P,P]),'clBuildProgram':(I,[P,U,P,c.c_char_p,P,P]),'clGetProgramBuildInfo':(I,[P,P,U,Z,P,P]),
   'clCreateKernel':(P,[P,c.c_char_p,P]),'clCreateBuffer':(P,[P,Q,Z,P,P]),'clSetKernelArg':(I,[P,U,Z,P]),
   'clEnqueueWriteBuffer':(I,[P,P,U,Z,Z,P,U,P,P]),'clEnqueueReadBuffer':(I,[P,P,U,Z,Z,P,U,P,P]),
   'clEnqueueNDRangeKernel':(I,[P,P,U,P,P,P,U,P,P]),'clFinish':(I,[P]),'clGetEventProfilingInfo':(I,[P,U,Z,P,P]),
   'clReleaseMemObject':(I,[P]),'clReleaseKernel':(I,[P]),'clReleaseProgram':(I,[P]),'clReleaseCommandQueue':(I,[P]),'clReleaseContext':(I,[P]),'clReleaseEvent':(I,[P])}
  for name,(ret,args) in signatures.items():fn=getattr(self.cl,name);fn.restype=ret;fn.argtypes=args
  platform=P();device=P();check(self.cl.clGetPlatformIDs(1,c.byref(platform),None));check(self.cl.clGetDeviceIDs(platform,4,1,c.byref(device),None));self.device=device
  def info(k,size=4096):
   b=c.create_string_buffer(size);check(self.cl.clGetDeviceInfo(device,k,size,b,None));return b.raw
  self.identity={'device':info(0x102b).split(b'\0')[0].decode(),'driver':info(0x102d).split(b'\0')[0].decode(),'global_memory_bytes':struct.unpack('Q',info(0x101f,8))[0],'double_fp_config':struct.unpack('Q',info(0x1032,8))[0],'api':'OpenCL','precision':'FP64','build_options':'-cl-std=CL1.2','FP_CONTRACT':'OFF','kernel_sha256':hashlib.sha256(KERNEL.encode()).hexdigest(),'runner_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
  err=I();self.context=self.cl.clCreateContext(None,1,c.byref(device),None,None,c.byref(err));check(err.value);self.queue=self.cl.clCreateCommandQueue(self.context,device,2,c.byref(err));check(err.value)
  source=c.c_char_p(KERNEL.encode());self.program=self.cl.clCreateProgramWithSource(self.context,1,c.byref(source),None,c.byref(err));check(err.value)
  code=self.cl.clBuildProgram(self.program,1,c.byref(device),b'-cl-std=CL1.2',None,None)
  log=c.create_string_buffer(65536);self.cl.clGetProgramBuildInfo(self.program,device,0x1183,len(log),log,None);self.identity['build_log']=log.value.decode()
  check(code);self.kernels={}
  for name in ['collide','stream','validate']:
   k=self.cl.clCreateKernel(self.program,name.encode(),c.byref(err));check(err.value);self.kernels[name]=k
 def buffer(self,size):
  err=I();b=self.cl.clCreateBuffer(self.context,1,size,None,c.byref(err));check(err.value);self.handles.append(b);return b
 def upload(self,b,data):check(self.cl.clEnqueueWriteBuffer(self.queue,b,1,0,len(data),c.c_char_p(data),0,None,None))
 def download(self,b,size):
  out=c.create_string_buffer(size);check(self.cl.clEnqueueReadBuffer(self.queue,b,1,0,size,out,0,None,None));return out.raw
 def dispatch(self,name,args,total):
  k=self.kernels[name]
  for i,a in enumerate(args):check(self.cl.clSetKernelArg(k,i,c.sizeof(a),c.byref(a)))
  event=P();global_size=Z(total);start=time.perf_counter_ns();check(self.cl.clEnqueueNDRangeKernel(self.queue,k,1,None,c.byref(global_size),None,0,None,c.byref(event)));dispatch=time.perf_counter_ns()-start
  return event,dispatch
 def event_ns(self,e):
  start=Q();end=Q();check(self.cl.clGetEventProfilingInfo(e,0x1282,8,c.byref(start),None));check(self.cl.clGetEventProfilingInfo(e,0x1283,8,c.byref(end),None));check(self.cl.clReleaseEvent(e));return end.value-start.value
 def close(self):
  for b in self.handles:self.cl.clReleaseMemObject(b)
  for k in self.kernels.values():self.cl.clReleaseKernel(k)
  self.cl.clReleaseProgram(self.program);self.cl.clReleaseCommandQueue(self.queue);self.cl.clReleaseContext(self.context)

def restore_checkpoint(folder,backend,cpu_source):
 header=json.loads((folder/'checkpoint.json').read_text())
 if header.get('schema')!='HABITAT-002/GPU-carrier-fixture-v1' or header.get('backend')!=backend or header.get('cpu_source_identity')!=cpu_source or header.get('config')!=backend.get('numerical_config'):
  raise ValueError('GPU checkpoint source/backend/numerical identity mismatch')
 payload=(folder/'carrier.f64le').read_bytes()
 if len(payload)>64*1024*1024 or hashlib.sha256(payload).hexdigest()!=header['payload_sha256']:
  raise ValueError('GPU checkpoint payload mismatch')
 return payload

def run(initial,final,build,out):
 fixture=json.loads(initial.read_text());expected=json.loads(final.read_text());receipt=json.loads(build.read_text())
 assert json.loads(fixture['source_identity'])==receipt['identity'],'CPU fixture source identity mismatch'
 assert expected['source_identity']==fixture['source_identity']
 assert fixture['execution']=={'backend':'CPU','precision':'f64','lane':'Reference','workers':1},'GPU fixture execution contract mismatch'
 s=decode(fixture['state']);target=decode(expected['state']);cfg=s['config'];assert cfg['density_kg_m3']>0 and not s['spheres'] and cfg['substeps']==1
 n=cfg['cells'];total=n**3;dt=.016666667;tau=.5+3*cfg['viscosity_pa_s']/cfg['density_kg_m3']*dt/(cfg['length_m']/n)**2;ticks=target['tick']-s['tick'];assert 0<ticks<=16384
 data=array.array('d',(x for cell in s['populations'] for x in cell)).tobytes();out.mkdir();d=Device();d.identity.update(numerical_config=cfg,dt_s=dt,tau=tau);size=len(data);a=d.buffer(size);post=d.buffer(size);b=d.buffer(size);bad=d.buffer(total*4)
 t=time.perf_counter_ns();d.upload(a,data);upload=time.perf_counter_ns()-t;rows=[]
 def evolve(start,ticks):
  current=a;next_b=b;d.upload(current,start);observations=[]
  for tick in range(ticks):
   t=time.perf_counter_ns();events=[];dispatch=0
   for name,args in [('collide',[P(current),P(post),c.c_double(tau),I(total)]),('stream',[P(post),P(next_b),I(n),I(cfg['boundary']=='Sealed')]),('validate',[P(next_b),P(bad),I(total),c.c_double(cfg['density_kg_m3'])])]:
    e,ns=d.dispatch(name,args,total);events.append(e);dispatch+=ns
   sync=time.perf_counter_ns();flags=d.download(bad,total*4);sync=time.perf_counter_ns()-sync;assert not any(flags),'GPU invalid tentative state (buffers not admitted)'
   wall=time.perf_counter_ns()-t;kernel=[d.event_ns(e) for e in events];observations.append({'tick':tick+1,'kernel_ns':kernel,'enqueue_ns':dispatch,'synchronize_and_status_download_ns':sync,'carrier_host_wall_ns':wall});current,next_b=next_b,current
  t=time.perf_counter_ns();result=d.download(current,size);checkpoint=time.perf_counter_ns()-t;return result,observations,checkpoint
 result,rows,checkpoint=evolve(data,ticks);repeat,_,_=evolve(data,ticks);assert result==repeat,'same-device replay differs'
 values=array.array('d');values.frombytes(result);cpu=[x for cell in target['populations'] for x in cell];maxerr=max(abs(a-b) for a,b in zip(values,cpu));vmax=0.;mass=0.;initialmass=0.
 for i in range(total):
  f=list(values[i*19:i*19+19]);g=target['populations'][i];rho=sum(f);mass+=rho*(cfg['length_m']/n)**3;initialmass+=sum(s['populations'][i])*(cfg['length_m']/n)**3
  for k in range(3):
   vectors=[[0,1,-1,0,0,0,0,1,-1,1,-1,1,-1,1,-1,0,0,0,0],[0,0,0,1,-1,0,0,1,-1,-1,1,0,0,0,0,1,-1,1,-1],[0,0,0,0,0,1,-1,0,0,0,0,1,-1,-1,1,1,-1,-1,1]]
   va=sum(f[q]*vectors[k][q] for q in range(19))/rho;vb=sum(g[q]*vectors[k][q] for q in range(19))/sum(g);vmax=max(vmax,abs(va-vb)*(cfg['length_m']/n)/dt)
 envelope={'schema':'HABITAT-002/GPU-carrier-fixture-v1','cpu_source_identity':fixture['source_identity'],'backend':d.identity,'config':cfg,'ticks':ticks,'payload_sha256':hashlib.sha256(result).hexdigest()};(out/'carrier.f64le').write_bytes(result);(out/'checkpoint.json').write_text(json.dumps(envelope,indent=2)+'\n')
 # Backend/source-bound restoration with an intentionally changed identity refusal.
 restored_data=restore_checkpoint(out,d.identity,fixture['source_identity'])
 altered=dict(d.identity,precision='FP32')
 try:restore_checkpoint(out,altered,fixture['source_identity'])
 except ValueError:pass
 else:raise AssertionError('Changed GPU numerical identity was not refused')
 try:restore_checkpoint(out,d.identity,'changed-source')
 except ValueError:pass
 else:raise AssertionError('Changed CPU source was not refused')
 continuation,_,_=evolve(restored_data,3);direct,_,_=evolve(data,ticks+3);assert continuation==direct
 report={'status':'PASS_TOLERANCE_CARRIER_ONLY' if maxerr<=1e-10 and vmax<=1e-10 and abs(mass-initialmass)<=initialmass*1e-10 else 'FAIL','backend':d.identity,'n':n,'ticks':ticks,'same_backend_replay_exact':True,'restored_continuation_exact':True,'changed_backend_identity_refused':True,'cross_backend_population_bits_exact':result==array.array('d',cpu).tobytes(),'max_population_abs_error':maxerr,'max_velocity_abs_error_m_s':vmax,'mass_residual_kg':mass-initialmass,'upload_ns':upload,'checkpoint_download_ns':checkpoint,'allocated_device_bytes':size*3+total*4,'rows':rows,'scope':'Detached carrier collide/stream/local validation only. CPU fields, spheres, ordered wall impulse and full Chamber transactional commit NOT GPU-integrated. No authoritative full-environment GPU timing or adoption claim. Device-resident f/post/output; local status buffer read each tick; global inventory checked at final qualification only.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n');d.close();return report
if __name__=='__main__':
 import sys
 report=run(*(Path(x) for x in sys.argv[1:]));print(json.dumps({k:v for k,v in report.items() if k not in ['rows','backend']},indent=2))
