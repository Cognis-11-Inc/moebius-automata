#!/usr/bin/env python3
"""Verified detached OpenCL arithmetic characterization; no FV owner or driver edits.
Reuses the existing ctypes context/queue/buffer/event interface unchanged. Its old
LBM program is built as an untimed prerequisite but never numerically executed.
"""
import array
import ctypes as c
import hashlib
import json
from pathlib import Path
import struct
import time
import habitat_002_gpu as cl

KERNEL=r'''
#pragma OPENCL EXTENSION cl_khr_fp64 : enable
#pragma OPENCL FP_CONTRACT OFF
__kernel void micro_f64(__global const double *a,__global double *b,int n,int loops){int i=get_global_id(0);if(i>=n)return;double x=a[i];for(int j=0;j<loops;j++)x=x*1.000001+0.125;b[i]=x;}
__kernel void micro_f32(__global const float *a,__global float *b,int n,int loops){int i=get_global_id(0);if(i>=n)return;float x=a[i];for(int j=0;j<loops;j++)x=x*1.000001f+0.125f;b[i]=x;}
__kernel void micro_copy(__global const double *a,__global double *b,int n){int i=get_global_id(0);if(i<n)b[i]=a[i];}
'''

def rounded32(x):return struct.unpack('f',struct.pack('f',x))[0]
def run(out):
    out=Path(out);out.mkdir(exist_ok=False);begin=time.perf_counter_ns();d=cl.Device();interface_setup=time.perf_counter_ns()-begin;program=None;records=[];allocated=0
    try:
        err=cl.I();src=c.c_char_p(KERNEL.encode());t=time.perf_counter_ns();program=d.cl.clCreateProgramWithSource(d.context,1,c.byref(src),None,c.byref(err));cl.check(err.value);code=d.cl.clBuildProgram(program,1,c.byref(d.device),b'-cl-std=CL1.2',None,None);log=c.create_string_buffer(65536);d.cl.clGetProgramBuildInfo(program,d.device,0x1183,len(log),log,None);(out/'build.log').write_text(log.value.decode());cl.check(code);build_ns=time.perf_counter_ns()-t
        for name in ['micro_f64','micro_f32','micro_copy']:
            kernel=d.cl.clCreateKernel(program,name.encode(),c.byref(err));cl.check(err.value);d.kernels[name]=kernel
        for dtype,name in [('d','micro_f64'),('f','micro_f32'),('d','micro_copy')]:
            item=8 if dtype=='d'else 4
            for n in [16384,262144,1048576]:
                size=n*item;assert allocated+2*size<=min(256*1024*1024,d.identity['global_memory_bytes']//16),'explicit resident-buffer capacity';a=d.buffer(size);b=d.buffer(size);allocated+=2*size;payload=array.array(dtype,[.25])*n;t=time.perf_counter_ns();d.upload(a,payload.tobytes());upload=time.perf_counter_ns()-t;rows=[];loops=1 if name=='micro_copy'else 64
                for _ in range(3):
                    t=time.perf_counter_ns();args=[cl.P(a),cl.P(b),cl.I(n)]+([]if name=='micro_copy'else[cl.I(loops)]);event,dispatch=d.dispatch(name,args,n);cl.check(d.cl.clFinish(d.queue));wall=time.perf_counter_ns()-t;rows.append({'host_dispatch_finish_ns':wall,'enqueue_ns':dispatch,'kernel_ns':d.event_ns(event)});a,b=b,a
                t=time.perf_counter_ns();actual=d.download(a,size);download=time.perf_counter_ns()-t;expected=.25
                if name!='micro_copy':
                    for _ in range(3*loops):
                        if dtype=='f':expected=rounded32(rounded32(expected*rounded32(1.000001))+.125)
                        else:expected=expected*1.000001+.125
                expected_bytes=struct.pack(dtype,expected)*n;exact=actual==expected_bytes;assert exact,(name,n,'scalar reference mismatch')
                file=out/f'{name}-{n}.payload';file.write_bytes(actual);records.append({'kernel':name,'precision':'FP64'if dtype=='d'else'FP32','elements':n,'loops_per_dispatch':loops,'rows':rows,'upload_host_ns':upload,'download_host_ns':download,'payload_bytes':size,'allocated_buffers_cumulative':allocated,'derived_flops_per_dispatch':2*n*loops if name!='micro_copy'else 0,'nominal_payload_traffic_per_dispatch':2*size,'expected_scalar':expected,'exact_scalar_reference':exact,'payload':file.name,'sha256':hashlib.sha256(actual).hexdigest()})
        identity=dict(d.identity);identity.update(experiment='HABITAT-004/detached-hardware-micro-v1',precision='separate FP32/FP64',kernel_sha256=hashlib.sha256(KERNEL.encode()).hexdigest(),runner_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),host_interface_sha256=hashlib.sha256(Path(cl.__file__).read_bytes()).hexdigest())
        result={'status':'VERIFIED_MICROBENCHMARKS_ONLY','backend':identity,'old_interface_program_setup_ns':interface_setup,'new_program_build_ns':build_ns,'records':records,'maximum_explicit_device_buffers_bytes':allocated,'limits':'No full FV or body/scalar atmosphere transaction; no mixed precision; theoretical flops/payload traffic are derived, not whole-machine ceilings. NVML residency/thermal telemetry unavailable. No concurrent renderer qualified.'};(out/'REPORT.json').write_text(json.dumps(result,indent=2)+'\n');return result
    finally:
        d.close()
        if program:d.cl.clReleaseProgram(program)

if __name__=='__main__':
    import sys
    if len(sys.argv)!=2:raise SystemExit('NEW_OUTPUT_DIR')
    if Path(sys.argv[1]).exists():raise SystemExit('immutable output directory exists')
    try:result=run(sys.argv[1]);print(result['status'])
    except Exception as e:
        p=Path(sys.argv[1]);p.mkdir(exist_ok=True);(p/'FAILURE.json').write_text(json.dumps({'status':'FAILED','error':str(e)},indent=2)+'\n');raise
