use std::{cmp::Ordering, str::FromStr};
use nom::{branch::alt, bytes::complete::tag_no_case, IResult, Parser};
use serde::{Deserialize, Serialize};

use crate::error::ParserError;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub enum HeaderType
{
    ///раздел
    Chapter,
    ///глава
    Section,
    ///статья
    Article
}

impl Ord for HeaderType
{
    fn cmp(&self, other: &Self) -> Ordering 
    {
        match self
        {
            HeaderType::Chapter =>
            {
                match other
                {
                    HeaderType::Chapter => Ordering::Equal,
                    _ => Ordering::Greater
                }
            }
            HeaderType::Section =>
            {
                match other 
                {
                    HeaderType::Chapter => Ordering::Less,
                    HeaderType::Section => Ordering::Equal,
                    _ => Ordering::Greater
                }
            }
            HeaderType::Article =>
            {
                match other
                {
                    HeaderType::Article => Ordering::Equal,
                    _ => Ordering::Greater
                }
            }
        }
    }
}

// impl HeaderType
// {
//     pub fn parse(s: &str) -> IResult<&str, Self, ParserError>
//     {
//         let chapt: Result<(&str, &str), nom::Err<ParserError>> = alt((
//                                     tag_no_case("раздел"),
//                                     tag_no_case("разделе"),
//                     )).parse(s);
//         if let Ok(c) = chapt
//         {
//             return Ok((c.0, Self::Chapter));
//         }
//         let sect: Result<(&str, &str), nom::Err<ParserError>> = alt((
//                                    tag_no_case("главами"),
//                                     tag_no_case("главой"),
//                                     tag_no_case("глава"),
//                                     tag_no_case("главы"),
//                                     tag_no_case("главу")
//                     )).parse(s); 
//         if let Ok(c) = sect
//         {
//             return Ok((c.0, Self::Section));
//         }
//         let art: Result<(&str, &str), nom::Err<ParserError>> = alt((
//                                   tag_no_case("статье"),
//                                     tag_no_case("статьи"),
//                                     tag_no_case("статью"),
//                                     tag_no_case("статья"),
//                     )).parse(s);
//         if let Ok(c) = art
//         {
//             return Ok((c.0, Self::Article));
//         }
//         else 
//         {
//             return ParserError::OperationError(["строка ", s, " не является валидным заголовком"].concat()).into();    
//         }
//     }
// }

impl FromStr for HeaderType
{
    type Err = ParserError;
    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        match s.to_lowercase()
        {
            h if h.starts_with("разд") => Ok(HeaderType::Chapter),
            h if h.starts_with("стать") => Ok(HeaderType::Article),
            h if h.starts_with("глав") => Ok(HeaderType::Section),
            _ => Err(ParserError::OperationError(["строка `", s, "` не является валидным заголовком"].concat()))
        }
    }
}