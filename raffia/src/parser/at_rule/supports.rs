use super::Parser;
use crate::{
    ast::*,
    error::{Error, ErrorKind, PResult},
    expect,
    pos::{Span, Spanned},
    tokenizer::Token,
    Parse,
};

// https://drafts.csswg.org/css-conditional-3/#at-supports
impl<'cmt, 's: 'cmt> Parse<'cmt, 's> for SupportsCondition<'s> {
    fn parse(input: &mut Parser<'cmt, 's>) -> PResult<Self> {
        match input.tokenizer.peek()? {
            Token::Ident(token) if token.name.eq_ignore_ascii_case("not") => {
                let keyword = input.parse::<Ident>()?;
                let condition = input.parse::<SupportsInParens>()?;
                let span = Span {
                    start: keyword.span.start,
                    end: condition.span().end,
                };
                Ok(SupportsCondition {
                    conditions: vec![SupportsConditionKind::Not(SupportsNot {
                        keyword,
                        condition,
                        span: span.clone(),
                    })],
                    span,
                })
            }
            _ => {
                let first = input.parse::<SupportsInParens>()?;
                let mut span = first.span().clone();
                let mut conditions = vec![SupportsConditionKind::SupportsInParens(first)];
                loop {
                    match input.tokenizer.peek()? {
                        Token::Ident(token) if token.name.eq_ignore_ascii_case("and") => {
                            let ident = input.parse::<Ident>()?;
                            let condition = input.parse::<SupportsInParens>()?;
                            let span = Span {
                                start: ident.span.start,
                                end: condition.span().end,
                            };
                            conditions.push(SupportsConditionKind::And(SupportsAnd {
                                keyword: ident,
                                condition,
                                span,
                            }));
                        }
                        Token::Ident(token) if token.name.eq_ignore_ascii_case("or") => {
                            let ident = input.parse::<Ident>()?;
                            let condition = input.parse::<SupportsInParens>()?;
                            let span = Span {
                                start: ident.span.start,
                                end: condition.span().end,
                            };
                            conditions.push(SupportsConditionKind::Or(SupportsOr {
                                keyword: ident,
                                condition,
                                span,
                            }));
                        }
                        _ => break,
                    }
                }
                if let Some(last) = conditions.last() {
                    span.end = last.span().end;
                }
                Ok(SupportsCondition { conditions, span })
            }
        }
    }
}

impl<'cmt, 's: 'cmt> Parse<'cmt, 's> for SupportsInParens<'s> {
    fn parse(input: &mut Parser<'cmt, 's>) -> PResult<Self> {
        match input.tokenizer.peek()? {
            Token::LParen(..) => input
                .try_parse(|parser| {
                    parser
                        .parse()
                        .map(|supports_decl| SupportsInParens::Feature(Box::new(supports_decl)))
                })
                .or_else(|_| {
                    expect!(input, LParen);
                    let condition = input.parse()?;
                    expect!(input, RParen);
                    Ok(SupportsInParens::SupportsCondition(condition))
                }),
            token => Err(Error {
                kind: ErrorKind::Unexpected("'('", token.symbol()),
                span: token.span().clone(),
            }),
        }
    }
}

impl<'cmt, 's: 'cmt> Parse<'cmt, 's> for SupportsDecl<'s> {
    fn parse(input: &mut Parser<'cmt, 's>) -> PResult<Self> {
        let l_paren = expect!(input, LParen);
        let decl = input.parse()?;
        let r_paren = expect!(input, RParen);
        Ok(SupportsDecl {
            decl,
            span: Span {
                start: l_paren.span.start,
                end: r_paren.span.end,
            },
        })
    }
}
