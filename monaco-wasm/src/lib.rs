use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum TokenType {
    Keyword = 0,
    Identifier = 1,
    String = 2,
    Number = 3,
    Comment = 4,
    Operator = 5,
    Whitespace = 6,
}

#[wasm_bindgen]
pub struct FastTokenizer {
    keywords: Vec<String>,
}

#[wasm_bindgen]
impl FastTokenizer {
    #[wasm_bindgen(constructor)]
    pub fn new(language: &str) -> Self {
        let keywords = match language {
            "javascript" | "typescript" => vec![
                "const", "let", "var", "function", "class", "if", "else", "for", "while", "return",
                "import", "export", "async", "await",
            ],
            "rust" => vec![
                "fn", "let", "mut", "const", "struct", "enum", "impl", "pub", "use", "mod", "if",
                "else", "match", "return",
            ],
            "python" => vec![
                "def", "class", "if", "else", "elif", "for", "while", "return", "import", "from",
                "as", "with", "try", "except",
            ],
            _ => vec![],
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        FastTokenizer { keywords }
    }

    /// Tokenize text and return flat array: [start, length, type, start, length, type, ...]
    #[wasm_bindgen]
    pub fn tokenize(&self, text: &str) -> Vec<u32> {
        let mut tokens = Vec::new();
        let bytes = text.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            let start = i;
            let ch = bytes[i] as char;

            let (length, token_type) = if ch.is_whitespace() {
                (self.consume_whitespace(bytes, i), TokenType::Whitespace)
            } else if ch == '/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                (self.consume_line_comment(bytes, i), TokenType::Comment)
            } else if ch == '"' || ch == '\'' {
                (self.consume_string(bytes, i, ch), TokenType::String)
            } else if ch.is_numeric() {
                (self.consume_number(bytes, i), TokenType::Number)
            } else if ch.is_alphabetic() || ch == '_' {
                let word_len = self.consume_word(bytes, i);
                let word = &text[i..i + word_len];
                if self.keywords.iter().any(|k| k == word) {
                    (word_len, TokenType::Keyword)
                } else {
                    (word_len, TokenType::Identifier)
                }
            } else {
                (1, TokenType::Operator)
            };

            tokens.push(start as u32);
            tokens.push(length as u32);
            tokens.push(token_type as u32);
            i += length;
        }

        tokens
    }

    fn consume_whitespace(&self, bytes: &[u8], start: usize) -> usize {
        let mut i = start;
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        i - start
    }

    fn consume_line_comment(&self, bytes: &[u8], start: usize) -> usize {
        let mut i = start;
        while i < bytes.len() && bytes[i] != b'\n' {
            i += 1;
        }
        i - start
    }

    fn consume_string(&self, bytes: &[u8], start: usize, quote: char) -> usize {
        let mut i = start + 1;
        let quote_byte = quote as u8;
        while i < bytes.len() {
            if bytes[i] == quote_byte {
                return i - start + 1;
            }
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
            } else {
                i += 1;
            }
        }
        i - start
    }

    fn consume_number(&self, bytes: &[u8], start: usize) -> usize {
        let mut i = start;
        while i < bytes.len() && (bytes[i] as char).is_numeric() {
            i += 1;
        }
        i - start
    }

    fn consume_word(&self, bytes: &[u8], start: usize) -> usize {
        let mut i = start;
        while i < bytes.len() {
            let ch = bytes[i] as char;
            if ch.is_alphanumeric() || ch == '_' {
                i += 1;
            } else {
                break;
            }
        }
        i - start
    }
}

#[wasm_bindgen]
pub fn benchmark_tokenize(text: &str, iterations: u32) -> f64 {
    let tokenizer = FastTokenizer::new("javascript");
    let start = js_sys::Date::now();

    for _ in 0..iterations {
        tokenizer.tokenize(text);
    }

    js_sys::Date::now() - start
}
