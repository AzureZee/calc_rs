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
impl Lexer {
    fn new(input: &str) -> Self {
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
                PLUS | MUL | DIV => {
                    tokens.push(Token::Op(this));
                }
                MINUS => {
                    if let Some(back) = chars.get(i - 2)
                        && (back.is_ascii_digit() || is_letter(*back) || *back == RP)
                    {
                        tokens.push(Token::Op(this));
                    } else {
                        buf.push(this);
                        while i < chars.len()
                            && let next = chars[i]
                            && (next.is_ascii_digit() || next == '.')
                        {
                            buf.push(next);
                            i += 1;
                        }
                        let num = buf.clone().parse::<Num>().unwrap();
                        tokens.push(Token::Number(num));
                    }
                }
                this if this.is_ascii_digit() => {
                    buf.push(this);

                    while i < chars.len()
                        && let next = chars[i]
                        && (next.is_ascii_digit() || next == '.')
                    {
                        buf.push(next);
                        i += 1;
                    }
                    let num = buf.clone().parse::<Num>().unwrap();
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
                _ => {}
            }
        }
        tokens.reverse();
        Self { tokens }
    }

    fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::Eof)
    }
    // fn peek(&mut self) -> Token {
    //     self.tokens.last().copied().unwrap_or(Token::Eof)
    // }
}

fn is_letter(this: char) -> bool {
    this.is_ascii_alphabetic() || this == '_'
}
#[test]
fn lexer_output() {
    let input = "((-1.0 + -22) * _a1_3) - bc2 / c";
    println!("{:?}", Lexer::new(input).tokens);
}

fn main() {
    println!("type `exit` quit");

    loop {
        print!(">>> ");
        stdout().flush().unwrap();
        let buf = &mut String::new();
        stdin().read_line(buf).unwrap();
        if buf.trim() == "exit" {
            break;
        }
        let tokens = Lexer::new(&buf);
        let output = infix_to_rpn(tokens);
        println!("{:?}", eval_expr(output));
    }
}
// let input = "(1+11)/((-3-5)*4)";

fn eval_expr(output: Vec<Token>) -> Num {
    let mut eval_stack = vec![];
    let mut iter = output.into_iter();
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
                    let value = eval(dbg!(op), dbg!(lhs), dbg!(rhs));
                    dbg!(&eval_stack);
                    eval_stack.push(value);
                }
            }
            _ => {}
        }
    }
    dbg!(eval_stack[0])
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
fn eval(op: char, lhs: Num, rhs: Num) -> Num {
    match op {
        PLUS => lhs + rhs,
        MINUS => lhs - rhs,
        MUL => lhs * rhs,
        DIV => lhs / rhs,
        _ => 0.0,
    }
}
