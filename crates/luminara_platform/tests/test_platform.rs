use luminara_platform::{Time, FileSystem, PlatformInfo, Os};
use std::thread;
use std::time::Duration;

#[test]
fn test_time_delta() {
    let mut time = Time::new();
    assert_eq!(time.delta_seconds(), 0.0);
    assert_eq!(time.frame_count(), 0);

    // Sleep a bit to ensure non-zero delta
    thread::sleep(Duration::from_millis(50));
    time.update();

    assert!(time.delta_seconds() > 0.0, "Delta seconds should be > 0.0, got {}", time.delta_seconds());
    // On some CI/Sandbox, sleep might be imprecise, but > 0 should hold.
    assert!(time.elapsed_seconds() >= time.delta_seconds());
    assert_eq!(time.frame_count(), 1);

    // Test FPS
    assert!(time.fps() > 0.0);
}

#[test]
fn test_filesystem_paths() {
    let root = FileSystem::project_root();
    // In sandbox, we are at root usually.
    assert!(root.exists(), "Project root should exist");

    let assets = FileSystem::assets_dir();
    // Assert logic
    assert!(assets.to_string_lossy().len() > 0);
}

#[test]
fn test_file_io() {
    let temp = FileSystem::temp_dir();
    let test_file = temp.join("luminara_test.txt");

    let content = "Hello Luminara";
    FileSystem::write_string(&test_file, content).expect("Write failed");

    let read_content = FileSystem::read_string(&test_file).expect("Read failed");
    assert_eq!(content, read_content);

    // Bytes
    let bytes = b"Binary Data";
    FileSystem::write_bytes(&test_file, bytes).expect("Write bytes failed");
    let read_bytes = FileSystem::read_bytes(&test_file).expect("Read bytes failed");
    assert_eq!(bytes, &read_bytes[..]);
}

#[test]
fn test_platform_info() {
    let info = PlatformInfo::current();
    println!("Detected OS: {:?}", info.os);
    println!("Detected Arch: {:?}", info.arch);

    // In the sandbox environment, it is likely Linux x86_64
    // But we just assert it's not Unknown usually, unless it is truly unknown environment.
    assert_ne!(info.os, Os::Unknown);
    assert_ne!(info.arch, luminara_platform::Arch::Unknown);
}
