use std::collections::HashMap;
use std::fmt::Display;
use std::mem::{discriminant, replace};

#[derive(Debug)]
enum Token {
    OpenBrace,
    CloseBrace,
    Colon,
    Comma,
    String(String),
    Null,
    Number(i64),
    Boolean(bool),
    OpenBracket,
    CloseBracket,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::OpenBrace => write!(f, "{{"),
            Token::CloseBrace => write!(f, "}}"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Null => write!(f, "null"),
            Token::Number(n) => write!(f, "{}", n),
            Token::Boolean(b) => write!(f, "{}", b),
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
        }
    }
}

#[derive(Debug)]
enum JsonValue {
    String(String),
    Object(HashMap<String, JsonValue>),
    Number(i64),
    Boolean(bool),
    Array(Vec<JsonValue>),
    Null,
}

#[derive(Debug)]
enum JsonErr {
    Lexer(LexErr),
    Parser(ParseErr),
}

#[derive(Debug)]
enum LexErr {
    UnexpectedToken(char, usize),
}

#[derive(Debug)]
enum ParseErr {
    ExtraToken,
    UnexpectedEnd,
    InvalidToken,
}

fn validate_boolean(
    word: &str,
    chars: &mut std::iter::Peekable<impl Iterator<Item = (usize, char)>>,
) -> bool {
    for expected_char in word.chars() {
        match chars.peek() {
            Some(&(_, actual_char)) if actual_char == expected_char => {
                chars.next(); // consume the character
            }
            _ => return false, // mismatch or end of input
        }
    }
    true
}

