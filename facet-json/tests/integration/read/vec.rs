use eyre::Result;
use facet::Facet;
use facet_json::from_str;

#[test]
fn json_read_empty_vec() -> Result<()> {
    facet_testhelpers::setup();

    let json = r#"[]"#;

    let v: Vec<i32> = from_str(json)?;
    assert_eq!(v, vec![]);

    Ok(())
}

#[test]
fn json_read_vec() -> Result<()> {
    facet_testhelpers::setup();

    let json = r#"[1, 2, 3, 4, 5]"#;

    let v: Vec<u64> = from_str(json)?;
    assert_eq!(v, vec![1, 2, 3, 4, 5]);

    Ok(())
}

#[test]
fn json_read_vec_string_ptr() -> Result<()> {
    facet_testhelpers::setup();

    let json = r#"["aa", "bb", "cc"]"#;

    let v: Vec<&str> = from_str(json)?;
    assert_eq!(v, vec!["aa", "bb", "cc"]);

    Ok(())
}

#[test]
fn test_two_empty_vecs() -> Result<()> {
    facet_testhelpers::setup();

    #[derive(Facet, Clone, Default)]
    pub struct RevisionConfig {
        pub one: Vec<String>,
        pub two: Vec<String>,
    }

    let markup = r#"
    {
      "one": [],
      "two": []
    }
    "#;

    let config: RevisionConfig = from_str(markup)?;
    assert!(config.one.is_empty());
    assert!(config.two.is_empty());

    Ok(())
}

#[test]
fn test_one_empty_one_nonempty_vec() -> Result<()> {
    facet_testhelpers::setup();

    #[derive(Facet, Clone, Default)]
    pub struct RevisionConfig {
        pub one: Vec<String>,
        pub two: Vec<String>,
    }

    let markup = r#"
    {
      "one": [],
      "two": ["a", "b", "c"]
    }
    "#;

    let config: RevisionConfig = from_str(markup)?;
    assert!(config.one.is_empty());
    assert_eq!(config.two, vec!["a", "b", "c"]);

    Ok(())
}

#[test]
fn test_one_nonempty_one_empty_vec() -> Result<()> {
    facet_testhelpers::setup();

    #[derive(Facet, Clone, Default)]
    pub struct RevisionConfig {
        pub one: Vec<String>,
        pub two: Vec<String>,
    }

    let markup = r#"
    {
      "one": ["x", "y"],
      "two": []
    }
    "#;

    let config: RevisionConfig = from_str(markup)?;
    assert_eq!(config.one, vec!["x", "y"]);
    assert!(config.two.is_empty());

    Ok(())
}

#[test]
fn test_nested_arrays() -> Result<()> {
    facet_testhelpers::setup();

    #[derive(Facet, Clone, Default)]
    pub struct NestedArrays {
        pub matrix: Vec<Vec<u64>>,
    }

    let markup = r#"
    {
      "matrix": [
        [1, 2, 3],
        [],
        [4, 5]
      ]
    }
    "#;

    let nested: NestedArrays = from_str(markup)?;
    assert_eq!(nested.matrix.len(), 3);
    assert_eq!(nested.matrix[0], vec![1, 2, 3]);
    assert_eq!(nested.matrix[1], vec![]);
    assert_eq!(nested.matrix[2], vec![4, 5]);

    Ok(())
}

#[test]
fn test_deserialize_list() -> Result<()> {
    let result: Vec<i32> = from_str(r#"[1,3]"#)?;
    assert_eq!(result[0], 1);
    assert_eq!(result[1], 3);

    Ok(())
}
