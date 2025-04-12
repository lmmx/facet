use facet::Facet;
use facet_reflect::{Poke, PokeUninit};

use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Facet)]
struct Person {
    age: u64,
    name: String,
}

impl Default for Person {
    fn default() -> Self {
        Person {
            age: 69,
            name: String::new(),
        }
    }
}

#[test]
fn build_person_through_reflection() {
    facet_testhelpers::setup();

    let (poke, guard) = PokeUninit::alloc::<Person>();
    let mut poke = poke.into_struct();
    poke.set_by_name("age", 42_u64).unwrap();
    poke.set_by_name("name", String::from("Joan Watson"))
        .unwrap();
    let person = poke.build::<Person>(Some(guard));

    assert_eq!(
        Person {
            age: 42,
            name: "Joan Watson".to_string()
        },
        person
    )
}

#[test]
fn set_by_name_no_such_field() {
    facet_testhelpers::setup();

    let (poke, _guard) = PokeUninit::alloc::<Person>();
    let mut poke = poke.into_struct();
    assert_eq!(
        poke.set_by_name("philosophy", 42u16),
        Err(facet_core::FieldError::NoSuchField)
    );
}

#[test]
fn set_by_name_type_mismatch() {
    facet_testhelpers::setup();

    let (poke, _guard) = PokeUninit::alloc::<Person>();
    let mut poke = poke.into_struct();
    assert_eq!(
        // note: age is a u64, not a u16
        poke.set_by_name("age", 42u16),
        Err(facet_core::FieldError::TypeMismatch {
            expected: u64::SHAPE,
            actual: u16::SHAPE,
        })
    );
}

#[test]
#[should_panic(expected = "Field 'name' was not initialized")]
fn build_person_incomplete() {
    facet_testhelpers::setup();

    let (poke, guard) = PokeUninit::alloc::<Person>();
    let mut poke = poke.into_struct();
    poke.set_by_name("age", 42u64).unwrap();

    // we haven't set name, this'll panic
    poke.build::<Person>(Some(guard));
}

#[test]
fn mutate_person() {
    facet_testhelpers::setup();

    let mut person: Person = Default::default();

    {
        let mut poke = Poke::new(&mut person).into_struct();
        // Use the safe set_by_name method
        poke.set_by_name("name", String::from("Hello, World!"))
            .unwrap();
        poke.build_in_place();
    }

    // Verify the fields were set correctly
    assert_eq!(person.age, 69);
    assert_eq!(person.name, "Hello, World!");
}
