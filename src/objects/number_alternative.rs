
use std::{cmp::Ordering, str::FromStr};
use nom::
{
    branch::{alt, Choice}, bytes::complete::{is_a, tag}, character::{complete::digit1, one_of}, combinator::{all_consuming, eof, not, opt, verify}, error::ParseError, sequence::{delimited, pair, separated_pair}, IResult, Parser
};
use serde::{de::value, Deserialize, Serialize};
use crate::{outputs::AsMarkdown, parsers::{consts::{SUBSCRIPT, SUPERSCRIPT}, space1, ALPHA_NUMERIC}};
use crate::{error::ParserError, parsers::ITEM_NUMBER};


#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
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

impl AsMarkdown for Number
{
    fn as_markdown(&self) -> String
    {
        let mut n = self.number.clone();
        if let Some(va) = self.va_number.as_ref()
        {
            match va.1
            {
                VerticalAlignment::Normal => n.push_str(&va.0),
                VerticalAlignment::Subscript => 
                {
                    n.push_str("<sub>");
                    n.push_str(&va.0);
                    n.push_str("</sub>");
                },
                VerticalAlignment::Superscript =>
                {
                    n.push_str("<sup>");
                    n.push_str(&va.0);
                    n.push_str("</sup>");
                }
            }
        }

        if let Some(postfix) = self.postfix.as_ref()
        {
            n.push_str(postfix);
        }
        n
    }
}



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
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
    }
}

impl FromStr for Number
{
    type Err = ParserError;
    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        let r = Number::parse(s)?;
        Ok(r.1)
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