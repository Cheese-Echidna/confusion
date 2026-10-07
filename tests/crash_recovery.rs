use confusion::{
    application::session::RecoverySchedule,
    document::schema::{Design, Parameter},
    persistence::recovery::RecoveryManager,
};
use std::time::{Duration, Instant};
use uuid::Uuid;
fn design(value: &str) -> Design {
    let mut d = Design::default();
    d.parameters.push(Parameter {
        id: Uuid::new_v4(),
        name: "length".into(),
        expression: value.into(),
        scalar: false,
        angular: false,
    });
    d
}
fn wait(manager: &mut RecoveryManager) -> Result<(), String> {
    let start = Instant::now();
    loop {
        if let Some(result) = manager.poll() {
            return result;
        }
        assert!(start.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn current_intent_recovers_and_live_sessions_are_not_candidates() {
    let root = tempfile::tempdir().unwrap();
    let mut a = RecoveryManager::new(root.path()).unwrap();
    let id = Uuid::new_v4();
    assert!(a.snapshot(vec![(id, design("10 mm"))]));
    wait(&mut a).unwrap();
    assert!(
        RecoveryManager::new(root.path())
            .unwrap()
            .candidates
            .is_empty()
    );
    assert!(a.snapshot(vec![(id, design("20 mm"))]));
    wait(&mut a).unwrap();
    drop(a);

    let mut next = RecoveryManager::new(root.path()).unwrap();
    assert_eq!(next.candidates.len(), 1);
    let (recovered_id, restored) = next.recover(0).unwrap();
    assert_eq!(restored.parameters[0].expression, "20 mm");
    assert!(next.candidates.is_empty());
    next.forget(recovered_id).unwrap();
    next.flush().unwrap();
    drop(next);

    assert!(
        RecoveryManager::new(root.path())
            .unwrap()
            .candidates
            .is_empty()
    );
}
#[test]
fn queued_cleanup_is_per_tab_and_corruption_is_independent() {
    let root = tempfile::tempdir().unwrap();
    let mut manager = RecoveryManager::new(root.path()).unwrap();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    assert!(manager.snapshot(vec![(a, design("1 mm")), (b, design("2 mm"))]));
    manager.forget(a).unwrap();
    manager.flush().unwrap();
    wait(&mut manager).unwrap();
    drop(manager);

    let dir = std::fs::read_dir(root.path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(dir.join("corrupt.con"), b"broken archive").unwrap();
    let mut next = RecoveryManager::new(root.path()).unwrap();
    assert_eq!(next.candidates.len(), 2);
    let damaged = next
        .candidates
        .iter()
        .position(|c| c.error.is_some())
        .unwrap();
    assert!(next.recover(damaged).is_err());
    next.discard(damaged).unwrap();
    let (_, restored) = next.recover(0).unwrap();
    assert_eq!(restored.parameters[0].expression, "2 mm");
}
#[test]
fn debounce_write_bound_and_continuous_edit_checkpoint() {
    let mut schedule = RecoverySchedule::default();
    let now = Instant::now();
    let a = blake3::hash(b"a");
    let b = blake3::hash(b"b");
    assert!(!schedule.observe(a, now));
    assert!(schedule.observe(a, now + Duration::from_secs(2)));
    schedule.submitted(now + Duration::from_secs(2));
    assert!(!schedule.observe(a, now + Duration::from_secs(20)));
    assert!(!schedule.observe(b, now + Duration::from_secs(3)));
    assert!(!schedule.observe(b, now + Duration::from_secs(5)));
    assert!(schedule.observe(b, now + Duration::from_secs(12)));
    schedule.submitted(now + Duration::from_secs(12));
    for n in 13u64..43 {
        assert!(!schedule.observe(blake3::hash(&n.to_le_bytes()), now + Duration::from_secs(n)));
    }
    assert!(schedule.observe(a, now + Duration::from_secs(43)));
    schedule.submitted(now + Duration::from_secs(43));
    schedule.failed();
    assert!(schedule.observe(a, now + Duration::from_secs(53)));
}
#[test]
fn failed_snapshot_reports_path_without_destroying_previous_snapshot() {
    let root = tempfile::tempdir().unwrap();
    let mut manager = RecoveryManager::new(root.path()).unwrap();
    let id = Uuid::new_v4();
    assert!(manager.snapshot(vec![(id, design("5 mm"))]));
    wait(&mut manager).unwrap();
    let mut invalid = design("6 mm");
    invalid.parameters[0].expression = "x".repeat(5 * 1024 * 1024);
    assert!(manager.snapshot(vec![(id, invalid)]));
    assert!(
        wait(&mut manager)
            .unwrap_err()
            .contains("Autosave failed at")
    );
    drop(manager);

    let mut next = RecoveryManager::new(root.path()).unwrap();
    assert_eq!(next.recover(0).unwrap().1.parameters[0].expression, "5 mm");
}
