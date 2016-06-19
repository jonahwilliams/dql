/*
    main hit damage - range /accuracy?
    multiplied by crit  multiple with a %chance
    (110 - 120) * 95  + 0 * 5
    [100, 0, 95]*[2.5, 1, 22]*2

    TODO: fix the grammar ... and clean all this up ... so much

    -- composed of splits
    program     = command expression
    expression  = split [("*"|"+") expression].
    split       = "[" simpleValue, simpleValue, number "]".
    simpleValue = number | range
    number      = (-) digit* ("." digit*).
    range       = "(" number, number ")".
    command     = "RANGE" | "AVERAGE" | "MIN" | "MAX".

*/
fn main() {
    let program = "AVERAGE (1,100)";
    let mut parser = Parser::new(program);
    let prog = parser.parse();
    println!("{:?}", prog);
    println!("{}", prog.run());
}

const RESERVED_SYMBOLS: [char; 8] = ['+', '*','[', ']', ';','(',')', ','];
const RESERVED_PHRASES: [&'static str; 4] = ["AVERAGE", "MIN", "MAX", "QUERY"];

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Plus,
    Times,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Semicolon,
    Command(CommandType),
    Number(f64)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandType {
    Average,
    Min,
    Max,
    Query
}


////// LEXER /////////

struct Lexer {
    raw: Vec<char>
}

impl Lexer {
    fn new(program: &str) -> Lexer {
        Lexer { raw: program.chars().collect() }
    }
    fn collect(&self) -> Vec<Token> {
        let mut result: Vec<Token> = Vec::new();
        let mut i = 0;
        let mut acc = String::new();
        while i < self.raw.len() {
            let ch = self.raw[i];
            match ch {
                ' '|',' => {
                    i += 1;
                    continue;
                },
                '+' => {
                    result.push(Token::Plus);
                    i += 1;
                    continue;
                },
                '*' => {
                    result.push(Token::Times);
                    i += 1;
                    continue;
                },
                '[' => {
                    result.push(Token::LBracket);
                    i += 1;
                    continue;
                },
                ']' => {
                    result.push(Token::RBracket);
                    i += 1;
                    continue;
                },
                '(' => {
                    result.push(Token::LParen);
                    i += 1;
                    continue;
                },
                ')' => {
                    result.push(Token::RParen);
                    i += 1;
                    continue;
                },
                ';' => {
                    result.push(Token::Semicolon);
                    i += 1;
                    continue;
                },
                _ => ()
            };
            let mut acc = String::new();
            acc.push(ch);
            let mut next_ch = self.raw[i + 1];
            while !RESERVED_SYMBOLS.contains(&next_ch) && next_ch != ' ' {
                acc.push(next_ch);
                i += 1;
                next_ch = self.raw[i + 1];
            }
            match acc.as_ref() {
                "AVERAGE" => {
                    result.push(Token::Command(CommandType::Average));
                    i += 1;
                    continue;
                },
                "MIN" => {
                    result.push(Token::Command(CommandType::Min));
                    i += 1;
                    continue;
                },
                "MAX" => {
                    result.push(Token::Command(CommandType::Max));
                    i += 1;
                    continue;
                },
                "QUERY" => {
                    result.push(Token::Command(CommandType::Query));
                    i += 1;
                    continue;
                },
                _ => ()
            };
            // otherwise its a number
            result.push(Token::Number(acc.parse::<f64>().unwrap()));
            i += 1;
        }
        return result;
    }
}

///// PARSING ///////

struct Parser {
    tokens: std::vec::IntoIter<Token>
}

impl Parser {
    fn new(program: &str) -> Parser {
        let tokens = Lexer::new(program).collect();
        for i in  0..tokens.len() {
            println!("{:?}", tokens[i]);
        }
        Parser { tokens: tokens.into_iter() }
    }
    fn parse(&mut self) -> Program {
        // special program type grabboid
        self.tokens.next().unwrap();
        Program { query: Query::Average, start: self.expression() }
    }
    fn expression(&mut self) -> ProgramNode {
        let left = match self.tokens.next().unwrap() {
            Token::LParen => self.range(),
            Token::LBracket => self.split(),
            Token::Number(val) => {
                ProgramNode::new(
                    None,
                    None,
                    NodeKind::Value(val))
            }
            t => panic!("Error: Invalid start of expression {:?}", t)
        };
        match self.tokens.next().unwrap() {
            Token::Semicolon => {
                ProgramNode::new(
                    Some(left),
                    None,
                    NodeKind::Expression(Operation::Add))
            },
            Token::Plus => {
                ProgramNode::new(
                    Some(left),
                    Some(self.expression()),
                    NodeKind::Expression(Operation::Add))
            },
            Token::Times => {
                ProgramNode::new(
                    Some(left),
                    Some(self.expression()),
                    NodeKind::Expression(Operation::Multiply))
            },
            _ => { panic!("Error on expression"); }
        }
    }
    fn split(&mut self) -> ProgramNode {
        let left = match self.tokens.next().unwrap() {
            Token::LParen => self.range(),
            Token::LBracket => self.split(),
            Token::Number(val)  => {
                ProgramNode::new(
                    None,
                    None,
                    NodeKind::Value(val))
            },
            c => panic!("Error: Invalid token {:?} in split parse", c)
        };
        let right = match self.tokens.next().unwrap() {
            Token::LParen => self.range(),
            Token::LBracket => self.split(),
            Token::Number(val)  => {
                ProgramNode::new(
                    None,
                    None,
                    NodeKind::Value(val))
            },
            _ => panic!("Error on 144")
        };
        let percent = match self.tokens.next().unwrap() {
            Token::Number(val) => val,
            _ => panic!("Error on 148")
        };
        // percentage is valid range
        assert!(percent <= 100.0 && percent >= 0.0,
            "Error: percentage must be between 0.0 and 100.0");

        // last bracket match
        assert!(self.tokens.next().unwrap() == Token::RBracket,
            "Error: Split must terminate with right bracket");
        ProgramNode::new(
            Some(left),
            Some(right),
            NodeKind::Split(percent))
    }
    fn range(&mut self) -> ProgramNode {
        let left = match self.tokens.next().unwrap() {
            Token::Number(val) => {
                ProgramNode::new(None, None, NodeKind::Value(val))
            },
            _ => panic!("Missing first value for range node")
        };
        let right = match self.tokens.next().unwrap() {
            Token::Number(val) => {
                ProgramNode::new(None, None, NodeKind::Value(val))
            },
            _ => panic!("Missing second value for range node")
        };
        assert!(self.tokens.next().unwrap() == Token::RParen,
            "Error: Range must terminate in right paren");
        ProgramNode::new(Some(left), Some(right), NodeKind::Range)
    }
}


//////////// Representation //////////

#[derive(Debug)]
struct Program {
    query: Query,
    start: ProgramNode,
}

impl Program {
    fn run(self) -> f64 {
        self.start.eval(self.query)
    }
}

#[derive(Debug)]
struct ProgramNode {
    kind: NodeKind,
    left: Box<Option<ProgramNode>>,
    right: Box<Option<ProgramNode>>
}

impl ProgramNode {
    fn new(
        left: Option<ProgramNode>,
        right: Option<ProgramNode>,
        kind: NodeKind) -> ProgramNode {
            ProgramNode {
                left: Box::new(left),
                right: Box::new(right),
                kind: kind
            }
        }
}

impl Node for ProgramNode {
    fn eval(self, q: Query) -> f64 {
        let k = self.kind.clone();
        match k {
            NodeKind::Split(percent) => {
                let l = &self.left.unwrap().eval(q);
                let r = &self.right.unwrap().eval(q);
                *l * (percent / 100.0) + *r * (1.0 - percent / 100.0)
            },
            NodeKind::Range => {
                let l = &self.left.unwrap().eval(q);
                let r = &self.right.unwrap().eval(q);
                (*l + *r) / 2.0
            },
            NodeKind::Value(fixed) => {
                fixed
            },
            NodeKind::Expression(op) => {
                let l = &self.left.unwrap().eval(q);
                let mut result = *l;
                let r = &self.right
                    .unwrap_or(ProgramNode::new(None, None, NodeKind::Value(0.0)))
                    .eval(q);
                if op == Operation::Add {
                    *l + *r
                } else {
                    *l * *r
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum NodeKind {
    Split(f64),
    Range,
    Value(f64),
    Expression(Operation)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Operation {
    Add,
    Multiply,
}

#[derive(Clone, Copy, Debug)]
enum Query {
    Average,
}

trait Node {
    fn eval(self, q: Query) -> f64;
}
