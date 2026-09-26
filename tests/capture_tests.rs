use rgrim::capture::{capture_primary_monitor, capture_settled_monitor};
use std::path::Path;

#[test]
#[ignore]
fn test_screen_capture_engine() {
    let result = capture_primary_monitor();

    assert!(
        result.is_ok(),
        "Capture failed! Check OS screen recording permissions: {:?}",
        result.err()
    );

    let captured = result.unwrap();

    assert!(
        captured.image.width() > 0,
        "Captured width must be greater than 0"
    );
    assert!(
        captured.image.height() > 0,
        "Captured height must be greater than 0"
    );
    assert!(
        !captured.name.is_empty(),
        "Monitor name should not be blank"
    );

    let output_path = Path::new("target_capture.png");
    let save_result = captured.image.save(output_path);

    assert!(
        save_result.is_ok(),
        "Failed to save the target_capture.png file to disk"
    );
    println!(
        "Successfully captured screen '{}' ({})",
        captured.name,
        output_path.display()
    );
}

#[test]
#[ignore]
fn test_settled_capture_matches_direct_capture_geometry() {
    let settled = capture_settled_monitor()
        .expect("Settled capture failed! Check OS screen recording permissions");
    let direct = capture_primary_monitor().expect("Direct capture failed");

    assert_eq!(
        settled.image.dimensions(),
        direct.image.dimensions(),
        "Settled capture must cover the same monitor geometry"
    );
}
