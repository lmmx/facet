use alloc::{borrow::Cow, format};

use facet_core::{Facet, SequenceType, Type};
pub use facet_deserialize::{DeserError, DeserErrorKind};
use facet_deserialize::{
    Expectation, Format, NextData, NextResult, Outcome, Scalar, Span, Spannable, Spanned,
};
use log::trace;

mod tokenizer;
use tokenizer::{Token, TokenError, TokenErrorKind, Tokenizer};

/// Deserialize JSON from a given byte slice
pub fn from_slice<'input: 'facet, 'facet, T: Facet<'facet>>(
    input: &'input [u8],
) -> Result<T, DeserError<'input>> {
    facet_deserialize::deserialize(input, Json)
}

/// Deserialize JSON from a given string
pub fn from_str<'input: 'facet, 'facet, T: Facet<'facet>>(
    input: &'input str,
) -> Result<T, DeserError<'input>> {
    let input = input.as_bytes();
    facet_deserialize::deserialize(input, Json)
}

/// Deserialize JSON from a given string, converting any dynamic error into a static one.
///
/// This function attempts to deserialize a type `T` implementing `Facet` from the input string slice.
/// If deserialization fails, the error is converted into an owned, static error type to avoid lifetime issues.
pub fn from_str_static_error<'input: 'facet, 'facet, T: Facet<'facet>>(
    input: &'input str,
) -> Result<T, DeserError<'input>> {
    let input = input.as_bytes();
    facet_deserialize::deserialize(input, Json).map_err(|e| e.into_owned())
}

/// The JSON format
pub struct Json;

impl Format for Json {
    fn next<'input, 'facet>(
        &mut self,
        nd: NextData<'input, 'facet>,
        mut expectation: Expectation,
    ) -> NextResult<'input, 'facet, Spanned<Outcome<'input>>, Spanned<DeserErrorKind>> {
        let input = &nd.input()[nd.start()..];
        let mut tokenizer = Tokenizer::new(input);

