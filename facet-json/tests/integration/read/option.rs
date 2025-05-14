use eyre::Result;
use facet::Facet;
use facet_json::from_str;

#[test]
fn test_from_json_with_option() -> Result<()> {
    facet_testhelpers::setup();

    #[derive(Facet)]
    struct Options {
        name: Option<String>,
        age: Option<u32>,
        inner: Option<Inner>,
    }

    #[derive(Facet)]
    struct Inner {
        foo: i32,
    }

    let json = r#"{
    "name": "Alice",
    "age": null,
    "inner": {
        "foo": 42
    }
}"#;

    let test_struct: Options = from_str(json)?;
    assert_eq!(test_struct.name.as_deref(), Some("Alice"));
    assert_eq!(test_struct.age, None);
    assert_eq!(test_struct.inner.as_ref().map(|i| i.foo), Some(42));

    Ok(())
}

#[test]
fn test_from_json_with_option_string_ptr() -> Result<()> {
    facet_testhelpers::setup();

    #[derive(Facet)]
    struct Options<'a> {
        name: Option<&'a str>,
        colour: Option<&'a str>,
        inner: Option<Inner<'a>>,
    }

    #[derive(Facet)]
    struct Inner<'a> {
        greeting: &'a str,
    }

    let json = r#"{
    "name": "Bob",
    "colour": null,
    "inner": {
        "greeting": "bonjour"
    }
}"#;

    let test_struct: Options = from_str(json)?;
    assert_eq!(test_struct.name.as_deref(), Some("Bob"));
    assert_eq!(test_struct.colour, None);
    assert_eq!(
        test_struct.inner.as_ref().map(|i| i.greeting),
        Some("bonjour")
    );

    Ok(())
}
