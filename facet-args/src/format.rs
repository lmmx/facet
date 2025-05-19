use alloc::borrow::Cow;
use alloc::string::ToString;
use core::fmt;
use facet_core::{Facet, FieldAttribute, Type, UserType};
use facet_deserialize::{
    DeserError, DeserErrorKind, Expectation, Format, NextData, NextResult, Outcome, Raw, Scalar,
    Span, Spanned,
};

/// Command-line argument format for Facet deserialization
pub struct Cli;

impl fmt::Display for Cli {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cli")
    }
}

impl Cli {
    /// Helper function to convert kebab-case to snake_case
    fn kebab_to_snake(input: &str) -> Cow<str> {
        if !input.contains('-') {
            return Cow::Borrowed(input);
        }
        Cow::Owned(input.replace('-', "_"))
    }
}

/// Parse command line arguments into a Facet-compatible type
pub fn from_slice<'input, 'facet, 'shape, T: Facet<'facet>>(
    args: &'input [&'input str],
) -> Result<T, DeserError<'input, 'shape>>
where
    'input: 'facet + 'shape,
{
    facet_deserialize::deserialize(args, Cli)
}

impl Format for Cli {
    type Input<'input> = [&'input str];
    type SpanType = Raw;

    fn source(&self) -> &'static str {
        "args"
    }

    fn next<'input, 'facet, 'shape>(
        &mut self,
        nd: NextData<'input, 'facet, 'shape, Self::SpanType, Self::Input<'input>>,
        expectation: Expectation,
    ) -> NextResult<
        'input,
        'facet,
        'shape,
        Spanned<Outcome<'input>, Self::SpanType>,
        Spanned<DeserErrorKind<'shape>, Self::SpanType>,
        Self::SpanType,
        Self::Input<'input>,
    >
    where
        'shape: 'input,
    {
        let arg_idx = nd.start();
        let shape = nd.wip.shape();
        let args = nd.input();

        match expectation {
            // Top-level value
            Expectation::Value => {
                // Check if it's a struct type
                if !matches!(shape.ty, Type::User(UserType::Struct(_))) {
                    return (
                        nd,
                        Err(Spanned {
                            node: DeserErrorKind::UnsupportedType {
                                got: shape,
                                wanted: "struct",
                            },
                            span: Span::new(arg_idx, 0),
                        }),
                    );
                }
                // For CLI args, we always start with an object (struct)
                (
                    nd,
                    Ok(Spanned {
                        node: Outcome::ObjectStarted,
                        span: Span::new(arg_idx, 0),
                    }),
                )
            }

            // Object key (or finished)
            Expectation::ObjectKeyOrObjectClose => {
                /* Check if we have more arguments */
                if arg_idx < args.len() {
                    let arg = args[arg_idx];
                    let span = Span::new(arg_idx, 1);

                    // Named long argument?
                    if let Some(key) = arg.strip_prefix("--") {
                        let key = Self::kebab_to_snake(key);

                        // Check if the field exists in the struct
                        if let Type::User(UserType::Struct(_)) = shape.ty {
                            if nd.wip.field_index(&key).is_none() {
                                return (
                                    nd,
                                    Err(Spanned {
                                        node: DeserErrorKind::UnknownField {
                                            field_name: key.to_string(),
                                            shape,
                                        },
                                        span: Span::new(arg_idx, 0),
                                    }),
                                );
                            }
                        }
                        return (
                            nd,
                            Ok(Spanned {
                                node: Outcome::Scalar(Scalar::String(key)),
                                span,
                            }),
                        );
                    }

                    // Short flag?
                    if let Some(key) = arg.strip_prefix('-') {
                        // Convert short argument to field name via shape
                        if let Type::User(UserType::Struct(st)) = shape.ty {
                            for field in st.fields.iter() {
                                for attr in field.attributes {
                                    if let FieldAttribute::Arbitrary(a) = attr {
                                        // Don't require specifying a short key for a single-char key
                                        if a.contains("short")
                                            && (a.contains(key)
                                                || (key.len() == 1 && field.name == key))
                                        {
                                            return (
                                                nd,
                                                Ok(Spanned {
                                                    node: Outcome::Scalar(Scalar::String(
                                                        Cow::Borrowed(field.name),
                                                    )),
                                                    span,
                                                }),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        return (
                            nd,
                            Err(Spanned {
                                node: DeserErrorKind::UnknownField {
                                    field_name: key.to_string(),
                                    shape,
                                },
                                span: Span::new(arg_idx, 0),
                            }),
                        );
                    }

                    // positional argument
                    if let Type::User(UserType::Struct(st)) = &shape.ty {
                        for (idx, field) in st.fields.iter().enumerate() {
                            for attr in field.attributes.iter() {
                                if let FieldAttribute::Arbitrary(a) = attr {
                                    if a.contains("positional") {
                                        // Check if this field is already set
                                        let is_set = nd.wip.is_field_set(idx).unwrap_or(false);

                                        if !is_set {
                                            // Use this positional field
                                            return (
                                                nd,
                                                Ok(Spanned {
                                                    node: Outcome::Scalar(Scalar::String(
                                                        Cow::Borrowed(field.name),
                                                    )),
                                                    span: Span::new(arg_idx, 0),
                                                }),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // If no positional field was found
                    return (
                        nd,
                        Err(Spanned {
                            node: DeserErrorKind::UnknownField {
                                field_name: "positional argument".to_string(),
                                shape,
                            },
                            span: Span::new(arg_idx, 0),
                        }),
                    );
                }

                // EOF: inject implicit-false-if-absent bool flags, if there are any
                if let Type::User(UserType::Struct(st)) = &shape.ty {
                    for (idx, field) in st.fields.iter().enumerate() {
                        if !nd.wip.is_field_set(idx).unwrap_or(false)
                            && field.shape().is_type::<bool>()
                        {
                            return (
                                nd,
                                Ok(Spanned {
                                    node: Outcome::Scalar(Scalar::String(Cow::Borrowed(
                                        field.name,
                                    ))),
                                    span: Span::new(arg_idx, 0),
                                }),
                            );
                        }
                    }
                }

                // Real end of object
                (
                    nd,
                    Ok(Spanned {
                        node: Outcome::ObjectEnded,
                        span: Span::new(arg_idx, 0),
                    }),
                )
            }

            // Value for the current key
            Expectation::ObjectVal => {
                // Synthetic implicit-false
                if arg_idx >= args.len() && shape.is_type::<bool>() {
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::Scalar(Scalar::Bool(false)),
                            span: Span::new(arg_idx, 0),
                        }),
                    );
                }

                // Explicit boolean true
                if shape.is_type::<bool>() {
                    // For boolean fields, we don't need an explicit value
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::Scalar(Scalar::Bool(true)),
                            span: Span::new(arg_idx, 0),
                        }),
                    );
                }

                // For other types, get the next arg as the value.
                // Need another CLI token:
                if arg_idx >= args.len() {
                    return (
                        nd,
                        Err(Spanned {
                            node: DeserErrorKind::MissingValue {
                                expected: "argument value",
                                field: args[arg_idx.saturating_sub(1)].to_string(),
                            },
                            span: Span::new(arg_idx.saturating_sub(1), 0),
                        }),
                    );
                }

                let arg = args[arg_idx];
                let span = Span::new(arg_idx, 1);

                // Skip this value if it starts with - (it's probably another flag)
                if arg.starts_with('-') {
                    // This means we're missing a value for the previous argument
                    return (
                        nd,
                        Err(Spanned {
                            node: DeserErrorKind::MissingValue {
                                expected: "argument value",
                                field: args[arg_idx.saturating_sub(1)].to_string(),
                            },
                            span: Span::new(arg_idx.saturating_sub(1), 0),
                        }),
                    );
                }

                // Check for slice/vec types that should be comma-separated
                if shape.is_type::<Vec<u64>>() || shape.is_type::<Vec<String>>() {
                    eprintln!("Hello Vec<u64>");
                    // For Vec<u64> wrapped in Opaque, start a list
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::ListStarted,
                            span: Span::new(arg_idx, 0), // Do not advance, the list is
                                                         // delimiter-separated in this arg
                        }),
                    );
                }

                // Try to parse as appropriate type
                // Handle numeric types
                if let Ok(v) = arg.parse::<u64>() {
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::Scalar(Scalar::U64(v)),
                            span,
                        }),
                    );
                }
                if let Ok(v) = arg.parse::<i64>() {
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::Scalar(Scalar::I64(v)),
                            span,
                        }),
                    );
                }
                if let Ok(v) = arg.parse::<f64>() {
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::Scalar(Scalar::F64(v)),
                            span,
                        }),
                    );
                }

                // Default to string type
                (
                    nd,
                    Ok(Spanned {
                        node: Outcome::Scalar(Scalar::String(Cow::Borrowed(arg))),
                        span,
                    }),
                )
            }

            // List items
            Expectation::ListItemOrListClose => {
                // eprintln!("Hello mum, {}", args[arg_idx]);
                eprintln!("Hello mum");
                // End the list if we're out of arguments, or if it's a new flag
                if arg_idx >= args.len() || args[arg_idx].starts_with('-') {
                    eprintln!("gootbye");
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::ListEnded,
                            span: Span::new(arg_idx, 0),
                        }),
                    );
                }

                let arg = args[arg_idx];
                let span = Span::new(arg_idx, 1);

                // Try to parse as appropriate type
                // Handle numeric types
                if let Ok(v) = arg.parse::<u64>() {
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::Scalar(Scalar::U64(v)),
                            span,
                        }),
                    );
                }
                if let Ok(v) = arg.parse::<i64>() {
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::Scalar(Scalar::I64(v)),
                            span,
                        }),
                    );
                }
                if let Ok(v) = arg.parse::<f64>() {
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::Scalar(Scalar::F64(v)),
                            span,
                        }),
                    );
                }

                // Default to string type
                (
                    nd,
                    Ok(Spanned {
                        node: Outcome::Scalar(Scalar::String(Cow::Borrowed(arg))),
                        span,
                    }),
                )
            }
        }
    }

    fn skip<'input, 'facet, 'shape>(
        &mut self,
        nd: NextData<'input, 'facet, 'shape, Self::SpanType, Self::Input<'input>>,
    ) -> NextResult<
        'input,
        'facet,
        'shape,
        Span<Self::SpanType>,
        Spanned<DeserErrorKind<'shape>, Self::SpanType>,
        Self::SpanType,
        Self::Input<'input>,
    >
    where
        'shape: 'input,
    {
        let arg_idx = nd.start();
        let args = nd.input();

        if arg_idx < args.len() {
            // Simply skip one position
            (nd, Ok(Span::new(arg_idx, 1)))
        } else {
            // No argument to skip
            (
                nd,
                Err(Spanned {
                    node: DeserErrorKind::UnexpectedEof {
                        wanted: "argument to skip",
                    },
                    span: Span::new(arg_idx, 1),
                }),
            )
        }
    }
}
