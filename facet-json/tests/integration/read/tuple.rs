use facet_json::from_str;

#[test]
fn test_deserialize_tuple_i32() {
    let result: Result<(i32,), _> = from_str(r#"[10]"#);
    let ok = result.unwrap();
    assert_eq!(ok.0, 10);

    let result: Result<(i32, i32), _> = from_str(r#"[10,20]"#);
    let ok = result.unwrap();
    assert_eq!(ok.0, 10);
    assert_eq!(ok.1, 20);

    let result: Result<(i32, i32, i32), _> = from_str(r#"[10,20,30]"#);
    let ok = result.unwrap();
    assert_eq!(ok.0, 10);
    assert_eq!(ok.1, 0);
    assert_eq!(ok.2, 30);

    let result: Result<(i32, i32, i32, i32), _> = from_str(r#"[10,20,30,40]"#);
    let ok = result.unwrap();
    assert_eq!(ok.0, 10);
    assert_eq!(ok.1, 0);
    assert_eq!(ok.2, 0);
    assert_eq!(ok.3, 40);

    let result: Result<(i32, i32, i32, i32, i32), _> = from_str(r#"[10,20,30,40,50]"#);
    let ok = result.unwrap();
    assert_eq!(ok.0, 10);
    assert_eq!(ok.1, 0);
    assert_eq!(ok.2, 0);
    assert_eq!(ok.3, 0);
    assert_eq!(ok.4, 50);
}

#[test]
#[ignore]
fn test_deserialize_tuple_mixed() {
    let result: Result<(&str, i32), _> = from_str(r#"["aaa",3]"#);
    let ok = result.unwrap();
    assert_eq!(ok.0, "aaa");
    assert_eq!(ok.1, 3);

    #[derive(facet::Facet)]
    struct TestTuple(i32, String, bool);
    let result: Result<TestTuple, _> = from_str(r#"[3,"aaa",true]"#);
    let ok = result.unwrap();
    assert_eq!(ok.0, 3);
    assert_eq!(ok.1, "aaa");
    assert!(ok.2);
}

#[test]
fn test_deserialize_list() {
    let result: Result<Vec<i32>, _> = from_str(r#"[1,3]"#);
    let ok = result.unwrap();
    assert_eq!(ok[0], 1);
    assert_eq!(ok[1], 3);
}
