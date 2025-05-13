use crate::error::{ArgsError, ArgsErrorKind};
use alloc::borrow::Cow;
use alloc::string::ToString;
use core::cell::RefCell;
use core::fmt;
use facet_core::{Facet, FieldAttribute, Type, UserType};
use facet_deserialize::{
    DeserErrorKind, Expectation, Format, NextData, NextResult, Outcome, Scalar, Span, Spannable,
    Spanned,
};

/// Command-line argument format for Facet deserialization
pub struct CliFormat<'a> {
    args: &'a [&'a str],
}

impl fmt::Display for CliFormat<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CliFormat")
    }
}

impl<'a> CliFormat<'a> {
    /// Create a new CLI format instance
    pub fn new(args: &'a [&'a str]) -> Self {
        Self { args }
    }

    /// Helper function to convert kebab-case to snake_case
    pub(crate) fn kebab_to_snake(input: &str) -> Cow<str> {
        if !input.contains('-') {
            return Cow::Borrowed(input);
        }
        Cow::Owned(input.replace('-', "_"))
    }
}

/// Parses command line arguments using the Facet deserialization framework
pub fn from_slice_with_format<'input, 'facet, T>(s: &'facet [&'input str]) -> Result<T, ArgsError>
where
    T: Facet<'facet>,
    'input: 'facet,
{
    // Check if the type is a struct first - return early if not
    let shape = T::SHAPE;
    if !matches!(shape.ty, Type::User(UserType::Struct(_))) {
        return Err(ArgsError::new(ArgsErrorKind::GenericArgsError(
            "Expected struct type".to_string(),
        )));
    }

    // Set up the arguments for tracking
    ARGS_LIST.with(|cell| {
        *cell.borrow_mut() = s.iter().map(|s| s.to_string()).collect();
    });

    // Convert string slice arguments to bytes for deserialization
    // For CLI args, we're not actually using the byte representation,
    // but we need to pass something to the deserialize function
    let dummy_bytes = &[];
    let format = CliFormat::new(s);

    // Use the Format trait for deserialization
    facet_deserialize::deserialize(dummy_bytes, format).map_err(|e| match e.kind {
        DeserErrorKind::ReflectError(re) => ArgsError::new(ArgsErrorKind::GenericReflect(re)),
        DeserErrorKind::MissingField(field) => ArgsError::new(ArgsErrorKind::GenericArgsError(
            format!("Missing required field: {}", field),
        )),
        DeserErrorKind::UnknownField { field_name, .. } => {
            if field_name == "positional argument" {
                // This happens when parsing non-struct types, so we'll return a specific error
                ArgsError::new(ArgsErrorKind::GenericArgsError(
                    "Expected struct type".to_string(),
                ))
            } else {
                ArgsError::new(ArgsErrorKind::GenericArgsError(format!(
                    "Unknown argument `{}`",
                    field_name
                )))
            }
        }
        DeserErrorKind::UnexpectedEof { wanted } => {
            if wanted == "argument value" {
                // Special error handling for CLI argument context
                CURRENT_ARG_NAME.with(|cell| {
                    let arg_name = cell.borrow().clone();
                    if !arg_name.is_empty() {
                        ArgsError::new(ArgsErrorKind::GenericArgsError(format!(
                            "expected value after argument `{}`",
                            arg_name
                        )))
                    } else {
                        ArgsError::new(ArgsErrorKind::GenericArgsError(
                            "expected value after argument".to_string(),
                        ))
                    }
                })
            } else if wanted == "argument to skip" {
                // Special error for skipping arguments
                CURRENT_ARG_NAME.with(|cell| {
                    let arg_name = cell.borrow().clone();
                    if !arg_name.is_empty() {
                        ArgsError::new(ArgsErrorKind::GenericArgsError(format!(
                            "Unknown argument `{}`",
                            arg_name
                        )))
                    } else {
                        ArgsError::new(ArgsErrorKind::GenericArgsError(
                            "Unknown argument".to_string(),
                        ))
                    }
                })
            } else {
                ArgsError::new(ArgsErrorKind::GenericArgsError(wanted.to_string()))
            }
        }
        _ => ArgsError::new(ArgsErrorKind::GenericArgsError(format!(
            "Error parsing arguments: {}",
            e
        ))),
    })
}

