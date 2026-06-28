//@ # Lexing
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
use logos::Logos; //#
//@
#[derive(Clone, Debug, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\n\r]+")]
pub enum Token {
    #[token("fn")]
    Fn,
    #[token("const")]
    Const,
    #[token("async")]
    Async,
    #[token("safe")]
    Safe,
    #[token("unsafe")]
    Unsafe,
    #[token("extern")]
    Extern,
    #[token("mut")]
    Mut,
    #[token("self")]
    Self_,
    #[token("bool")]
    Bool,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("->")]
    Arrow,
    #[token("&")]
    Amp,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(";")]
    Semicolon,
    #[token("...")]
    Ellipsis,
    #[token("_")]
    Underscore,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_owned(), priority = 1)]
    Identifier(String),
    #[regex(r#""[^"]*""#, string_literal)]
    StringLiteral(String),
    #[regex(r"'[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_owned())]
    Lifetime(String),
    #[token("__unsupported__")]
    Unsupported,
}

fn string_literal(lex: &mut logos::Lexer<'_, Token>) -> String {
    let slice = lex.slice();
    slice[1..slice.len() - 1].to_owned()
}

//@ ```rustylr declarations
//@ %tokentype Token;
//@ %start Program;
//@
//@ %token fn_ Token::Fn;
//@ %token const_ Token::Const;
//@ %token async_ Token::Async;
//@ %token safe Token::Safe;
//@ %token unsafe_ Token::Unsafe;
//@ %token extern_ Token::Extern;
//@ %token mut_ Token::Mut;
//@ %token self_ Token::Self_;
//@ %token bool_ Token::Bool;
//@ %token true_ Token::True;
//@ %token false_ Token::False;
//@ %token arrow Token::Arrow;
//@ %token amp Token::Amp;
//@ %token colon Token::Colon;
//@ %token comma Token::Comma;
//@ %token lparen Token::LParen;
//@ %token rparen Token::RParen;
//@ %token lbrace Token::LBrace;
//@ %token rbrace Token::RBrace;
//@ %token semicolon Token::Semicolon;
//@ %token ellipsis Token::Ellipsis;
//@ %token underscore Token::Underscore;
//@ %token identifier Token::Identifier(_);
//@ %token string_literal Token::StringLiteral(_);
//@ %token lifetime Token::Lifetime(_);
//@ %token unsupported Token::Unsupported;
//@
//@ %precedence self_;
//@ %precedence colon;
//@
//@ %allow unit_production_eliminated(Identifier);
//@ %allow unit_production_eliminated(GenericParams);
//@ %allow unit_production_eliminated(WhereClause);
//@ %allow unit_production_eliminated(OuterAttribute);
//@ %allow unit_production_eliminated(Lifetime);
//@ ```
