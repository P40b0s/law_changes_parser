use std::{cmp::Ordering, str::FromStr};
use nom::
{
    branch::alt, bytes::complete::{is_a, tag}, combinator::{all_consuming, not, opt, verify}, error::ParseError, sequence::{delimited, pair, separated_pair}, IResult, Parser
};
use serde::{Deserialize, Serialize};
use crate::parsers::consts::{SUBSCRIPT, SUPERSCRIPT};
use crate::{error::ParserError, parsers::ITEM_NUMBER};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Number
{
    ///Номер пунката статьи итд
    pub number: String,
    ///продолжение номера, но со стилем va верхним или нижним
    pub va_number: Option<(String, VerticalAlignment)>,
    ///символ после номера, например . или )
    pub postfix: Option<String>
}
impl Number
{
    pub fn parse(s: &str) -> IResult<&str, Number, ParserError>
    {
        //если абзац идет первым словом то is_a его сжирал, делаем доп условие
        let num = verify(is_a(ITEM_NUMBER), |s: &str| !s.starts_with("абза"));
        let mut normal_parser =  
        pair(
        num, 
        opt(alt((tag(")"), tag("."))))
        );
        if let Ok((remains, (first_number_part , second_number_part))) = is_superscript_number(s)
        {
            let (remains, postfix) = opt(alt((tag(")"), tag(".")))).parse(remains)?;
            return Ok((remains, Number
            {
                number: first_number_part.to_owned(),
                va_number: Some((second_number_part.to_owned(), VerticalAlignment::Superscript)),
                postfix: postfix.and_then(|a| Some(a.to_owned()))
            }))
        }
        if let Ok((remains, (first_number_part , second_number_part))) = is_subscript_number(s)
        {
            let (remains, postfix) = opt(alt((tag(")"), tag(".")))).parse(remains)?;
            return Ok((remains, Number
            {
                number: first_number_part.to_owned(),
                va_number: Some((second_number_part.to_owned(), VerticalAlignment::Subscript)),
                postfix: postfix.and_then(|a| Some(a.to_owned()))
            }))
        }

        let (remains, (number, postfix)) = 
        normal_parser.parse(s)?;
        Ok((remains, Number
        {
            number: number.to_owned(),
            va_number: None,
            postfix: postfix.and_then(|a| Some(a.to_owned()))
        })
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VerticalAlignment
{
    Subscript,
    Superscript,
    Normal
}

impl Ord for Number
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering 
    {
        if let Ok(n) = self.number.parse::<u32>()
        {
            if let Ok(other_n) = other.number.parse::<u32>()
            {
                let cmp = n.cmp(&other_n);
                if cmp == Ordering::Equal
                {
                    if let Some((additional, _)) = self.va_number.as_ref()
                    {
                        if let Some((other_additional, _)) = other.va_number.as_ref()
                        {
                            if let Ok(an) = additional.parse::<u32>()
                            {
                                if let Ok(other_an) = other_additional.parse::<u32>()
                                {
                                    an.cmp(&other_an)
                                }
                                else 
                                {
                                    additional.cmp(other_additional)    
                                }
                            }
                            else 
                            {
                                additional.cmp(other_additional)    
                            }
                        }
                        else 
                        {
                            Ordering::Less  
                        }
                    }
                    else 
                    {
                        if let Some((other_additional, _)) = other.va_number.as_ref()
                        {
                            Ordering::Less   
                        }
                        else 
                        {
                            cmp 
                        }
                    }
                }
                else 
                {
                    cmp    
                }
                
            }
            else 
            {
                Ordering::Equal    
            }
        }
        else 
        {
            let ord = self.number.cmp(&other.number);
            if ord == Ordering::Equal
            {
                if self.va_number.is_none() && other.va_number.is_none()
                {
                    Ordering::Equal
                }
                else 
                {
                    if let Some((n1, _)) = self.va_number.as_ref()
                    {
                        if let Some((n2, _)) = other.va_number.as_ref()
                        {
                            if let Ok(n) = n1.parse::<u32>()
                            {
                                if let Ok(other_n) = n2.parse::<u32>()
                                {
                                    n.cmp(&other_n)
                                }
                                else 
                                {
                                    Ordering::Less   
                                }
                            }
                            else 
                            {
                                Ordering::Equal
                            }
                        }
                        else 
                        {
                            Ordering::Less    
                        }
                    }
                    else 
                    {
                        if let Some((_, _)) = other.va_number.as_ref()
                        {
                            Ordering::Less
                        }
                        else 
                        {
                            Ordering::Equal
                        }
                    }
                }
            }
            else
            {
                ord
            }
        }
    }
}

impl PartialEq for Number
{
    fn eq(&self, other: &Self) -> bool 
    {
        if self.va_number.is_some() && other.va_number.is_some()
        {
            let (n, _) = self.va_number.as_ref().unwrap();
            let (other_n, _) = other.va_number.as_ref().unwrap();
            &self.number == &other.number && n == other_n && self.postfix == other.postfix
        }
        else
        {
            &self.number == &other.number && self.postfix == other.postfix
        }
    }
}
impl Eq for Number {}

impl PartialOrd for Number
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> 
    {
        Some(self.cmp(other))
        // match self.number.partial_cmp(&other.number) 
        // {
        //     Some(core::cmp::Ordering::Equal) => {}
        //     ord => return ord,
        // }
        // match self.va_number.partial_cmp(&other.va_number) 
        // {
        //     Some(core::cmp::Ordering::Equal) => {}
        //     ord => return ord,
        // }
        // self.postfix.partial_cmp(&other.postfix)
    }
}

impl FromStr for Number
{
    type Err = ParserError;
    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        let r = Number::parse(s)?;
        Ok(r.1)
       
