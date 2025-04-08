use super::*;

/// Processes a regular struct to implement Facet
///
/// Example input:
/// ```rust
/// struct Blah {
///     foo: u32,
///     bar: String,
/// }
/// ```
pub(crate) fn process_struct(parsed: Struct) -> proc_macro::TokenStream {
    let struct_name = parsed.name.to_string();

    // Generate dummy fields
    let dummy_fields = parsed
        .body
        .content
        .0
        .iter()
        .map(|field| field.value.name.to_string())
        .map(|field| format!("{field}: Facet::DUMMY"))
        .collect::<Vec<String>>()
        .join(", ");

    // Generate field definitions
    let field_definitions = parsed
        .body
        .content
        .0
        .iter()
        .map(|field| {
            let field_name = field.value.name.to_string();

            // Determine field flags
            let mut flags = "facet::FieldFlags::EMPTY";
            for attr in &field.value.attributes {
                if let AttributeInner::Facet(_) = &attr.body.content {
                    // Since FacetInner only has Sensitive variant, we can directly set flags
                    flags = "facet::FieldFlags::SENSITIVE";
                }
            }

            // Generate each field definition
            format!(
                "facet::Field::builder()
                .name(\"{field_name}\")
                .shape(facet::shape_of(&|s: {struct_name}| s.{field_name}))
                .offset(::std::mem::offset_of!({struct_name}, {field_name}))
                .flags({flags})
                .build()"
            )
        })
        .collect::<Vec<String>>()
        .join(
            ",
            ",
        );

    // Generate the impl
    let output = format!(
        r#"
#[automatically_derived]
unsafe impl facet::Facet for {struct_name} {{
    const DUMMY: Self = Self {{
        {dummy_fields}
    }};

    const SHAPE: &'static facet::Shape = &const {{
        static FIELDS: &[facet::Field] = &[
            {field_definitions}
        ];

        facet::Shape::builder()
            .layout(std::alloc::Layout::new::<Self>())
            .vtable(facet::value_vtable!(
                {struct_name},
                |f, _opts| std::fmt::Write::write_str(f, "{struct_name}")
            ))
            .def(facet::Def::Struct(facet::StructDef::builder()
                .kind(facet::StructKind::Struct)
                .fields(FIELDS)
                .build()))
            .build()
    }};
}}
        "#
    );
    output.into_token_stream().into()
}
