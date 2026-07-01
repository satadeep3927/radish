use bytes::Bytes;
use radish_storage::{Keyspace, Value};

#[test]
fn test_snapshot_isolation() {
    let mut ks = Keyspace::new();
    
    let key_a = Bytes::from("A");
    let key_b = Bytes::from("B");
    
    // Insert initial data
    ks.set(key_a.clone(), Value::String(Bytes::from("1")));
    ks.set(key_b.clone(), Value::String(Bytes::from("2")));
    
    // Take an instant snapshot
    let snapshot = ks.snapshot();
    
    // Modify the main keyspace
    ks.set(key_a.clone(), Value::String(Bytes::from("1-modified")));
    ks.del(b"B"); // Delete B
    
    // Verify the main keyspace reflects modifications
    assert_eq!(ks.get(b"A"), Some(&Value::String(Bytes::from("1-modified"))));
    assert_eq!(ks.get(b"B"), None);
    
    // Verify the snapshot remains perfectly untouched (structural sharing)
    assert_eq!(snapshot.get(b"A"), Some(&Value::String(Bytes::from("1"))));
    assert_eq!(snapshot.get(b"B"), Some(&Value::String(Bytes::from("2"))));
}
