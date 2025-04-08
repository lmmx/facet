//! Pretty printer implementation for Facet types

use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Write},
    hash::{DefaultHasher, Hash, Hasher},
    str,
};

use facet_peek::Peek;
use facet_trait::Facet;

use crate::{ansi, color::ColorGenerator};

/// A formatter for pretty-printing Facet types
pub struct PrettyPrinter {
    indent_size: usize,
    max_depth: Option<usize>,
    color_generator: ColorGenerator,
    use_colors: bool,
}

impl Default for PrettyPrinter {
    fn default() -> Self {
        Self {
            indent_size: 2,
            max_depth: None,
            color_generator: ColorGenerator::default(),
            use_colors: true,
        }
    }
}

/// Stack state for iterative formatting
enum StackState {
    Start,
    ProcessStructField { field_index: usize },
    ProcessListItem { item_index: usize },
    ProcessMapEntry,
    Finish,
}

/// Stack item for iterative traversal
struct StackItem<'a> {
    peek: Peek<'a>,
    format_depth: usize,
    type_depth: usize,
    state: StackState,
}

impl PrettyPrinter {
    /// Create a new PrettyPrinter with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the indentation size
    pub fn with_indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }

    /// Set the maximum depth for recursive printing
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Set the color generator
    pub fn with_color_generator(mut self, generator: ColorGenerator) -> Self {
        self.color_generator = generator;
        self
    }

    /// Enable or disable colors
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }

    /// Format a value to a string
    pub fn format<T: Facet>(&self, value: &T) -> String {
        let peek = Peek::new(value);

        let mut output = String::new();
        self.format_peek_internal(peek, &mut output, 0, 0, &mut HashMap::new())
            .expect("Formatting failed");

        output
    }

    /// Format a value to a formatter
    pub fn format_to<T: Facet>(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let peek = Peek::new(value);
        self.format_peek_internal(peek, f, 0, 0, &mut HashMap::new())
    }

    /// Format a Peek value to a string
    pub fn format_peek(&self, peek: Peek<'_>) -> String {
        let mut output = String::new();
        self.format_peek_internal(peek, &mut output, 0, 0, &mut HashMap::new())
            .expect("Formatting failed");
        output
    }

    /// Internal method to format a Peek value
    pub(crate) fn format_peek_internal(
        &self,
        peek: Peek<'_>,
        f: &mut impl Write,
        format_depth: usize,
        type_depth: usize,
        visited: &mut HashMap<*const (), usize>,
    ) -> fmt::Result {
        // Create a queue for our stack items
        let mut stack = VecDeque::new();

        // Push the initial item
        stack.push_back(StackItem {
            peek,
            format_depth,
            type_depth,
            state: StackState::Start,
        });

        // Process items until the stack is empty
        while let Some(mut item) = stack.pop_back() {
            match item.state {
                StackState::Start => {
                    // Check if we've reached the maximum depth
                    if let Some(max_depth) = self.max_depth {
                        if item.format_depth > max_depth {
                            self.write_punctuation(f, "[")?;
                            write!(f, "...")?;
                            continue;
                        }
                    }

                    // Get the data pointer for cycle detection
                    let ptr = unsafe { item.peek.data().as_ptr() };

                    // Check for cycles - if we've seen this pointer before at a different type_depth
                    if let Some(&ptr_type_depth) = visited.get(&ptr) {
                        // If the current type_depth is significantly deeper than when we first saw this pointer,
                        // we have a true cycle, not just a transparent wrapper
                        if item.type_depth > ptr_type_depth + 1 {
                            self.write_type_name(f, &item.peek)?;
                            self.write_punctuation(f, " { ")?;
                            self.write_comment(
                                f,
                                &format!(
                                    "/* cycle detected at {:p} (first seen at type_depth {}) */",
                                    ptr, ptr_type_depth
                                ),
                            )?;
                            self.write_punctuation(f, " }")?;
                            continue;
                        }
                    } else {
                        // First time seeing this pointer, record its type_depth
                        visited.insert(ptr, item.type_depth);
                    }

                    // Process based on the peek variant
                    match item.peek {
                        Peek::Value(value) => {
                            self.format_value(value, f)?;
                        }
                        Peek::Struct(struct_) => {
                            // When recursing into a struct, always increment format_depth
                            // Only increment type_depth if we're moving to a different address
                            let new_type_depth =
                                if std::ptr::eq(unsafe { struct_.data().as_ptr() }, ptr) {
                                    item.type_depth // Same pointer, don't increment type_depth
                                } else {
                                    item.type_depth + 1 // Different pointer, increment type_depth
                                };

                            // Print the struct name
                            self.write_type_name(f, &struct_)?;
                            self.write_punctuation(f, " {")?;

                            if struct_.field_count() == 0 {
                                self.write_punctuation(f, " }")?;
                                continue;
                            }

                            writeln!(f)?;

                            // Push back the item with the next state to continue processing fields
                            item.state = StackState::ProcessStructField { field_index: 0 };
                            item.format_depth += 1;
                            item.type_depth = new_type_depth;
                            stack.push_back(item);
                        }
                        Peek::List(list) => {
                            // When recursing into a list, always increment format_depth
                            // Only increment type_depth if we're moving to a different address
                            let new_type_depth =
                                if std::ptr::eq(unsafe { list.data().as_ptr() }, ptr) {
                                    item.type_depth // Same pointer, don't increment type_depth
                                } else {
                                    item.type_depth + 1 // Different pointer, increment type_depth
                                };

                            // Print the list name
                            self.write_type_name(f, &list)?;
                            self.write_punctuation(f, " [")?;
                            writeln!(f)?;

                            // Push back the item with the next state to continue processing list items
                            item.state = StackState::ProcessListItem { item_index: 0 };
                            item.format_depth += 1;
                            item.type_depth = new_type_depth;
                            stack.push_back(item);
                        }
                        Peek::Map(map) => {
                            // Print the map name
                            self.write_type_name(f, &map)?;
                            self.write_punctuation(f, " {")?;
                            writeln!(f)?;

                            // Push back the item with the next state to continue processing map
                            item.state = StackState::ProcessMapEntry;
                            item.format_depth += 1;
                            // When recursing into a map, always increment format_depth
                            // Only increment type_depth if we're moving to a different address
                            item.type_depth = if std::ptr::eq(unsafe { map.data().as_ptr() }, ptr) {
                                item.type_depth // Same pointer, don't increment type_depth
                            } else {
                                item.type_depth + 1 // Different pointer, increment type_depth
                            };
                            stack.push_back(item);
                        }
                        _ => {
                            writeln!(f, "unsupported peek variant: {:?}", item.peek)?;
                        }
                    }
                }
                StackState::ProcessStructField { field_index } => {
                    if let Peek::Struct(struct_) = item.peek {
                        let fields: Vec<_> = struct_.fields_with_metadata().collect();

                        if field_index >= fields.len() {
                            // All fields processed, write closing brace
                            write!(
                                f,
                                "{:width$}{}",
                                "",
                                self.style_punctuation("}"),
                                width = (item.format_depth - 1) * self.indent_size
                            )?;
                            continue;
                        }

                        let (_, field_name, field_value, flags) = &fields[field_index];

                        // Indent
                        write!(
                            f,
                            "{:width$}",
                            "",
                            width = item.format_depth * self.indent_size
                        )?;

                        // Field name
                        self.write_field_name(f, field_name)?;
                        self.write_punctuation(f, ": ")?;

                        // Check if field is sensitive
                        if flags.contains(facet_trait::FieldFlags::SENSITIVE) {
                            // Field value is sensitive, use write_redacted
                            self.write_redacted(f, "[REDACTED]")?;
                            self.write_punctuation(f, ",")?;
                            writeln!(f)?;

                            // Process next field
                            item.state = StackState::ProcessStructField {
                                field_index: field_index + 1,
                            };
                            stack.push_back(item);
                        } else {
                            // Field value is not sensitive, format normally
                            // Push back current item to continue after formatting field value
                            item.state = StackState::ProcessStructField {
                                field_index: field_index + 1,
                            };

                            let finish_item = StackItem {
                                peek: *field_value,
                                format_depth: item.format_depth,
                                type_depth: item.type_depth + 1,
                                state: StackState::Finish,
                            };
                            let start_item = StackItem {
                                peek: *field_value,
                                format_depth: item.format_depth,
                                type_depth: item.type_depth + 1,
                                state: StackState::Start,
                            };

                            stack.push_back(item);
                            stack.push_back(finish_item);
                            stack.push_back(start_item);
                        }
                    }
                }
                StackState::ProcessListItem { item_index } => {
                    if let Peek::List(list) = item.peek {
                        if item_index >= list.len() {
                            // All items processed, write closing bracket
                            write!(
                                f,
                                "{:width$}",
                                "",
                                width = (item.format_depth - 1) * self.indent_size
                            )?;
                            self.write_punctuation(f, "]")?;
                            continue;
                        }

                        // Indent
                        write!(
                            f,
                            "{:width$}",
                            "",
                            width = item.format_depth * self.indent_size
                        )?;

                        // Push back current item to continue after formatting list item
                        item.state = StackState::ProcessListItem {
                            item_index: item_index + 1,
                        };
                        let next_format_depth = item.format_depth;
                        let next_type_depth = item.type_depth + 1;
                        stack.push_back(item);

                        // Push list item to format first
                        let list_item = list.iter().nth(item_index).unwrap();
                        stack.push_back(StackItem {
                            peek: list_item,
                            format_depth: next_format_depth,
                            type_depth: next_type_depth,
                            state: StackState::Finish,
                        });

                        // When we push a list item to format, we need to process it from the beginning
                        stack.push_back(StackItem {
                            peek: list_item,
                            format_depth: next_format_depth,
                            type_depth: next_type_depth,
                            state: StackState::Start, // Use Start state to properly process the item
                        });
                    }
                }
                StackState::ProcessMapEntry => {
                    if let Peek::Map(_) = item.peek {
                        // TODO: Implement proper map iteration when available in facet_peek

                        // Indent
                        write!(
                            f,
                            "{:width$}",
                            "",
                            width = item.format_depth * self.indent_size
                        )?;
                        write!(f, "{}", self.style_comment("/* Map contents */"))?;
                        writeln!(f)?;

                        // Closing brace with proper indentation
                        write!(
                            f,
                            "{:width$}{}",
                            "",
                            self.style_punctuation("}"),
                            width = (item.format_depth - 1) * self.indent_size
                        )?;
                    }
                }
                StackState::Finish => {
                    // This state is reached after processing a field or list item
                    // Add comma and newline for struct fields and list items
                    self.write_punctuation(f, ",")?;
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }

    /// Format a scalar value
    fn format_value(&self, value: facet_peek::PeekValue, f: &mut impl Write) -> fmt::Result {
        // Generate a color for this shape
        let mut hasher = DefaultHasher::new();
        value.shape().def.hash(&mut hasher);
        let hash = hasher.finish();
        let color = self.color_generator.generate_color(hash);

        // Apply color if needed
        if self.use_colors {
            color.write_fg(f)?;
        }

        // Display the value
        struct DisplayWrapper<'a>(&'a facet_peek::PeekValue<'a>);

        impl fmt::Display for DisplayWrapper<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if self.0.display(f).is_none() {
                    // If the value doesn't implement Display, use Debug
                    if self.0.debug(f).is_none() {
                        // If the value doesn't implement Debug either, just show the type name
                        self.0.type_name(f, facet_trait::TypeNameOpts::infinite())?;
                        write!(f, "(⋯)")?;
                    }
                }
                Ok(())
            }
        }

        write!(f, "{}", DisplayWrapper(&value))?;

        // Reset color if needed
        if self.use_colors {
            ansi::write_reset(f)?;
        }

        Ok(())
    }

    /// Write styled type name to formatter
    fn write_type_name<W: fmt::Write>(
        &self,
        f: &mut W,
        peek: &facet_peek::PeekValue,
    ) -> fmt::Result {
        struct TypeNameWriter<'a, 'b: 'a>(&'b facet_peek::PeekValue<'a>);

        impl std::fmt::Display for TypeNameWriter<'_, '_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.type_name(f, facet_trait::TypeNameOpts::infinite())
            }
        }
        let type_name = TypeNameWriter(peek);

        if self.use_colors {
            ansi::write_bold(f)?;
            write!(f, "{}", type_name)?;
            ansi::write_reset(f)
        } else {
            write!(f, "{}", type_name)
        }
    }

    /// Style a type name and return it as a string
    #[allow(dead_code)]
    fn style_type_name(&self, peek: &facet_peek::PeekValue) -> String {
        let mut result = String::new();
        self.write_type_name(&mut result, peek).unwrap();
        result
    }

    /// Write styled field name to formatter
    fn write_field_name<W: fmt::Write>(&self, f: &mut W, name: &str) -> fmt::Result {
        if self.use_colors {
            ansi::write_rgb(f, 114, 160, 193)?;
            write!(f, "{}", name)?;
            ansi::write_reset(f)
        } else {
            write!(f, "{}", name)
        }
    }

    /// Write styled punctuation to formatter
    fn write_punctuation<W: fmt::Write>(&self, f: &mut W, text: &str) -> fmt::Result {
        if self.use_colors {
            ansi::write_dim(f)?;
            write!(f, "{}", text)?;
            ansi::write_reset(f)
        } else {
            write!(f, "{}", text)
        }
    }

    /// Style punctuation and return it as a string
    fn style_punctuation(&self, text: &str) -> String {
        let mut result = String::new();
        self.write_punctuation(&mut result, text).unwrap();
        result
    }

    /// Write styled comment to formatter
    fn write_comment<W: fmt::Write>(&self, f: &mut W, text: &str) -> fmt::Result {
        if self.use_colors {
            ansi::write_dim(f)?;
            write!(f, "{}", text)?;
            ansi::write_reset(f)
        } else {
            write!(f, "{}", text)
        }
    }

    /// Style a comment and return it as a string
    fn style_comment(&self, text: &str) -> String {
        let mut result = String::new();
        self.write_comment(&mut result, text).unwrap();
        result
    }

    /// Write styled redacted value to formatter
    fn write_redacted<W: fmt::Write>(&self, f: &mut W, text: &str) -> fmt::Result {
        if self.use_colors {
            ansi::write_rgb(f, 224, 49, 49)?; // Use bright red for redacted values
            ansi::write_bold(f)?;
            write!(f, "{}", text)?;
            ansi::write_reset(f)
        } else {
            write!(f, "{}", text)
        }
    }

    /// Style a redacted value and return it as a string
    #[allow(dead_code)]
    fn style_redacted(&self, text: &str) -> String {
        let mut result = String::new();
        self.write_redacted(&mut result, text).unwrap();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic tests for the PrettyPrinter
    #[test]
    fn test_pretty_printer_default() {
        let printer = PrettyPrinter::default();
        assert_eq!(printer.indent_size, 2);
        assert_eq!(printer.max_depth, None);
        assert!(printer.use_colors);
    }

    #[test]
    fn test_pretty_printer_with_methods() {
        let printer = PrettyPrinter::new()
            .with_indent_size(4)
            .with_max_depth(3)
            .with_colors(false);

        assert_eq!(printer.indent_size, 4);
        assert_eq!(printer.max_depth, Some(3));
        assert!(!printer.use_colors);
    }
}
