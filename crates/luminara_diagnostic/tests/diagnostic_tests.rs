use luminara_diagnostic::{init_logging, FrameStats, Diagnostics, ProfileScope};
use instant::Duration;

#[test]
fn test_profile_scope_calculations() {
    let mut scope = ProfileScope::new("test".to_string(), 3);
    scope.record(Duration::from_millis(10));
    scope.record(Duration::from_millis(20));
    scope.record(Duration::from_millis(30));

    assert_eq!(scope.average(), Duration::from_millis(20));
    assert_eq!(scope.max(), Duration::from_millis(30));
    assert_eq!(scope.min(), Duration::from_millis(10));

    // Test overflow
    scope.record(Duration::from_millis(40));
    assert_eq!(scope.samples.len(), 3);
    assert_eq!(scope.average(), Duration::from_millis(30)); // (20+30+40)/3 = 30
}

#[test]
fn test_diagnostics_history() {
    let mut diagnostics = Diagnostics::new();
    diagnostics.add("cpu_usage", 10.0);
    diagnostics.add("cpu_usage", 20.0);

    let entry = diagnostics.get("cpu_usage").unwrap();
    assert_eq!(entry.value, 20.0);
    assert_eq!(entry.history.len(), 2);
    assert_eq!(diagnostics.get_average("cpu_usage"), Some(15.0));
}

#[test]
fn test_frame_stats_percentiles() {
    let mut stats = FrameStats::default();
    for i in 1..=100 {
        stats.frame_time_history.push_back(i as f32);
    }

    assert_eq!(stats.percentile_frame_time(0.0), 1.0);
    // index = round(0.5 * 99) = 50. index 50 is value 51.
    assert_eq!(stats.percentile_frame_time(50.0), 51.0);
    assert_eq!(stats.percentile_frame_time(100.0), 100.0);

    // p99
    // index = round(0.99 * 99) = 98. index 98 is value 99.
    assert_eq!(stats.percentile_frame_time(99.0), 99.0);
}

#[test]
fn test_logging_init_safety() {
    init_logging();
    init_logging(); // Should not panic
}
