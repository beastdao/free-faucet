// to run from root directory cargo test --test db_test -- --nocapture

use db::*;
use dotenv::dotenv;
use std::env;
use tempfile::tempdir;

#[test]
fn test_claim_insert_and_retrieve() {
    dotenv().ok();
    let partition_size_limit: u64 = env::var("PARTITION_SIZE_LIMIT")
        .expect("should get env var for partition size limit")
        .parse()
        .expect("should parse u64");

    // Setup - create temp database
    let dir = tempdir().expect("should create temp dir");
    let db_path = dir.path().to_str().expect("valid path");
    let db = DB::new(db_path, partition_size_limit).expect("should create DB");

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
fn test_claim_inserts_and_last_claim() {
    dotenv().ok();
    let partition_size_limit: u64 = env::var("PARTITION_SIZE_LIMIT")
        .expect("should get env var for partition size limit")
        .parse()
        .expect("should parse u64");

    // Setup - create temp database
    let dir = tempdir().expect("should create temp dir");
    let db_path = dir.path().to_str().expect("valid path");
    let db = DB::new(db_path, partition_size_limit).expect("should create DB");

    // Test data
    let input = "beast@beast".to_string();
    let result = "test_result".to_string();
    let test_time_old = 1755965000u64; //oldest
    let test_time_mid = 1755965100u64;
    let test_time_last = 1755965200u64;

    db.insert_k_v_logs(test_time_old, true, input.clone(), result.clone())
        .expect("should insert log");
    db.insert_k_v_logs(test_time_mid, true, input.clone(), result.clone())
        .expect("should insert log");
    db.insert_k_v_logs(test_time_last, true, input.clone(), result.clone())
        .expect("should insert log");

    let last_claim_timestamp = db
        .get_last_claim_timestamp(0, 1755965900u64)
        .expect("should retrieve timestamp");

    dbg!(
        "last claimed timestamp:{}, retrieved last claim timestamp: {}",
        test_time_last,
        last_claim_timestamp
    );

    assert_eq!(
        last_claim_timestamp, test_time_last,
        "Retrieved value should match biggest timestamp inserted"
    );
}

#[test]
fn test_log_insert_and_retrieve() {
    dotenv().ok();
    let partition_size_limit: u64 = env::var("PARTITION_SIZE_LIMIT")
        .expect("should get env var for partition size limit")
        .parse()
        .expect("should parse u64");
    // Setup
    let dir = tempdir().expect("should create temp dir");
    let db_path = dir.path().to_str().expect("valid path");
    let db = DB::new(db_path, partition_size_limit).expect("should create DB");

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

#[test]
pub fn insert_many_k_v_test_fifo() {
    dotenv().ok();
    let partition_size_limit: u64 = env::var("PARTITION_SIZE_LIMIT")
        .expect("should get env var for partition size limit")
        .parse()
        .expect("should parse u64");
    let dir = tempdir().expect("./fjall_data_test");
    let db_path = dir.path().to_str().expect("valid path");
    let db = DB::new(db_path, partition_size_limit).expect("should create DB");
    for x in 0u64..1_000_000 {
        if x % 100_000 == 0 {
            let db_meta = db.get_db_meta().unwrap();

            println!(
                "Keyspace DISK SPACE : {} KB",
                db_meta.journal_disk_space / 1024
            );
            println!(
                "# logs entries  : {:?} - LOGS DISK SPACE : {} KB - # Segments: {}",
                db_meta.log_entries,
                db_meta.log_disk_space / 1024,
                db_meta.log_segments
            );

            println!(
                "# claim entries  : {:?} - Registry DISK SPACE : {} KB - # Segments: {}",
                db_meta.claim_entries,
                db_meta.claim_disk_space / 1024,
                db_meta.claim_segments
            );

            assert!(db_meta.log_disk_space < (db_meta.partition_size_limit as f64 * 1.5) as u64);
            assert!(db_meta.claim_disk_space < (db_meta.partition_size_limit as f64 * 1.5) as u64);
        }

        db.insert_k_v_logs(
            x,
            false,
            format!("user{}", x),
            "Unable to resolve the name: No name found".to_string(),
        )
        .unwrap();

        let base: i128 = 0x3983cD648de4d0509f56; //just 10 bytes of real address
        let claim_key = &format!("0x{:016x}{:020x}", base, x); //preparing address like 0x3983cd648de4d0509f560000000000000000a87a
        db.insert_k_v_claim(claim_key, 1234567890u64).unwrap();
    }
}
