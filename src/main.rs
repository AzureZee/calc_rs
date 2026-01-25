use std::{
    io::{BufRead, Write, stdin, stdout},
    ops::Not,
};
type Num = f32;
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Variable(String),
    Number(Num),
    Op(char),
    LParen,
    RParen,
    Eof,
}

#[derive(Debug)]
struct Lexer {
    tokens: Vec<Token>,
    // chars: Vec<char>,
}
const LP: char = '(';
const RP: char = ')';
const PLUS: char = '+';
const MINUS: char = '-';
const MUL: char = '*';
const DIV: char = '/';
type LexerResult = Result<Lexer, String>;
impl Lexer {
    fn new(input: &str) -> LexerResult {
        let chars = input
            .chars()
            .filter(|input_char| input_char.is_ascii_whitespace().not())
            .collect::<Vec<_>>();
        let mut tokens = vec![];
        let mut i = 0;
        while i < chars.len() {
            let mut buf = String::new();
            let this = chars[i];
            i += 1;
            match this {
                LP => {
                    tokens.push(Token::LParen);
                }
                RP => {
                    tokens.push(Token::RParen);
                }
                MUL | DIV => {
                    tokens.push(Token::Op(this));
                }
                PLUS => {
                    if i > 1
                        && let Some(back) = chars.get(i - 2)
                        && (back.is_ascii_digit() || is_letter(*back) || *back == RP)
                    {
                        tokens.push(Token::Op(this));
                    } else {
                        buf.push(this);
                        let num = read_number(&chars, &mut i, &mut buf)?;
                        tokens.push(Token::Number(num));
                    }
                }
                MINUS => {
                    if i > 1
                        && let Some(back) = chars.get(i - 2)
                        && (back.is_ascii_digit() || is_letter(*back) || *back == RP)
                    {
                        tokens.push(Token::Op(this));
                    } else {
                        buf.push(this);
                        let num = read_number(&chars, &mut i, &mut buf)?;
                        tokens.push(Token::Number(num));
                    }
                }
                this if this.is_ascii_digit() => {
                    buf.push(this);
                    let num = read_number(&chars, &mut i, &mut buf)?;
                    tokens.push(Token::Number(num));
                }
                this if is_letter(this) => {
                    buf.push(this);

                    while i < chars.len()
                        && let next = chars[i]
                        && (is_letter(next) || next.is_ascii_digit())
                    {
                        buf.push(next);
                        i += 1;
                    }
                    tokens.push(Token::Variable(buf.clone()));
                }
                other @ _ => {
                    return Err(format!("invalid char: {other}"));
                }
            }
        }
        tokens.reverse();
        Ok(Self { tokens })
    }

    fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::Eof)
    }
    // fn peek(&mut self) -> Token {
    //     self.tokens.last().copied().unwrap_or(Token::Eof)
    // }
}
type PraseResult = Result<Num, String>;
fn read_number(chars: &Vec<char>, i: &mut usize, buf: &mut String) -> PraseResult {
    fn stringify(buf: &str) -> String {
        format!("invalid number: {buf}")
    }
    while *i < chars.len()
        && let next = chars[*i]
        && (next.is_ascii_digit() || next == '.')
    {
        buf.push(next);
        *i += 1;
    }
    buf.clone().parse::<Num>().map_err(|_| stringify(&buf))
}

fn is_letter(this: char) -> bool {
    this.is_ascii_alphabetic() || this == '_'
}
#[test]
fn lexer_output() {
    let input = "--5*((-1.0 + -22) * _a1_3) - bc2 / c";
    println!("{:?}", Lexer::new(input));
}

fn main() {
    println!("Enter `bye` exit");

    loop {
        print!(">>> ");
        stdout().flush().unwrap();
        let buf = &mut String::new();
        stdin().read_line(buf).unwrap();
        if buf.trim() == "bye" {
            break;
        }
        match Lexer::new(&buf) {
            Ok(tokens) => {
                let output = infix_to_rpn(tokens);
                match eval_expr(output) {
                    Ok(res) => {
                        println!("{:?}", res);
                    }
                    Err(err) => {
                        eprintln!("{:?}", err);
                    }
                };
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        };
    }
}
// let input = "(1+11)/((-3-5)*4)";
type EvalResult = Result<Num, String>;
fn eval_expr(output: Vec<Token>) -> EvalResult {
    let mut eval_stack = vec![];
    let mut iter = output.into_iter();
    let err_info = "invalid expr".to_string();
    while let Some(ele) = iter.next() {
        match ele {
            Token::Number(n) => {
                eval_stack.push(n);
            }
            Token::Op(op) => {
                dbg!(&eval_stack);
                if let Some(rhs) = eval_stack.pop()
                    && let Some(lhs) = eval_stack.pop()
                {
                    let value = eval(dbg!(lhs), dbg!(op), dbg!(rhs));
                    dbg!(&eval_stack);
                    eval_stack.push(value);
                } else {
                    return Err(err_info);
                }
            }
            _ => {}
        }
    }
    match eval_stack.get(0) {
        Some(res) => Ok(*res),
        None => Err(err_info),
    }
}
fn eval(lhs: Num, op: char, rhs: Num) -> Num {
    match op {
        PLUS => lhs + rhs,
        MINUS => lhs - rhs,
        MUL => lhs * rhs,
        DIV => lhs / rhs,
        _ => 0.0,
    }
}
fn infix_to_rpn(mut tokens: Lexer) -> Vec<Token> {
    let mut op_stack = vec![];
    let mut output = vec![];
    loop {
        match tokens.next() {
            n @ Token::Number(_) => {
                output.push(n);
            }
            lp @ Token::LParen => {
                op_stack.push(lp);
            }
            Token::RParen => {
                while let Some(op) = op_stack.pop()
                    && op != Token::LParen
                {
                    output.push(op);
                }
            }
            Token::Op(incoming_op) => {
                if let Some(Token::Op(top_op)) = op_stack.last()
                    && OpInfo::new(*top_op).should_pop(OpInfo::new(incoming_op))
                {
                    let tmp = op_stack.pop().unwrap();
                    op_stack.push(Token::Op(incoming_op));
                    output.push(tmp);
                } else {
                    op_stack.push(Token::Op(incoming_op));
                }
            }
            Token::Eof => {
                op_stack.reverse();
                output.extend(op_stack);
                break;
            }
            _ => {}
        }
    }
    println!("{:?}", output);
    output
}

#[derive(Default, PartialEq)]
enum Associativity {
    #[default]
    Left,
    Right,
}

#[derive(Default)]
struct OpInfo {
    precedence: u32,
    associativity: Associativity,
}

impl OpInfo {
    fn new(op: char) -> Self {
        match op {
            PLUS | MINUS => Self {
                precedence: 10,
                associativity: Associativity::Left,
            },
            MUL | DIV => Self {
                precedence: 20,
                associativity: Associativity::Left,
            },
            _ => Self::default(),
        }
    }
    fn should_pop(&self, other: OpInfo) -> bool {
        self.precedence > other.precedence
            || (self.precedence == other.precedence && self.associativity == Associativity::Left)
    }
}
