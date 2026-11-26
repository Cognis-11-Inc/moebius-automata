#![cfg(feature = "history-v2-prototype")]
use moebius_morph::{
    history::{Policy, Store},
    runtime::*,
};
use std::path::PathBuf;
fn root(label: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "moebius-history-{label}-{}-{}",
        std::process::id(),
        N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    assert!(!path.exists());
    path
}
fn policy() -> Policy {
    Policy {
        resident_events: 300,
        segment_events: 44,
        ..Policy::default()
    }
}
fn command(tick: u64, sequence: u64) -> Intervention {
    Intervention {
        tick,
        sequence,
        origin: "history-proof".into(),
        action: Operation::SetAmbientLight {
            value: sequence % 100,
        },
    }
}
fn all(runtime: &ShipRuntime) -> Vec<Event> {
    let s = runtime.state();
    let mut events = Vec::new();
    runtime
        .history_store()
        .unwrap()
        .visit(s.history.as_ref().unwrap(), |e| {
            events.push(e.clone());
            Ok(())
        })
        .unwrap();
    events.extend_from_slice(&s.events);
    events
}
#[test]
fn exact_stream_migration_portable_restore_and_continuation() {
    let mut legacy = fixture(2, 9).unwrap();
    let initial = legacy.snapshot();
    let origin_digest = legacy.digest().unwrap();
    let backing = root("stream");
    let mut segmented = ShipRuntime::restore(initial.clone()).unwrap();
    segmented.enable_history_v2(&backing, policy()).unwrap();
    assert_ne!(origin_digest, segmented.digest().unwrap());
    assert_eq!(
        WorldSnapshot::load(&backing.join("LEGACY-ORIGIN.json")).unwrap(),
        initial
    );
    for tick in 0..512 {
        let c = command(tick, tick + 1);
        legacy.submit(c.clone()).unwrap();
        segmented.submit(c).unwrap();
        legacy.tick().unwrap();
        segmented.tick().unwrap();
        assert!(segmented.state().events.len() <= 300);
        assert_eq!(all(&segmented), legacy.state().events);
        let mut physical = segmented.snapshot();
        physical.events = legacy.state().events.clone();
        physical.history = None;
        assert_eq!(physical, legacy.snapshot());
    }
    segmented.validate().unwrap();
    let before = segmented.digest().unwrap();
    assert!(segmented.submit(command(600, 1)).is_err());
    assert_eq!(before, segmented.digest().unwrap());
    assert!(ShipRuntime::restore(segmented.snapshot()).is_err());
    let portable = root("portable");
    segmented
        .history_store()
        .unwrap()
        .copy_prefix(segmented.state().history.as_ref().unwrap(), &portable)
        .unwrap();
    let checkpoint = portable.join("checkpoint.json");
    segmented.save_checkpoint_v2(&checkpoint).unwrap();
    let snapshot: WorldSnapshot =
        serde_json::from_slice(&std::fs::read(&checkpoint).unwrap()).unwrap();
    let mut restored = ShipRuntime::restore_with_history(snapshot, &portable).unwrap();
    assert_eq!(restored.snapshot(), segmented.snapshot());
    for _ in 0..8 {
        segmented.tick().unwrap();
        restored.tick().unwrap();
        assert_eq!(restored.snapshot(), segmented.snapshot());
    }
    let segment = portable.join("segments/00000000000000000000.json");
    let good = std::fs::read(&segment).unwrap();
    let second = portable.join("segments/00000000000000000001.json");
    let second_good = std::fs::read(&second).unwrap();
    std::fs::write(&segment, &second_good).unwrap();
    std::fs::write(&second, &good).unwrap();
    assert!(ShipRuntime::restore_with_history(restored.snapshot(), &portable).is_err());
    std::fs::write(&segment, &good).unwrap();
    std::fs::write(&second, &second_good).unwrap();
    std::fs::write(&segment, b"corrupt").unwrap();
    assert!(ShipRuntime::restore_with_history(restored.snapshot(), &portable).is_err());
    std::fs::write(&segment, good).unwrap();
    std::fs::rename(&segment, portable.join("missing-segment")).unwrap();
    assert!(ShipRuntime::restore_with_history(restored.snapshot(), &portable).is_err());
    assert!(Store::open(&portable).is_ok()); // backing header alone is not restoration proof
}
#[test]
fn failed_seal_is_atomic_and_retry_keeps_pending_command() {
    let backing = root("failure");
    let mut w = fixture(1, 7).unwrap();
    w.enable_history_v2(&backing, policy()).unwrap();
    for sequence in 1..=299 {
        w.submit(command(10000, sequence)).unwrap();
    }
    w.submit(command(0, 300)).unwrap();
    let before = w.snapshot();
    std::fs::rename(backing.join("segments"), backing.join("offline-segments")).unwrap();
    assert!(w.tick().is_err());
    assert_eq!(before, w.snapshot());
    std::fs::rename(backing.join("offline-segments"), backing.join("segments")).unwrap();
    w.tick().unwrap();
    w.validate().unwrap();
    assert_eq!(w.state().tick, 1);
    assert_eq!(w.state().pending.len(), 299);
}
#[test]
fn explicit_provenance_limit_refuses_admission_atomically() {
    let backing = root("budget");
    let mut w = fixture(1, 7).unwrap();
    let mut p = policy();
    p.provenance_entries = 16;
    w.enable_history_v2(&backing, p).unwrap();
    for sequence in 1..=16 {
        w.submit(command(0, sequence)).unwrap();
    }
    let before = w.snapshot();
    assert!(w.submit(command(0, 17)).is_err());
    assert_eq!(before, w.snapshot());
    w.tick().unwrap();
    w.validate().unwrap();
}