        // let mut number = String::new();
        // let mut index_number: Option<(String, VerticalAlignment)> = None;
        // let mut postfix: Option<String> = None;
        // if s.contains("^")
        // {
        //     if let Some((first, second)) = s.split_once("^")
        //     {
        //         number = first.to_owned();
        //         if second.ends_with(".") || second.ends_with(")")
        //         {
        //             index_number = Some((second[.. second.len() -1].to_owned(), VerticalAlignment::Superscript));
        //             postfix = Some(second[second.len() -1 ..].to_owned());
        //         }
        //         else 
        //         {
        //             index_number = Some((second.to_owned(), VerticalAlignment::Superscript));
        //         }
        //     }
        // }
        // else if s.contains("_")
        // {
        //     if let Some((first, second)) = s.split_once("_")
        //     {
        //         number = first.to_owned();
        //         if second.ends_with(".") || second.ends_with(")")
        //         {
        //             index_number = Some((second[.. second.len() -1].to_owned(), VerticalAlignment::Subscript));
        //             postfix = Some(second[second.len() -1 ..].to_owned());
        //         }
        //         else 
        //         {
        //             index_number = Some((second.to_owned(), VerticalAlignment::Subscript));
        //         }
        //     }
        // }
        // else 
        // {
        //     if s.ends_with(".") || s.ends_with(")")
        //     {
        //         number = s[.. s.len() -1].to_owned();
        //         postfix = Some(s[s.len() -2 .. s.len() -1].to_owned());
        //     }
        //     else 
        //     {
        //         number = s.to_owned();   
        //     }
        // }
        // if number.is_empty()
        // {
        //     Err(Error::ParseNumberError(s.to_owned()))
        // }
        // else 
        // {
        //     Ok(Number
        //     {
        //         number,
        //         va_number: index_number,
        //         postfix
        //     })
        // }
    }
    
}


fn is_subscript_number(s: &str) -> IResult<&str, (&str,&str)>
{ 
    let res = (
        is_a(ITEM_NUMBER),
        is_a(SUBSCRIPT),
        is_a(ITEM_NUMBER),
    ).parse(s)?;
    Ok((res.0, (res.1.0, res.1.2)))
}
fn is_superscript_number(s: &str) -> IResult<&str, (&str,&str)>
{ 
    let res = (
        is_a(ITEM_NUMBER),
        is_a(SUPERSCRIPT),
        is_a(ITEM_NUMBER),
    ).parse(s)?;
    Ok((res.0, (res.1.0, res.1.2)))
}

#[cfg(test)]
mod tests
{
    use crate::objects::number::{Number, VerticalAlignment};

    #[test]
    fn test_ordering()
    {
        let n1 = Number
        {
            number: "123".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: Some(("1".to_owned(), VerticalAlignment::Normal))
        };
        let n2 = Number
        {
            number: "123".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: Some(("2".to_owned(), VerticalAlignment::Normal))
        };
        assert_eq!(n1 < n2, true);
    }
     #[test]
    fn test_ordering2()
    {
        let n1 = Number
        {
            number: "123".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: None
        };
        let n2 = Number
        {
            number: "123".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: Some(("2".to_owned(), VerticalAlignment::Normal))
        };
        assert_eq!(n1 < n2, true);
    }
      #[test]
    fn test_ordering3()
    {
        let n1 = Number
        {
            number: "124".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: None
        };
        let n2 = Number
        {
            number: "123".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: Some(("2".to_owned(), VerticalAlignment::Normal))
        };
        assert_eq!(n1 > n2, true);
    }
    #[test]
    fn test_ordering4()
    {
        let n1 = Number
        {
            number: "а".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: None
        };
        let n2 = Number
        {
            number: "в".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: Some(("2".to_owned(), VerticalAlignment::Normal))
        };
        assert_eq!(n1 < n2, true);
    }
    #[test]
    fn test_ordering5()
    {
        let n1 = Number
        {
            number: "а".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: Some(("24".to_owned(), VerticalAlignment::Normal))
        };
        let n2 = Number
        {
            number: "а".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: Some(("23".to_owned(), VerticalAlignment::Normal))
        };
        assert_eq!(n1 > n2, true);
    }
     #[test]
    fn test_eq1()
    {
        let n1 = Number
        {
            number: "123".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: Some(("1".to_owned(), VerticalAlignment::Normal))
        };
        let n2 = Number
        {
            number: "123".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: Some(("1".to_owned(), VerticalAlignment::Normal))
        };
        assert_eq!(n1 == n2, true);
    }
    #[test]
    fn test_eq2()
    {
        let n1 = Number
        {
            number: "123".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: None
        };
        let n2 = Number
        {
            number: "123".to_owned(),
            postfix: Some(")".to_owned()),
            va_number: None
        };
        assert_eq!(n1 == n2, true);
    }
}