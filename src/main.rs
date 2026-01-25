use std::{
    io::{Write, stdin, stdout},
    ops::Not,
};
type Num = f32;
type LexerResult = Result<Lexer, String>;
type PraseResult = Result<Num, String>;
type EvalResult = Result<Num, String>;
const LP: char = '(';
const RP: char = ')';
const PLUS: char = '+';
const MINUS: char = '-';
const MUL: char = '*';
const DIV: char = '/';
const POW: char = '^';

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
        match Lexer::scan(&buf) {
            Ok(mut infix_expr) => {
                let rpn_expr = infix_expr.to_rpn();
                match eval(rpn_expr) {
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

#[derive(Debug)]
struct Lexer {
    tokens: Vec<Token>,
}
impl Lexer {
    fn scan(input: &str) -> LexerResult {
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
                MUL | DIV | POW => {
                    tokens.push(Token::Op(this));
                }

                PLUS | MINUS => {
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

    fn to_rpn(&mut self) -> Vec<Token> {
        let mut op_stack = vec![];
        // Reverse Polish Notation (RPN)
        let mut rpn = vec![];
        loop {
            match self.next() {
                num @ Token::Number(_) => {
                    rpn.push(num);
                }
                lp @ Token::LParen => {
                    op_stack.push(lp);
                }
                Token::RParen => {
                    while let Some(op) = op_stack.pop()
                        && op != Token::LParen
                    {
                        rpn.push(op);
                    }
                }
                Token::Op(incoming_op) => {
                    if let Some(Token::Op(top_op)) = op_stack.last()
                        && OpInfo::of(*top_op).should_pop(OpInfo::of(incoming_op))
                    {
                        let tmp = op_stack.pop().unwrap();
                        op_stack.push(Token::Op(incoming_op));
                        rpn.push(tmp);
                    } else {
                        op_stack.push(Token::Op(incoming_op));
                    }
                }
                Token::Eof => {
                    op_stack.reverse();
                    rpn.extend(op_stack);
                    break;
                }
                _ => {}
            }
        }
        println!("{:?}", rpn);
        rpn
    }
    fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::Eof)
    }
    // fn peek(&mut self) -> Token {
    //     self.tokens.last().copied().unwrap_or(Token::Eof)
    // }
}
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Variable(String),
    Number(Num),
    Op(char),
    LParen,
    RParen,
    Eof,
}
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

fn eval(rpn_expr: Vec<Token>) -> EvalResult {
    let mut eval_stack = vec![];
    let mut expr = rpn_expr.into_iter();
    let err_info = "invalid expr".to_string();
    while let Some(ele) = expr.next() {
        match ele {
            Token::Number(n) => {
                eval_stack.push(n);
            }
            Token::Op(op) => {
                dbg!(&eval_stack);
                if let Some(rhs) = eval_stack.pop()
                    && let Some(lhs) = eval_stack.pop()
                {
                    dbg!(&eval_stack);
                    let value = eval_expr(dbg!(lhs), dbg!(op), dbg!(rhs));
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
fn eval_expr(lhs: Num, op: char, rhs: Num) -> Num {
    match op {
        PLUS => lhs + rhs,
        MINUS => lhs - rhs,
        MUL => lhs * rhs,
        DIV => lhs / rhs,
        POW => lhs.powf(rhs),
        _ => 0.0,
    }
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
    fn of(op: char) -> Self {
        match op {
            PLUS | MINUS => Self {
                precedence: 10,
                associativity: Associativity::Left,
            },
            MUL | DIV => Self {
                precedence: 20,
                associativity: Associativity::Left,
            },
            POW => Self {
                precedence: 30,
                associativity: Associativity::Right,
            },
            _ => Self::default(),
        }
    }
    fn should_pop(&self, other: OpInfo) -> bool {
        self.precedence > other.precedence
            || (self.precedence == other.precedence && self.associativity == Associativity::Left)
    }
}

#[test]
fn lexer_output() {
    let input = "--5*((-1.0 + -22) * _a1_3) - bc2 / c";
    println!("{:?}", Lexer::scan(input));
}
