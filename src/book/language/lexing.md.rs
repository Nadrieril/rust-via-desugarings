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
    #[token("Self")]
    TraitSelf,
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

//@ ```lexer
//@ %tokentype Token;
//@ %start Program;
//@
//@ `fn` Fn;
//@ `const` Const;
//@ `async` Async;
//@ `safe` Safe;
//@ `unsafe` Unsafe;
//@ `extern` Extern;
//@ `mut` Mut;
//@ `self` Self_;
//@ `Self` TraitSelf;
//@ `bool` Bool;
//@ `true` True;
//@ `false` False;
//@ `->` Arrow;
//@ `&` Amp;
//@ `:` Colon;
//@ `,` Comma;
//@ `(` LParen;
//@ `)` RParen;
//@ `{` LBrace;
//@ `}` RBrace;
//@ `;` Semicolon;
//@ `...` Ellipsis;
//@ `_` Underscore;
//@ IDENTIFIER Identifier(String);
//@ STRING_LITERAL StringLiteral(String);
//@ LIFETIME Lifetime(String);
//@ UNSUPPORTED Unsupported;
//@
//@ %precedence `self`;
//@ %precedence `:`;
//@
//@ %allow unit_production_eliminated(Identifier);
//@ %allow unit_production_eliminated(GenericParams);
//@ %allow unit_production_eliminated(WhereClauses);
//@ %allow unit_production_eliminated(OuterAttribute);
//@ %allow unit_production_eliminated(Lifetime);
//@ ```