// Executed in a separate process with a real OS file-size limit. The ignored
// SIGXFSZ is inherited through exec; writes return EFBIG instead of terminating.
#[test]
fn limited_file_child() {
    let Ok(root) = std::env::var("MOEBIUS_HISTORY_LIMITED_CHILD") else {
        return;
    };
    let backing = PathBuf::from(root);
    let snapshot: WorldSnapshot =
        serde_json::from_slice(&std::fs::read(backing.join("before.json")).unwrap()).unwrap();
    let mut w = ShipRuntime::restore_with_history(snapshot, &backing).unwrap();
    let before = w.snapshot();
    let error = w.tick().unwrap_err();
    assert!(error.0.contains("File too large"), "{error}");
    assert_eq!(before, w.snapshot());
}
#[test]
#[cfg(target_os = "linux")]
fn os_write_exhaustion_and_unpublished_segment_recovery() {
    let backing = root("exhaustion");
    let mut w = fixture(1, 7).unwrap();
    w.enable_history_v2(&backing, policy()).unwrap();
    for sequence in 1..=299 {
        w.submit(command(10000, sequence)).unwrap();
    }
    w.submit(command(0, 300)).unwrap();
    let before = w.snapshot();
    std::fs::write(
        backing.join("before.json"),
        serde_json::to_vec(&before).unwrap(),
    )
    .unwrap();
    let output=std::process::Command::new("python3")
        .args(["-c","import os,resource,signal,sys; signal.signal(signal.SIGXFSZ,signal.SIG_IGN); resource.setrlimit(resource.RLIMIT_FSIZE,(1024,1024)); os.execv(sys.argv[1],[sys.argv[1],'--exact','limited_file_child','--nocapture'])"])
        .arg(std::env::current_exe().unwrap()).env("MOEBIUS_HISTORY_LIMITED_CHILD",&backing).output().unwrap();
    // Pipes keep libtest stdout/stderr outside the file-size fault domain.
    assert!(
        output.status.success(),
        "child {:?}: {} {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    assert_eq!(before, w.snapshot());
    // Committed segment bytes exist, while the only published checkpoint is old.
    // Restoring that checkpoint ignores orphans, and exact replay reuses them.
    w.tick().unwrap();
    let expected = w.snapshot();
    let mut recovered = ShipRuntime::restore_with_history(before, &backing).unwrap();
    recovered.tick().unwrap();
    assert_eq!(expected, recovered.snapshot());
    recovered.validate().unwrap();
    assert!(
        std::fs::read_dir(backing.join("segments"))
            .unwrap()
            .any(|e| e
                .unwrap()
                .path()
                .extension()
                .is_some_and(|s| s.to_string_lossy().starts_with("tmp-"))),
        "failed partial write evidence was lost"
    );
}
