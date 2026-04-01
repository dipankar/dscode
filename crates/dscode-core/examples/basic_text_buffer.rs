//! Basic text buffer operations example.
//!
//! Demonstrates creating a rope-backed buffer and extracting text.

use dscode_core::TextBuffer;

fn main() {
    // Create a new buffer from a string
    let buffer = TextBuffer::new("Hello, world!");
    println!("Text: {}", buffer.get_text());
    println!("Length: {} chars", buffer.len_chars());
    println!("Is empty: {}", buffer.is_empty());

    // Create an empty buffer
    let empty = TextBuffer::new("");
    println!("Empty buffer length: {}", empty.len_chars());
    assert!(empty.is_empty());
}
