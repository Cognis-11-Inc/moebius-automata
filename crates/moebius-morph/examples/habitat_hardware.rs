//! HABITAT-004 bounded verified arithmetic/memory probes, no simulation authority.
use serde_json::{Value, json};
use std::{hint::black_box, sync::mpsc, thread, time::Instant};
fn seed(n: usize, offset: usize) -> Vec<f64> {
    (0..n)
        .map(|i| 0.25 + ((i + offset) % 64) as f64 / 65536.)
        .collect()
}
#[inline(never)]
fn scalar(a: &mut [f64], passes: usize) {
    for _ in 0..passes {
        for x in a.iter_mut() {
            *x = black_box(*x) * 1.000001 + 0.125;
        }
    }
}
#[inline(never)]
fn portable(a: &mut [f64], passes: usize) {
    for _ in 0..passes {
        for x in a.iter_mut() {
            *x = *x * 1.000001 + 0.125;
        }
    }
}
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn avx2(a: &mut [f64], passes: usize) {
    use std::arch::x86_64::*;
    unsafe {
        let factor = _mm256_set1_pd(1.000001);
        let increment = _mm256_set1_pd(0.125);
        for _ in 0..passes {
            let mut i = 0;
            while i + 4 <= a.len() {
                let x = _mm256_loadu_pd(a.as_ptr().add(i));
                let y = _mm256_add_pd(_mm256_mul_pd(x, factor), increment);
                _mm256_storeu_pd(a.as_mut_ptr().add(i), y);
                i += 4;
            }
            for x in &mut a[i..] {
                *x = *x * 1.000001 + 0.125;
            }
        }
    }
}
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn avx2_f32(a: &mut [f32], passes: usize) {
    use std::arch::x86_64::*;
    unsafe {
        let factor = _mm256_set1_ps(1.000001);
        let increment = _mm256_set1_ps(0.125);
        for _ in 0..passes {
            let mut i = 0;
            while i + 8 <= a.len() {
                let x = _mm256_loadu_ps(a.as_ptr().add(i));
                let y = _mm256_add_ps(_mm256_mul_ps(x, factor), increment);
                _mm256_storeu_ps(a.as_mut_ptr().add(i), y);
                i += 8;
            }
            for x in &mut a[i..] {
                *x = *x * 1.000001 + 0.125;
            }
        }
    }
}
fn arithmetic(n: usize, kind: &str) -> Value {
    let passes = (8 * 1024 * 1024 / n).clamp(2, 64);
    let mut a = seed(n, 0);
    let mut reference = a.clone();
    let mut timings = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        match kind {
            "scalar-black-box" => scalar(&mut a, passes),
            "portable" => portable(&mut a, passes),
            "avx2" => {
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    avx2(&mut a, passes);
                }
            }
            _ => unreachable!(),
        }
        timings.push(start.elapsed().as_nanos());
    }
    scalar(&mut reference, passes * 3);
    assert!(
        a.iter()
            .zip(&reference)
            .all(|(a, b)| a.to_bits() == b.to_bits())
    );
    json!({"kind":kind,"precision":"FP64","elements":n,"working_array_bytes":n*8,"passes_per_window":passes,"windows_ns":timings,"derived_flops_per_window":n*passes*2,"nominal_read_write_bytes_per_window":n*passes*16,"exact_scalar_reference":true,"checksum":a.iter().sum::<f64>()})
}
fn fp32(n: usize) -> Value {
    let passes = (8 * 1024 * 1024 / n).clamp(2, 64);
    let mut a = vec![0.25f32; n];
    let mut timings = Vec::new();
    for _ in 0..3 {
        let t = Instant::now();
        #[cfg(target_arch = "x86_64")]
        unsafe {
            avx2_f32(&mut a, passes);
        }
        timings.push(t.elapsed().as_nanos());
    }
    let mut expected = 0.25f32;
    for _ in 0..passes * 3 {
        expected = black_box(expected) * 1.000001 + 0.125;
    }
    assert!(a.iter().all(|x| x.to_bits() == expected.to_bits()));
    json!({"kind":"avx2","precision":"FP32","elements":n,"working_array_bytes":n*4,"passes_per_window":passes,"windows_ns":timings,"derived_flops_per_window":n*passes*2,"exact_scalar_reference":true,"expected_scalar":expected})
}
fn access(n: usize, kind: &str) -> Value {
    let mut a = seed(n, 0);
    let b = a.clone();
    let passes = (8 * 1024 * 1024 / n).clamp(2, 64);
    let mut windows = Vec::new();
    let expected = b.iter().sum::<f64>() * passes as f64;
    for _ in 0..3 {
        let t = Instant::now();
        let mut sum = 0.;
        if kind == "copy" {
            for _ in 0..passes {
                black_box(&mut a).copy_from_slice(black_box(&b));
            }
        } else {
            for _ in 0..passes {
                let a = black_box(&a);
                let mut random = 0;
                for i in 0..n {
                    let at = match kind {
                        "sequential" => i,
                        "stride17" => (i * 17) & (n - 1),
                        "dependent-lcg" => {
                            random = (random * 1664525usize + 1013904223) & (n - 1);
                            random
                        }
                        _ => unreachable!(),
                    };
                    sum += a[at];
                }
            }
            black_box(sum);
        }
        black_box(&a);
        windows.push(t.elapsed().as_nanos());
        if kind == "copy" {
            assert_eq!(a, b);
        } else {
            assert_eq!(sum, expected);
        }
    }
    json!({"kind":kind,"elements":n,"working_array_bytes":n*8,"passes_per_window":passes,"windows_ns":windows,"nominal_traffic_bytes":n*8*passes*if kind=="copy"{2}else{1},"verified":true,"limits":"nominal payload traffic, not measured DRAM traffic; address-generation/loop costs included"})
}
enum Task {
    Run(usize),
    Verify,
    Stop,
}
fn pool(workers: usize, n: usize) -> Value {
    assert!(n.is_multiple_of(workers));
    let mut commands = Vec::new();
    let (mut joins, mut replies) = (Vec::new(), Vec::new());
    let start = Instant::now();
    for w in 0..workers {
        let (tx, rx) = mpsc::channel();
        let (done, reply) = mpsc::channel();
        commands.push(tx);
        replies.push(reply);
        joins.push(thread::spawn(move || {
            let mut a = seed(n / workers, w * n / workers);
            let mut steps = 0;
            done.send(true).unwrap();
            while let Ok(task) = rx.recv() {
                match task {
                    Task::Run(p) => {
                        portable(&mut a, p);
                        steps += p;
                        done.send(true).unwrap();
                    }
                    Task::Verify => {
                        let mut reference = seed(n / workers, w * n / workers);
                        scalar(&mut reference, steps);
                        done.send(
                            a.iter()
                                .zip(reference)
                                .all(|(a, b)| a.to_bits() == b.to_bits()),
                        )
                        .unwrap();
                    }
                    Task::Stop => break,
                }
            }
        }));
    }
    for reply in &replies {
        assert!(reply.recv().unwrap());
    }
    let setup = start.elapsed().as_nanos();
    let passes = 16;
    let mut windows = Vec::new();
    for _ in 0..3 {
        let t = Instant::now();
        for tx in &commands {
            tx.send(Task::Run(passes)).unwrap();
        }
        for reply in &replies {
            assert!(reply.recv().unwrap());
        }
        windows.push(t.elapsed().as_nanos());
    }
    for tx in &commands {
        tx.send(Task::Verify).unwrap();
    }
    for reply in &replies {
        assert!(reply.recv().unwrap());
    }
    for tx in &commands {
        tx.send(Task::Stop).unwrap();
    }
    for j in joins {
        j.join().unwrap();
    }
    json!({"workers":workers,"elements_total":n,"passes_per_window":passes,"setup_ns":setup,"windows_ns":windows,"persistent_threads":true,"exact_reference":true,"scope":"independent disjoint arithmetic kernel with channel scheduling, not full atmospheric transactions"})
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() == 1 && args[0] == "--build-identity" {
        println!("{}", moebius_morph::BENCHMARK_BUILD_IDENTITY);
        return Ok(());
    }
    if args.is_empty() || args.len() > 2 || args.get(1).is_some_and(|s| s != "workers-only") {
        return Err("NEW_JSON [workers-only]".into());
    }
    let mut out = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[0])?;
    let workers_only = args.get(1).is_some_and(|s| s == "workers-only");
    let mut arithmetic_rows = Vec::new();
    let mut access_rows = Vec::new();
    let mut vector32 = Vec::new();
    #[cfg(target_arch = "x86_64")]
    let available = std::is_x86_feature_detected!("avx2");
    #[cfg(not(target_arch = "x86_64"))]
    let available = false;
    if !workers_only {
        for bytes in [32768, 262144, 4194304, 67108864] {
            let n = bytes / 8;
            arithmetic_rows.push(arithmetic(n, "scalar-black-box"));
            arithmetic_rows.push(arithmetic(n, "portable"));
            if available {
                arithmetic_rows.push(arithmetic(n, "avx2"));
                vector32.push(fp32(bytes / 4));
            }
            for kind in ["sequential", "stride17", "dependent-lcg", "copy"] {
                access_rows.push(access(n, kind));
            }
        }
    }
    let pool_rows = if workers_only {
        vec![pool(8, 4 * 1024 * 1024)]
    } else {
        [1, 2, 4, 8, 16, 32]
            .into_iter()
            .map(|w| pool(w, 4 * 1024 * 1024))
            .collect()
    };
    serde_json::to_writer_pretty(
        &mut out,
        &json!({"schema":1,"source_identity":serde_json::from_str::<Value>(moebius_morph::BENCHMARK_BUILD_IDENTITY)?,"max_estimated_array_residency_bytes":3*67108864,"avx2_execution_selected":available,"avx512":"NOT RUN; ISA inventory is not execution verification","arithmetic":arithmetic_rows,"fp32":vector32,"memory_access":access_rows,"worker_pool":pool_rows,"limits":"Synthetic kernel characterization only. Scalar black_box inhibits vectorization and adds barriers. No atmospheric throughput, cache bottleneck, totalRSS, DRAM peak or permanent tuning claim."}),
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_kernel_exact() {
        let mut a = seed(100, 0);
        let mut b = a.clone();
        portable(&mut a, 12);
        scalar(&mut b, 12);
        assert_eq!(a, b);
    }
    #[test]
    fn avx2_recurrence_exact() {
        #[cfg(target_arch = "x86_64")]
        if std::is_x86_feature_detected!("avx2") {
            let mut a = seed(103, 0);
            let mut b = a.clone();
            unsafe {
                avx2(&mut a, 17);
            }
            scalar(&mut b, 17);
            assert_eq!(a, b);
        }
    }
}