        loop {
            let token = match tokenizer.next_token() {
                Ok(token) => token,
                Err(err) => {
                    trace!("Tokenizer error in next: {:?}", err.kind);
                    return (nd, Err(convert_token_error(err)));
                }
            };

            // Adjust token span to be relative to the beginning of the overall input
            let token_offset = nd.start();
            let span = Span::new(token.span.start() + token_offset, token.span.len());

            let res = match token.node {
                // // Token::String(s) => Ok(Spanned {
                // //     node: Outcome::Scalar(Scalar::String(Cow::Owned(s))),
                // //     span,
                // // }),
                // Token::String(s) => {
                //     let shape_type = &nd.wip.shape().ty;

                //     // Direct &str check
                //     let is_direct_str = nd.wip.shape().is_type::<&str>();

                //     // Check if we're in an array of &str
                //     let is_str_array_element = match shape_type {
                //         Type::Sequence(SequenceType::Array(array_type)) => {
                //             match &array_type.t.ty {
                //                 // Check if the pointer target is a string type
                //                 Type::Pointer(ptr_type) => array_type.t.is_type::<&str>(),
                //                 _ => false
                //             }
                //         },
                //         _ => false
                //     };

                //     // Now handle the string based on our detection
                //     let scalar = if is_direct_str || is_str_array_element {
                //         // Extract the actual string content from the JSON string token
                //         // (removing surrounding quotes)
                //         Scalar::String(Cow::Borrowed(&s))
                //     } else {
                //         // Normal case - use owned string
                //         Scalar::String(Cow::Owned(s))
                //     };

                //     Ok(Spanned {
                //         node: Outcome::Scalar(scalar),
                //         span,
                //     })
                // }
                Token::String(s) => {
                    eprintln!("String token: {:?}", s);

                    // Determine if we need a &str vs String based on the type
                    let needs_str_reference = match &nd.wip.shape().ty {
                        Type::Pointer(ptr_type) => true, // Direct &str
                        Type::Sequence(SequenceType::Array(array_type)) => {
                            // Check if array element is &str
                            match &array_type.t.ty {
                                // Check if the pointer target is a string type
                                Type::Pointer(ptr_type) => array_type.t.is_type::<&str>(),
                                _ => false,
                            }
                        }
                        _ => false,
                    };

                    let scalar = if needs_str_reference {
                        // WARNING: Memory leak - this creates a string that lives for the entire program
                        // This is a temporary solution until a proper string arena is implemented
                        let static_str: &'static str = Box::leak(s.clone().into_boxed_str());
                        Scalar::String(Cow::Borrowed(static_str))
                    } else {
                        // For regular String, use the owned string
                        Scalar::String(Cow::Owned(s))
                    };

                    Ok(Spanned {
                        node: Outcome::Scalar(scalar),
                        span,
                    })
                }
                Token::F64(n) => Ok(Spanned {
                    node: Outcome::Scalar(Scalar::F64(n)),
                    span,
                }),
                Token::I64(n) => Ok(Spanned {
                    node: Outcome::Scalar(Scalar::I64(n)),
                    span,
                }),
                Token::U64(n) => Ok(Spanned {
                    node: Outcome::Scalar(Scalar::U64(n)),
                    span,
                }),
                Token::True => Ok(Spanned {
                    node: Outcome::Scalar(Scalar::Bool(true)),
                    span,
                }),
                Token::False => Ok(Spanned {
                    node: Outcome::Scalar(Scalar::Bool(false)),
                    span,
                }),
                Token::Null => Ok(Spanned {
                    node: Outcome::Scalar(Scalar::Null),
                    span,
                }),
                Token::LBrace => Ok(Spanned {
                    node: Outcome::ObjectStarted,
                    span,
                }),
                Token::RBrace => {
                    if expectation == Expectation::ObjectKeyOrObjectClose {
                        Ok(Spanned {
                            node: Outcome::ObjectEnded,
                            span,
                        })
                    } else {
                        trace!("Did not expect closing brace, expected {:?}", expectation);
                        Err(DeserErrorKind::UnexpectedChar {
                            got: '}',
                            wanted: "a value",
                        }
                        .with_span(span))
                    }
                }
                Token::LBracket => Ok(Spanned {
                    node: Outcome::ListStarted,
                    span,
                }),
                Token::RBracket => {
                    if expectation == Expectation::ListItemOrListClose {
                        Ok(Spanned {
                            node: Outcome::ListEnded,
                            span,
                        })
                    } else {
                        Err(DeserErrorKind::UnexpectedChar {
                            got: ']',
                            wanted: "a value",
                        }
                        .with_span(span))
                    }
                }
                Token::Colon => {
                    if expectation == Expectation::ObjectVal {
                        expectation = Expectation::Value;
                        continue;
                    } else {
                        trace!("Did not expect ObjectValue, expected {:?}", expectation);
                        Err(DeserErrorKind::UnexpectedChar {
                            got: ':',
                            wanted: "a value, not a colon",
                        }
                        .with_span(span))
                    }
                }
                Token::Comma => match expectation {
                    Expectation::ListItemOrListClose | Expectation::ObjectKeyOrObjectClose => {
                        expectation = Expectation::Value;
                        continue;
                    }
                    other => {
                        trace!("Did not expect comma, expected {:?}", other);
                        Err(DeserErrorKind::UnexpectedChar {
                            got: ',',
                            wanted: "<value or key>",
                        }
                        .with_span(span))
                    }
                },
                Token::Eof => {
                    return (
                        nd,
                        Err(DeserErrorKind::UnexpectedEof {
                            wanted: "any value (got EOF)",
                        }
                        .with_span(span)),
                    );
                }
            };

            return (nd, res);
        }
    }

    fn skip<'input, 'facet>(
        &mut self,
        nd: NextData<'input, 'facet>,
    ) -> NextResult<'input, 'facet, Span, Spanned<DeserErrorKind>> {
        trace!("Starting skip at offset {}", nd.start());
        let input = &nd.input()[nd.start()..];
        let mut tokenizer = Tokenizer::new(input);

        loop {
            let token = match tokenizer.next_token() {
                Ok(token) => token,
                Err(err) => {
                    trace!("Tokenizer error on initial token: {:?}", err.kind);
                    return (nd, Err(convert_token_error(err)));
                }
            };

            let res = match token.node {
                Token::LBrace | Token::LBracket => {
                    let mut depth = 1;
                    let mut last_span = token.span;
                    while depth > 0 {
                        let token = match tokenizer.next_token() {
                            Ok(token) => token,
                            Err(err) => {
                                trace!("Tokenizer error while skipping container: {:?}", err.kind);
                                return (nd, Err(convert_token_error(err)));
                            }
                        };

                        match token.node {
                            Token::LBrace | Token::LBracket => {
                                depth += 1;
                                last_span = token.span;
                            }
                            Token::RBrace | Token::RBracket => {
                                depth -= 1;
                                last_span = token.span;
                            }
                            _ => {
                                last_span = token.span;
                            }
                        }
                    }
                    (nd, Ok(last_span))
                }
                Token::String(_)
                | Token::F64(_)
                | Token::I64(_)
                | Token::U64(_)
                | Token::True
                | Token::False
                | Token::Null => (nd, Ok(token.span)),
                Token::Colon => {
                    // Skip colon token
                    continue;
                }
                other => (
                    nd,
                    Err(DeserErrorKind::UnexpectedChar {
                        got: format!("{:?}", other).chars().next().unwrap_or('?'),
                        wanted: "value",
                    }
                    .with_span(Span::new(token.span.start(), token.span.len()))),
                ),
            };
            let (nd, mut span) = res;
            if let Ok(valid_span) = &mut span {
                let offset = nd.start();
                valid_span.start += offset;
            }
            return (nd, span);
        }
    }
}

fn convert_token_error(err: TokenError) -> Spanned<DeserErrorKind> {
    match err.kind {
        TokenErrorKind::UnexpectedCharacter(c) => DeserErrorKind::UnexpectedChar {
            got: c,
            wanted: "valid JSON character",
        }
        .with_span(err.span),
        TokenErrorKind::UnexpectedEof(why) => {
            DeserErrorKind::UnexpectedEof { wanted: why }.with_span(err.span)
        }
        TokenErrorKind::InvalidUtf8(s) => DeserErrorKind::InvalidUtf8(s).with_span(err.span),
        TokenErrorKind::NumberOutOfRange(number) => {
            DeserErrorKind::NumberOutOfRange(number).with_span(err.span)
        }
    }
}
