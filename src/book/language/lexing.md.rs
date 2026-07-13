//@ # Lexing
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
use logos::Logos; //#
//@
#[derive(Clone, Debug, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\n\r]+")]
#[logos(skip(r"//[^\n\r]*", allow_greedy = true))]
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
    #[token("let")]
    Let,
    #[token("pub")]
    Pub,
    #[token("crate")]
    Crate,
    #[token("super")]
    Super,
    #[token("as")]
    As,
    #[token("in")]
    In,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("mut")]
    Mut,
    #[token("self")]
    Self_,
    #[token("Self")]
    TraitSelf,
    #[token("bool")]
    Bool,
    #[token("str")]
    Str,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("->")]
    Arrow,
    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("&")]
    Amp,
    #[token("::")]
    PathSep,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
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
    #[token(".")]
    Dot,
    #[token("...")]
    Ellipsis,
    #[token("_")]
    Underscore,
    #[token("$crate")]
    MacroCrate,
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<u128>().unwrap())]
    IntegerLiteral(u128),
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
//@ `if` If;
//@ `else` Else;
//@ `let` Let;
//@ `pub` Pub;
//@ `crate` Crate;
//@ `super` Super;
//@ `as` As;
//@ `in` In;
//@ `mut` Mut;
//@ `self` Self_;
//@ `Self` TraitSelf;
//@ `bool` Bool;
//@ `str` Str;
//@ `true` True;
//@ `false` False;
//@ `->` Arrow;
//@ `=` Eq;
//@ `+` Plus;
//@ `-` Minus;
//@ `*` Star;
//@ `&` Amp;
//@ `::` PathSep;
//@ `<` Lt;
//@ `>` Gt;
//@ `:` Colon;
//@ `,` Comma;
//@ `(` LParen;
//@ `)` RParen;
//@ `{` LBrace;
//@ `}` RBrace;
//@ `;` Semicolon;
//@ `.` Dot;
//@ `...` Ellipsis;
//@ `_` Underscore;
//@ `$crate` MacroCrate;
//@ INTEGER_LITERAL IntegerLiteral(u128);
//@ IDENTIFIER Identifier(String);
//@ STRING_LITERAL StringLiteral(String);
//@ LIFETIME Lifetime(String);
//@ UNSUPPORTED Unsupported;
//@
//@ %precedence `if`;
//@ %precedence `else`;
//@ %precedence `self`;
//@ %precedence `:`;
//@ %precedence `=`;
//@ %precedence `+`;
//@ %precedence `&`;
//@ %precedence `*`;
//@ %precedence `.`;
//@
//@ %allow unit_production_eliminated(Identifier);
//@ %allow unit_production_eliminated(GenericParams);
//@ %allow unit_production_eliminated(WhereClauses);
//@ %allow unit_production_eliminated(OuterAttribute);
//@ %allow unit_production_eliminated(Lifetime);
//@ ```
