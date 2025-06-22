// use nom::
// {
//     branch::alt, bytes::complete::{is_a, tag}, combinator::opt, error::ParseError, sequence::{delimited, pair, separated_pair}, IResult, Parser
// };
// use crate::{error::{ParserError}, objects::number::{Number, VerticalAlignment}, parsers::consts::{SUBSCRIPT, SUPERSCRIPT}};

// use super::ITEM_NUMBER;

// ///один из номеров в списке вносимых изменений, пока его использовать не будем
// /// 1) | 1-1) | 1^1 | 1_1 | а) итд
// pub fn parse_number(s: &str) -> IResult<&str, Number, ParserError>
// {
//     let mut normal_parser =  
//     pair(
//     is_a(ITEM_NUMBER), 
//     opt(alt((tag(")"), tag("."))))
//     );
//     if let Ok((remains, (first_number_part , second_number_part))) = is_superscript_number(s)
//     {
//         let (remains, postfix) = opt(alt((tag(")"), tag(".")))).parse(remains)?;
//         return Ok((remains, Number
//         {
//             number: first_number_part.to_owned(),
//             va_number: Some((second_number_part.to_owned(), VerticalAlignment::Superscript)),
//             postfix: postfix.and_then(|a| Some(a.to_owned()))
//         }))
//     }
//     if let Ok((remains, (first_number_part , second_number_part))) = is_subscript_number(s)
//     {
//         let (remains, postfix) = opt(alt((tag(")"), tag(".")))).parse(remains)?;
//         return Ok((remains, Number
//         {
//             number: first_number_part.to_owned(),
//             va_number: Some((second_number_part.to_owned(), VerticalAlignment::Subscript)),
//             postfix: postfix.and_then(|a| Some(a.to_owned()))
//         }))
//     }

//     let (remains, (number, postfix)) = 
//     normal_parser.parse(s)?;
//     Ok((remains, Number
//     {
//         number: number.to_owned(),
//         va_number: None,
//         postfix: postfix.and_then(|a| Some(a.to_owned()))
//     })
//     )
// }

// fn is_subscript_number(s: &str) -> IResult<&str, (&str,&str)>
// { 
//     let res = (
//         is_a(ITEM_NUMBER),
//         is_a(SUBSCRIPT),
//         is_a(ITEM_NUMBER),
//     ).parse(s)?;
//     Ok((res.0, (res.1.0, res.1.2)))
// }
// fn is_superscript_number(s: &str) -> IResult<&str, (&str,&str)>
// { 
//     let res = (
//         is_a(ITEM_NUMBER),
//         is_a(SUPERSCRIPT),
//         is_a(ITEM_NUMBER),
//     ).parse(s)?;
//     Ok((res.0, (res.1.0, res.1.2)))
// }
// #[cfg(test)]
// mod tests
// {
//     #[test]
//     fn test_numbers()
//     {
//         let item = "1) бла бла бла....";
//         let parsed = super::parse_number(item).unwrap();
//         assert_eq!(parsed.0, " бла бла бла....");
//     }
// }