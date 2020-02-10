/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for counter properties.

#[cfg(feature = "servo")]
use crate::computed_values::list_style_type::T as ListStyleType;
use crate::parser::{Parse, ParserContext};
use crate::values::generics::counters as generics;
use crate::values::generics::counters::CounterPair;
#[cfg(feature = "gecko")]
use crate::values::generics::CounterStyle;
use crate::values::specified::url::SpecifiedImageUrl;
#[cfg(feature = "gecko")]
use crate::values::specified::Attr;
use crate::values::specified::Integer;
use crate::values::CustomIdent;
use cssparser::{Parser, Token};
use selectors::parser::SelectorParseErrorKind;
use style_traits::{ParseError, StyleParseErrorKind};

/// A specified value for the `counter-increment` property.
pub type CounterIncrement = generics::GenericCounterIncrement<Integer>;

impl Parse for CounterIncrement {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(Self::new(parse_counters(context, input, 1)?))
    }
}

/// A specified value for the `counter-set` and `counter-reset` properties.
pub type CounterSetOrReset = generics::GenericCounterSetOrReset<Integer>;

impl Parse for CounterSetOrReset {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(Self::new(parse_counters(context, input, 0)?))
    }
}

fn parse_counters<'i, 't>(
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
    default_value: i32,
) -> Result<Vec<CounterPair<Integer>>, ParseError<'i>> {
    if input
        .try(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(vec![]);
    }

    let mut counters = Vec::new();
    loop {
        let location = input.current_source_location();
        let name = match input.next() {
            Ok(&Token::Ident(ref ident)) => CustomIdent::from_ident(location, ident, &["none"])?,
            Ok(t) => {
                let t = t.clone();
                return Err(location.new_unexpected_token_error(t));
            },
            Err(_) => break,
        };

        let value = input
            .try(|input| Integer::parse(context, input))
            .unwrap_or(Integer::new(default_value));
        counters.push(CounterPair { name, value });
    }

    if !counters.is_empty() {
        Ok(counters)
    } else {
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

/// The specified value for the `content` property.
pub type Content = generics::GenericContent<SpecifiedImageUrl>;

/// The specified value for a content item in the `content` property.
pub type ContentItem = generics::GenericContentItem<SpecifiedImageUrl>;

impl Content {
    #[cfg(feature = "servo")]
    fn parse_counter_style(_: &ParserContext, input: &mut Parser) -> ListStyleType {
        input
            .try(|input| {
                input.expect_comma()?;
                ListStyleType::parse(input)
            })
            .unwrap_or(ListStyleType::Decimal)
    }

    #[cfg(feature = "gecko")]
    fn parse_counter_style(context: &ParserContext, input: &mut Parser) -> CounterStyle {
        input
            .try(|input| {
                input.expect_comma()?;
                CounterStyle::parse(context, input)
            })
            .unwrap_or(CounterStyle::decimal())
    }
}

impl Parse for Content {
    // normal | none | [ <string> | <counter> | open-quote | close-quote | no-open-quote |
    // no-close-quote ]+
    // TODO: <uri>, attr(<identifier>)
    #[cfg_attr(feature = "servo", allow(unused_mut))]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input
            .try(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(generics::Content::Normal);
        }
        if input
            .try(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(generics::Content::None);
        }

        let mut content = vec![];
        let mut has_alt_content = false;
        loop {
            #[cfg(feature = "gecko")]
            {
                if let Ok(url) = input.try(|i| SpecifiedImageUrl::parse(context, i)) {
                    content.push(generics::ContentItem::Url(url));
                    continue;
                }
            }
            match input.next() {
                Ok(&Token::QuotedString(ref value)) => {
                    content.push(generics::ContentItem::String(
                        value.as_ref().to_owned().into(),
                    ));
                },
                Ok(&Token::Function(ref name)) => {
                    let result = match_ignore_ascii_case! { &name,
                        "counter" => input.parse_nested_block(|input| {
                            let location = input.current_source_location();
                            let name = CustomIdent::from_ident(location, input.expect_ident()?, &[])?;
                            let style = Content::parse_counter_style(context, input);
                            Ok(generics::ContentItem::Counter(name, style))
                        }),
                        "counters" => input.parse_nested_block(|input| {
                            let location = input.current_source_location();
                            let name = CustomIdent::from_ident(location, input.expect_ident()?, &[])?;
                            input.expect_comma()?;
                            let separator = input.expect_string()?.as_ref().to_owned().into();
                            let style = Content::parse_counter_style(context, input);
                            Ok(generics::ContentItem::Counters(name, separator, style))
                        }),
                        #[cfg(feature = "gecko")]
                        "attr" => input.parse_nested_block(|input| {
                            Ok(generics::ContentItem::Attr(Attr::parse_function(context, input)?))
                        }),
                        _ => {
                            let name = name.clone();
                            return Err(input.new_custom_error(
                                StyleParseErrorKind::UnexpectedFunction(name),
                            ))
                        }
                    }?;
                    content.push(result);
                },
                Ok(&Token::Ident(ref ident)) => {
                    content.push(match_ignore_ascii_case! { &ident,
                        "open-quote" => generics::ContentItem::OpenQuote,
                        "close-quote" => generics::ContentItem::CloseQuote,
                        "no-open-quote" => generics::ContentItem::NoOpenQuote,
                        "no-close-quote" => generics::ContentItem::NoCloseQuote,
                        #[cfg(feature = "gecko")]
                        "-moz-alt-content" => {
                            has_alt_content = true;
                            generics::ContentItem::MozAltContent
                        },
                        _ =>{
                            let ident = ident.clone();
                            return Err(input.new_custom_error(
                                SelectorParseErrorKind::UnexpectedIdent(ident)
                            ));
                        }
                    });
                },
                Err(_) => break,
                Ok(t) => {
                    let t = t.clone();
                    return Err(input.new_unexpected_token_error(t));
                },
            }
        }
        // We don't allow to parse `-moz-alt-content in multiple positions.
        if content.is_empty() || (has_alt_content && content.len() != 1) {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(generics::Content::Items(content.into()))
    }
}