// Thread-local storage for static strings
thread_local! {
    static STATIC_STRINGS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static CURRENT_ARG_NAME: RefCell<String> = const { RefCell::new(String::new()) };
    static ARGS_LIST: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

impl Format for CliFormat<'_> {
    fn next<'i, 'facet>(
        &mut self,
        nd: NextData<'i, 'facet>,
        expectation: Expectation,
    ) -> NextResult<'i, 'facet, Spanned<Outcome<'i>>, Spanned<DeserErrorKind>> {
        let pos = nd.start();
        let shape = nd.wip.shape();

        match expectation {
            // Top-level value
            Expectation::Value => {
                // Check if it's a struct type
                if !matches!(shape.ty, Type::User(UserType::Struct(_))) {
                    return (
                        nd,
                        Err(DeserErrorKind::UnsupportedType {
                            got: shape,
                            wanted: "struct",
                        }
                        .with_span(Span::new(pos, 0))),
                    );
                }
                // For CLI args, we always start with an object (struct)
                (
                    nd,
                    Ok(Spanned {
                        node: Outcome::ObjectStarted,
                        span: Span::new(pos, 0),
                    }),
                )
            }

            // Object key (or finished)
            Expectation::ObjectKeyOrObjectClose => {
                /* Check if we have more arguments ? */
                if pos < self.args.len() {
                    let arg = self.args[pos];
                    let span = Span::new(pos, 1);

                    // Named long argument?
                    if let Some(key) = arg.strip_prefix("--") {
                        let key = Self::kebab_to_snake(key);
                        // Store the current argument name for error context
                        CURRENT_ARG_NAME.with(|c| *c.borrow_mut() = key.to_string());

                        // Check if the field exists in the struct
                        if let Type::User(UserType::Struct(_)) = shape.ty {
                            if nd.wip.field_index(&key).is_none() {
                                return (
                                    nd,
                                    Err(DeserErrorKind::UnknownField {
                                        field_name: key.into_owned(),
                                        shape,
                                    }
                                    .with_span(span)),
                                );
                            }
                        }
                        return (
                            nd,
                            Ok(Spanned {
                                node: Outcome::Scalar(Scalar::String(Cow::Owned(key.into_owned()))),
                                span,
                            }),
                        );
                    }

                    // Short flag?
                    if let Some(key) = arg.strip_prefix('-') {
                        CURRENT_ARG_NAME.with(|c| *c.borrow_mut() = key.to_string());

                        // Convert short argument to field name via shape
                        if let Type::User(UserType::Struct(st)) = shape.ty {
                            for field in st.fields.iter() {
                                for attr in field.attributes {
                                    if let FieldAttribute::Arbitrary(a) = attr {
                                        if a.contains("short") && a.contains(key) {
                                            return (
                                                nd,
                                                Ok(Spanned {
                                                    node: Outcome::Scalar(Scalar::String(
                                                        Cow::Owned(field.name.to_string()),
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
                            Err(DeserErrorKind::UnknownField {
                                field_name: key.to_string(),
                                shape,
                            }
                            .with_span(span)),
                        );
                    }

                    // positional argument
                    if let Type::User(UserType::Struct(st)) = &nd.wip.shape().ty {
                        for (idx, field) in st.fields.iter().enumerate() {
                            for attr in field.attributes.iter() {
                                if let FieldAttribute::Arbitrary(a) = attr {
                                    if a.contains("positional") {
                                        // Check if this field is already set
                                        let is_set = nd.wip.is_field_set(idx).unwrap_or(false);

                                        if !is_set {
                                            // Use this positional field
                                            // The token itself is the *value*
                                            CURRENT_ARG_NAME.with(|c| c.borrow_mut().clear());
                                            return (
                                                nd,
                                                Ok(Spanned {
                                                    node: Outcome::Scalar(Scalar::String(
                                                        Cow::Owned(field.name.to_string()),
                                                    )),
                                                    span: Span::new(pos, 0),
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
                        Err(DeserErrorKind::UnknownField {
                            field_name: "positional argument".to_string(),
                            shape,
                        }
                        .with_span(span)),
                    );
                }

                // EOF: inject implicit-false-if-absent bool flags, if there are any
                if let Type::User(UserType::Struct(st)) = &nd.wip.shape().ty {
                    for (idx, field) in st.fields.iter().enumerate() {
                        if !nd.wip.is_field_set(idx).unwrap_or(false)
                            && field.shape().is_type::<bool>()
                        {
                            return (
                                nd,
                                Ok(Spanned {
                                    node: Outcome::Scalar(Scalar::String(Cow::Owned(
                                        field.name.to_string(),
                                    ))),
                                    span: Span::new(pos, 0),
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
                        span: Span::new(pos, 0),
                    }),
                )
            }

            // Value for the current key
            // We're expecting a value for a named argument
            Expectation::ObjectVal => {
                // Synthetic implicit-false
                if pos >= self.args.len() && shape.is_type::<bool>() {
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::Scalar(Scalar::Bool(false)),
                            span: Span::new(pos, 0),
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
                            span: Span::new(pos, 0),
                        }),
                    );
                }

                // For other types, get the next arg as the value.
                // Need another CLI token:
                if pos >= self.args.len() {
                    return (
                        nd,
                        Err(DeserErrorKind::UnexpectedEof {
                            wanted: "argument value",
                        }
                        .with_span(Span::new(pos, 0))),
                    );
                }

                let arg = self.args[pos];
                let span = Span::new(pos, 1);

                // Skip this value if it starts with - (it's probably another flag)
                if arg.starts_with('-') {
                    // This means we're missing a value for the previous argument
                    return (
                        nd,
                        Err(DeserErrorKind::UnexpectedEof {
                            wanted: "argument value",
                        }
                        .with_span(span)),
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
                // String types
                // `String`: own the token; otherwise: hand out a borrow.
                let scalar = if shape.is_type::<String>() {
                    Scalar::String(Cow::Owned(arg.to_string()))
                } else {
                    // `&str` (or any other borrowed-string compatible shape)
                    let leaked: &'static str = Box::leak(arg.to_string().into_boxed_str());
                    eprintln!("Leak me");
                    Scalar::String(Cow::Borrowed(leaked))
                };

                (
                    nd,
                    Ok(Spanned {
                        node: Outcome::Scalar(scalar),
                        span,
                    }),
                )
            }

            // List items
            // CLI args don't have explicit list support, but we can handle arrays/tuples.
            Expectation::ListItemOrListClose => {
                // End the list if we're out of arguments, or if it's a new flag (which
                // would end the list)
                if pos >= self.args.len() || self.args[pos].starts_with('-') {
                    return (
                        nd,
                        Ok(Spanned {
                            node: Outcome::ListEnded,
                            span: Span::new(pos, 0),
                        }),
                    );
                }
                (
                    nd,
                    Ok(Spanned {
                        node: Outcome::Scalar(Scalar::String(Cow::Owned(
                            self.args[pos].to_string(),
                        ))),
                        span: Span::new(pos, 1),
                    }),
                )
            }
        }
    }

    fn skip<'i, 'facet>(
        &mut self,
        nd: NextData<'i, 'facet>,
    ) -> NextResult<'i, 'facet, Span, Spanned<DeserErrorKind>> {
        let pos = nd.start();
        if pos < self.args.len() {
            // Simply skip one position (span of 1 indicates we consumed 1 argument)
            (nd, Ok(Span::new(pos, 1)))
        } else {
            // No argument to skip
            (
                nd,
                Err(DeserErrorKind::UnexpectedEof {
                    wanted: "argument to skip",
                }
                .with_span(Span::new(pos, 0))),
            )
        }
    }
}
