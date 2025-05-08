// to run from root directory cargo test --test db_test -- --nocapture

use db::*;
use tempfile::tempdir;

#[test]
fn test_claim_insert_and_retrieve() {
    // Setup - create temp database
    let dir = tempdir().expect("should create temp dir");
    let db_path = dir.path().to_str().expect("valid path");
    let db = DB::new(db_path, "claims").expect("should create DB");

    // Test data
    let test_key = "0xf0E5D3Cc05206987a125afC404b719e54Fa942a8";
    let test_value = 1234567890u64;

    db.insert_k_v_claim(test_key, test_value)
        .expect("should insert claim");

    let retrieved = db
        .get_value_claim(test_key)
        .expect("should retrieve claim")
        .expect("value should exist");
    dbg!(
        "test claim vlaue:{}, retrieved value: {}",
        test_value,
        retrieved
    );

    assert_eq!(
        retrieved, test_value,
        "Retrieved value should match inserted"
    );
}

#[test]
fn test_log_insert_and_retrieve() {
    // Setup
    let dir = tempdir().expect("should create temp dir");
    let db_path = dir.path().to_str().expect("valid path");
    let db = DB::new(db_path, "claims").expect("should create DB");

    // Test data
    let timestamp = 1234567890u64;
    let input = "beast@beast".to_string();
    let result = "test_result".to_string();

    db.insert_k_v_logs(timestamp, true, input.clone(), result.clone())
        .expect("should insert log");

    let retrieved = db
        .get_value_log((timestamp, true as u8))
        .expect("should retrieve log")
        .expect("log should exist");

    assert_eq!(retrieved.input, input);
    assert_eq!(retrieved.result, result);
}