fn lexer(input: &str) -> Result<Vec<Token>, JsonErr> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().enumerate().peekable();

    while let Some((i, char)) = chars.next() {
        match char {
            '{' => tokens.push(Token::OpenBrace),
            '}' => tokens.push(Token::CloseBrace),
            ':' => tokens.push(Token::Colon),
            ',' => tokens.push(Token::Comma),
            '[' => tokens.push(Token::OpenBracket),
            ']' => tokens.push(Token::CloseBracket),
            '"' => {
                let mut string = String::new();
                while let Some(&(_, next_char)) = chars.peek() {
                    chars.next();
                    if next_char == '"' {
                        break;
                    }
                    string.push(next_char);
                }
                tokens.push(Token::String(string))
            }
            'f' => {
                if validate_boolean("alse", &mut chars) {
                    tokens.push(Token::Boolean(false));
                } else {
                    return Err(JsonErr::Lexer(LexErr::UnexpectedToken(char, i)));
                }
            }
            't' => {
                if validate_boolean("rue", &mut chars) {
                    tokens.push(Token::Boolean(true));
                } else {
                    return Err(JsonErr::Lexer(LexErr::UnexpectedToken(char, i)));
                }
            }
            'n' => {
                if validate_boolean("ull", &mut chars) {
                    tokens.push(Token::Null);
                } else {
                    return Err(JsonErr::Lexer(LexErr::UnexpectedToken(char, i)));
                }
            }
            c if c.is_numeric() => {
                let mut number_str = c.to_string();
                while let Some(&(_, next_char)) = chars.peek() {
                    if !next_char.is_numeric() {
                        break;
                    } else {
                        chars.next();
                        number_str.push(next_char);
                    }
                }

                let number = number_str.parse::<i64>().unwrap_or(0);
                tokens.push(Token::Number(number))
            }
            c if c.is_whitespace() => continue,
            other => return Err(JsonErr::Lexer(LexErr::UnexpectedToken(other, i))),
        }
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        return self.tokens.get(self.pos);
    }

    fn consume(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, want: &Token) -> Result<(), JsonErr> {
        match self.peek() {
            Some(t) => {
                if discriminant(t) == discriminant(want) {
                    self.consume();
                    Ok(())
                } else {
                    return Err(JsonErr::Parser(ParseErr::InvalidToken));
                }
            }
            None => return Err(JsonErr::Parser(ParseErr::UnexpectedEnd)),
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, JsonErr> {
        match self.peek() {
            Some(Token::OpenBrace) => self.parse_object(),
            Some(Token::OpenBracket) => self.parse_array(),
            Some(Token::String(_)) => {
                if let Token::String(s) = replace(&mut self.tokens[self.pos], Token::Null) {
                    self.consume();
                    Ok(JsonValue::String(s))
                } else {
                    unreachable!()
                }
            }
            Some(Token::Number(_)) => {
                if let Token::Number(n) = replace(&mut self.tokens[self.pos], Token::Null) {
                    self.consume();
                    Ok(JsonValue::Number(n))
                } else {
                    unreachable!()
                }
            }
            Some(Token::Boolean(_)) => {
                if let Token::Boolean(b) = replace(&mut self.tokens[self.pos], Token::Null) {
                    self.consume();
                    Ok(JsonValue::Boolean(b))
                } else {
                    unreachable!()
                }
            }
            Some(Token::Null) => {
                self.consume();
                Ok(JsonValue::Null)
            }
            Some(_) => return Err(JsonErr::Parser(ParseErr::InvalidToken)),
            None => return Err(JsonErr::Parser(ParseErr::UnexpectedEnd)),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, JsonErr> {
        self.expect(&Token::OpenBrace)?;
        let mut map = HashMap::new();

        if let Some(Token::CloseBrace) = self.peek() {
            self.consume();
            return Ok(JsonValue::Object(map));
        }

        loop {
            // key
            let key = match self.peek() {
                Some(Token::String(_)) => {
                    if let Token::String(s) = replace(&mut self.tokens[self.pos], Token::Null) {
                        self.consume();
                        s
                    } else {
                        unreachable!()
                    }
                }
                _ => return Err(JsonErr::Parser(ParseErr::InvalidToken)),
            };

            self.expect(&Token::Colon)?;
            let value = self.parse_value()?;
            map.insert(key, value);

            match self.peek() {
                Some(Token::Comma) => {
                    self.consume();
                    // cannot have ,}
                    if let Some(Token::CloseBracket) = self.peek() {
                        return Err(JsonErr::Parser(ParseErr::InvalidToken));
                    }
                }
                Some(Token::CloseBrace) => {
                    self.consume();
                    break;
                }
                _ => return Err(JsonErr::Parser(ParseErr::InvalidToken)),
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, JsonErr> {
        self.expect(&Token::OpenBracket)?;
        let mut array = Vec::new();

        if let Some(Token::CloseBrace) = self.peek() {
            self.consume();
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);

            match self.peek() {
                Some(Token::Comma) => {
                    self.consume();
                    // we cannot have ,]
                    if let Some(token) = self.peek() {
                        if discriminant(token) == discriminant(&Token::CloseBrace) {
                            return Err(JsonErr::Parser(ParseErr::InvalidToken));
                        }
                    }
                }
                Some(Token::CloseBracket) => {
                    self.consume();
                    break;
                }
                _ => return Err(JsonErr::Parser(ParseErr::InvalidToken)),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse(&mut self) -> Result<JsonValue, JsonErr> {
        // we want the first token to be an open paren
        if let Some(t) = self.peek() {
            if !(discriminant(t) == discriminant(&Token::OpenBrace)) {
                return Err(JsonErr::Parser(ParseErr::InvalidToken));
            }
        }
        let v = self.parse_value()?;
        if self.pos != self.tokens.len() {
            return Err(JsonErr::Parser(ParseErr::ExtraToken));
        } else {
            return Ok(v);
        }
    }
}

fn main() {
    let input = r#"{"hello": [3,2,1, "321312", {"lol":";"}]}"#;
    let tokens: Vec<Token> = lexer(input).unwrap_or(Vec::new());
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(json_value) => {
            println!("Valid JSON: {:?}", json_value);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
