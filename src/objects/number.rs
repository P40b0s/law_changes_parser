use std::{cmp::Ordering, str::FromStr};
use nom::
{
    branch::{alt, Choice}, bytes::complete::{is_a, tag}, character::{complete::digit1, one_of}, combinator::{all_consuming, eof, not, opt, verify}, error::ParseError, sequence::{delimited, pair, separated_pair}, IResult, Parser
};
use serde::{de::value, Deserialize, Serialize};
use crate::{outputs::AsMarkdown, parsers::{consts::{SUBSCRIPT, SUPERSCRIPT}, space1, ALPHA_NUMERIC}};
use crate::{error::ParserError, parsers::ITEM_NUMBER};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberIndex
{
    Subscript,
    Superscript,
    Normal
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq)]
pub struct Number
{
    ///Номер пунката статьи итд
    pub number: String,
    ///продолжение номера, но со стилем va верхним или нижним
    pub number_index: NumberIndex,
    ///символ после номера, например . или )
    pub postfix: Option<String>,
    pub next: Option<Box<Number>>
}
impl Number
{
    pub fn get_last_mut(&mut self) -> &mut Self
    {
        let mut current = self;
        while let Some(ref mut next) = current.next
        {
            current = next;
        }
        current
    }
    pub fn parse<'a>(s: &'a str) -> IResult<&'a str, Number, ParserError>
    {
        let index = NumberIndex::Normal;
        complex_number_parser(s, index, None)
    }
    pub fn new<T: ToString>(number: T, index: NumberIndex, postfix: Option<String>) -> Self
    {
        Number
        {
            number: number.to_string(),
            number_index: index,
            postfix,
            next: None
        }
    }
    pub fn apply(&mut self, number: Self)
    {
       let last = self.get_last_mut();
       last.next = Some(Box::new(number));
    }
}

// Итератор по значениям (неизменяемый)
pub struct NumberIterator<'a> 
{
    current: Option<&'a Number>,
}
impl<'a> Iterator for NumberIterator<'a> 
{
    type Item = &'a Number;
    fn next(&mut self) -> Option<Self::Item> 
    {
        let current = self.current;
        self.current = current.and_then(|n| n.next.as_deref());
        current
    }
}

// Итератор по владеющим значениям (потребляет структуру)
pub struct NumberIntoIterator 
{
    current: Option<Number>,
}

impl Iterator for NumberIntoIterator 
{
    type Item = Number;

    fn next(&mut self) -> Option<Self::Item> 
    {
        self.current.take().map(|mut n| 
        {
            self.current = n.next.take().map(|b| *b);
            n
        })
    }
}


///например а в подпункте а)
fn is_alpha_number(s: &str) -> IResult<&str, &str, ParserError>
{
    let tags = [tag("а"),tag("б"),tag("в"),tag("г"),tag("д"),tag("е"),tag("ё"),tag("ж"),tag("з"),tag("и"),tag("й"),tag("к"),tag("л"),tag("м"),tag("н"),tag("о"),tag("п"),tag("р"),tag("с"),tag("т"),tag("у"),tag("ф"),tag("х"),tag("ц"),tag("ч"),tag("ш"),tag("щ"),tag("ъ"),tag("ы"),tag("ь"),tag("э"),tag("ю"),tag("я")];
    let item = alt(tags).parse(s)?;
    //для того чтобы не хватать слова целиком у подпункта может быть только одна буква
    let _ = alt((is_digit_number, space1, eof, tag(":"), is_postfix)).parse(item.0)?;
    Ok((item.0, item.1))
    //let res = one_of("абвгдеёжзийклмнопрстуфхцчшщъыьэюя").parse(s)?;
}
fn is_digit_number(s: &str) -> IResult<&str, &str, ParserError>
{
    digit1(s)
}
fn is_postfix(s: &str) -> IResult<&str, &str, ParserError>
{
    alt((tag(")"), tag("."), tag("-"))).parse(s)
}
fn is_superscript(s: &str) -> IResult<&str, &str, ParserError>
{
    tag("^").parse(s)
}

fn is_subscript(s: &str) -> IResult<&str, &str, ParserError>
{
    tag("_").parse(s)
}


fn complex_number_parser<'a>(s: &'a str, index:  NumberIndex, mut input_number: Option<Number>) -> IResult<&'a str, Number, ParserError>
{
    //logger::debug!("input {} index: {:?} number: {:?}", s, index, input_number.as_ref());
    if let Ok((r, _)) = is_superscript(s)
    {
        return complex_number_parser(r, NumberIndex::Superscript, input_number);
    }
    if let Ok((r, _)) = is_subscript(s)
    {
        return complex_number_parser(r, NumberIndex::Subscript, input_number);
    }
    //logger::debug!("input {} index: {:?} number: {:?}", s, index, input_number.as_ref());
    let (mut remains, number) = alt((is_alpha_number, is_digit_number)).parse(s)?;
    //logger::debug!("input {} index: {:?} number: {:?}", remains, index, input_number.as_ref());
    let postfix = is_postfix(remains).map_or(None, |m|
    {
        remains = m.0;
        Some(m.1.to_owned())
    });
    //logger::debug!("input {} index: {:?} number: {:?}", remains, index, input_number.as_ref());
    let new_num = Number
    {
        number: number.to_owned(),
        number_index: index,
        postfix,
        next: None
    };
    //logger::debug!("input {} index: {:?} new_number: {:?}", remains, index, &new_num);
    let space: Result<(&str, &str), nom::Err<ParserError>> = alt((space1, eof, tag(":"))).parse(remains);
    if space.is_ok()
    {
       
        if let Some(mut n) = input_number
        {
            n.get_last_mut().next = Some(Box::new(new_num));
            //logger::debug!("input {} index: {:?} number: {:?}", remains, index, &n);
            Ok((remains, n))
        }
        else 
        {
            //logger::debug!("input {} index: {:?} number: {:?}", remains, index, &new_num);
            Ok((remains, new_num))    
        }
    }
    else 
    {
        if let Some(n) = input_number.as_mut()
        {
            n.get_last_mut().next = Some(Box::new(new_num));
            //logger::debug!("input {} index: {:?} number: {:?}", remains, index, n);
            complex_number_parser(remains, index, input_number)
        }
        else 
        {
            //logger::debug!("input {} index: {:?} number: {:?}", remains, index, &new_num);
            complex_number_parser(remains, index, Some(new_num))
        }
    }
}


// impl AsMarkdown for Number
// {
//     fn as_markdown(&self) -> String
//     {
//         let mut output = String::new();
//         let mut current_index: NumberIndex = NumberIndex::Normal;
//         let count = self.next.iter().count();
//         output.push_str(&self.number);
//         if let Some(postfix) = self.postfix.as_ref()
//         {
//             output.push_str(&postfix);
//         }
//         for (i, v) in self.next.iter().enumerate()
//         {
//             if v.number_index == NumberIndex::Superscript
//             {
//                 if current_index != v.number_index
//                 {
//                     output.push_str("<sup>");
//                     current_index = v.number_index;
//                 }
//             }
//             if v.number_index == NumberIndex::Subscript
//             {
//                 if current_index != v.number_index
//                 {
//                     output.push_str("<sub>");
//                     current_index = v.number_index;
//                 }
//             }
//             output.push_str(&v.number);
//             if let Some(postfix) = v.postfix.as_ref()
//             {
//                 if i == count -1
//                 {
//                     if current_index == NumberIndex::Superscript
//                     {
//                         output.push_str("</sup>");
//                     }
//                     if current_index == NumberIndex::Subscript
//                     {
//                         output.push_str("</sub>");
//                     }
//                 }
//                 output.push_str(postfix);
//             }
//             else 
//             {
//                 if i == count -1
//                 {
//                     if current_index == NumberIndex::Superscript
//                     {
//                         output.push_str("</sup>");
//                     }
//                     if current_index == NumberIndex::Subscript
//                     {
//                         output.push_str("</sub>");
//                     }
//                 }
//             }
//         }
//         output
//     }
// }

impl AsMarkdown for Number
{
    fn as_markdown(&self) -> String
    {
        let mut output = String::new();
        let mut current_index: NumberIndex = NumberIndex::Normal;
        output.push_str(&self.number);
        if let Some(postfix) = self.postfix.as_ref()
        {
            output.push_str(&postfix);
        }
        let mut current = self;
        while let Some(next) = current.next.as_ref()
        {
            current = next;
            if current.number_index == NumberIndex::Superscript
            {
                if current_index != current.number_index
                {
                    output.push_str("<sup>");
                    current_index = current.number_index;
                }
            }
            if current.number_index == NumberIndex::Subscript
            {
                if current_index != current.number_index
                {
                    output.push_str("<sub>");
                    current_index = current.number_index;
                }
            }
            output.push_str(&current.number);
            if let Some(postfix) = current.postfix.as_ref()
            {
                if current.next.is_none()
                {
                    end_index(&current_index, &mut output);
                }
                output.push_str(postfix);
            }
            else 
            {
                if current.next.is_none()
                {
                    end_index(&current_index, &mut output);
                }
            }
        }
        output
    }
}
fn end_index(index: &NumberIndex, output: &mut String)
{
    if index == &NumberIndex::Superscript
    {
        output.push_str("</sup>");
    }
    if index == &NumberIndex::Subscript
    {
        output.push_str("</sub>");
    }
}





impl Ord for Number
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering 
    {
        let num1 = self.number.parse::<u32>();
        let num2 = other.number.parse::<u32>();
        if num1.is_ok() && num2.is_ok()
        {
            let cmp = digit_and_digit(num1.as_ref().unwrap(), num2.as_ref().unwrap());
            logger::debug!("n1: {}, n2: {}, eq:{:?}", &self.number, &other.number, &cmp);
            if cmp == Ordering::Equal
            {
                let num1_next = self.next.as_ref();
                let num2_next = other.next.as_ref();
                if let (Some(n1), Some(n2)) = (num1_next, num2_next)
                {
                    return n1.cmp(n2);
                }
                else 
                {
                    if num1_next.is_some()
                    {
                        return Ordering::Greater;
                    }
                    else 
                    {
                        return  Ordering::Less;    
                    }
                }
            }
            else 
            {
                return cmp; 
            }
        }
        else 
        {
            if let Ok(n) = num1.as_ref()
            {
                return digit_and_alpha(n, &other.number);
            }
            else if let Ok(n2) = num2.as_ref()
            {
                return alpha_and_digit(&self.number, n2);
            }
            else 
            {
                let cmp = self.number.cmp(&other.number);
                if cmp == Ordering::Equal
                {
                    if let (Some(next1), Some(next2)) = (self.next.as_ref(), other.next.as_ref())
                    {
                        return next1.cmp(next2);
                    }
                    else 
                    {
                        if self.next.is_some()
                        {
                            return Ordering::Greater;
                        }
                        else 
                        {
                            return  Ordering::Less;    
                        }
                    }
                }
                else 
                {
                    return cmp;
                } 
            }
        }
    }
}
impl PartialEq for Number
{
    fn eq(&self, other: &Self) -> bool 
    {
        let num1 = self.number.parse::<u32>();
        let num2 = other.number.parse::<u32>();
        if let (Ok(n1), Ok(n2)) = (num1.as_ref(), num2.as_ref())
        {
            if n1.eq(n2)
            {
                if let (Some(next1), Some(next2)) = (self.next.as_ref(), other.next.as_ref())
                {
                    return next1.eq(next2);
                }
                else 
                {
                    if self.next.is_none() && other.next.is_none()
                    {
                        return true;
                    }
                    else 
                    {
                        return false;
                    }
                }
            }
            else
            {
                return false;
            }
        }
        else 
        {
            if num1.is_err() && num2.is_err()
            {
                let r = self.number.eq(&other.number);
                if r
                {
                    if let (Some(next1), Some(next2)) = (self.next.as_ref(), other.next.as_ref())
                    {
                        return next1.eq(next2)
                    }
                    else 
                    {
                        return false;
                    }
                }
                else 
                {
                    return r;
                }
            }
            else 
            {
                return false;
            }
        }
    }
}
impl PartialOrd for Number
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> 
    {
        Some(self.cmp(other))
    }
}

fn digit_and_digit(num1: &u32, num2: &u32) -> Ordering
{
    num1.cmp(num2)
}
fn digit_and_alpha(_: &u32, _: &str) -> Ordering
{
    Ordering::Less
}
fn alpha_and_digit(_: &str, _: &u32) -> Ordering
{
    Ordering::Greater
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



#[cfg(test)]
mod tests
{
    use crate::{objects::number::{Number, NumberIndex}, outputs::AsMarkdown};

    #[test]
    fn test_ordering()
    {
        let mut n1 = Number::new("123", NumberIndex::Normal, Some("-".to_owned()));
        let n1_1 = Number::new("1", NumberIndex::Normal, None);
        n1.apply(n1_1);
        let mut n2 = Number::new("123", NumberIndex::Normal, Some("-".to_owned()));
        let n2_1 = Number::new("2", NumberIndex::Normal, None);
        n2.apply(n2_1);
        assert_eq!(n1 < n2, true);
    }
     #[test]
    fn test_ordering2()
    {
        let n1 = Number::new("123", NumberIndex::Normal, Some("-".to_owned()));
        let mut n2 = Number::new("123", NumberIndex::Normal, Some("-".to_owned()));
        let n2_1 = Number::new("1", NumberIndex::Normal, None);
        n2.apply(n2_1);
        assert_eq!(n1 < n2, true);
    }
      #[test]
    fn test_ordering3()
    {
        let n1 = Number::new("124", NumberIndex::Normal, Some("-".to_owned()));
        let mut n2 = Number::new("123", NumberIndex::Normal, Some("-".to_owned()));
        let n2_1 = Number::new("2", NumberIndex::Normal, None);
        n2.apply(n2_1);
        assert_eq!(n1 > n2, true);
    }
    #[test]
    fn test_ordering4()
    {
        let mut n1 = Number::new("а", NumberIndex::Normal, None);
        let mut n2 = Number::new("в", NumberIndex::Subscript, None);
        let n2_1 = Number::new("2", NumberIndex::Normal, Some(")".to_owned()));
        n2.apply(n2_1);
        assert_eq!(n1 < n2, true);
    }
    #[test]
    fn test_ordering5()
    {
        let mut n1 = Number::new("а", NumberIndex::Normal, None);
        let n1_1 = Number::new("24", NumberIndex::Normal, Some(")".to_owned()));
        n1.apply(n1_1);
        let mut n2 = Number::new("а", NumberIndex::Normal, None);
        let n2_1 = Number::new("23", NumberIndex::Normal, Some(")".to_owned()));
        n2.apply(n2_1);
        assert_eq!(n1 > n2, true);
    }
     #[test]
    fn test_ordering6()
    {
        logger::StructLogger::new_default();
        let mut n1 = Number::new("24", NumberIndex::Normal, None);
        let n1_1 = Number::new("2", NumberIndex::Normal, Some(")".to_owned()));
        n1.apply(n1_1);
        let mut n2 = Number::new("20", NumberIndex::Normal, None);
        let n2_1 = Number::new("3", NumberIndex::Superscript, None);
        let n2_2 = Number::new("20", NumberIndex::Normal, None);
        n2.apply(n2_1);
        n2.apply(n2_2);
        assert_eq!(n1 > n2, true);
    }
    #[test]
    fn test_eq3()
    {
        let n1 = Number
        {
            number: "123".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: Some(")".to_owned()),
            next: None
            
        };
        let n2 = Number
        {
            number: "123".to_owned(),
            postfix: Some(")".to_owned()),
            number_index: crate::objects::number::NumberIndex::Normal,
            next: None
        };
        assert_eq!(n1 == n2, true);
    }
    #[test]
    fn test_eq4()
    {
        let n1 = Number
        {
            number: "123".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "1".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        let n2 = Number
        {
            number: "123".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "1".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        assert_eq!(n1 == n2, true);
    }

    #[test]
    fn test_eq5()
    {
        let n1 = Number
        {
            number: "а".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "1".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some(")".to_owned()),
                next: None
            }))
            
        };
        let n2 = Number
        {
            number: "а".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "1".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some(")".to_owned()),
                next: None
            }))
            
        };
            
        assert_eq!(n1 == n2, true);
    }

    #[test]
    fn test_eq6()
    {
        let n1 = Number
        {
            number: "123".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "1".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        let n2 = Number
        {
            number: "123".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "2".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        assert_eq!(n1 != n2, true);
    }

     #[test]
    fn test_ord_1()
    {
        logger::StructLogger::new_default();
        let n1 = Number
        {
            number: "123".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "1".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        let n2 = Number
        {
            number: "123".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "2".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        let n3 = Number
        {
            number: "123".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: None,
                next: None
            }))
            
        };
        let mut values = vec![n2, n1, n3];
        values.sort();
        logger::debug!("{:?}", &values);
        assert_eq!(&values[0].get_last_mut().number, "12");
    }

    #[test]
    fn test_ord_2()
    {
        logger::StructLogger::new_default();
        let n1 = Number
        {
            number: "а".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "1".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        let n2 = Number
        {
            number: "а".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "2".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        let n3 = Number
        {
            number: "а".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: None,
                next: None
            }))
            
        };
        let mut values = vec![n2, n1, n3];
        values.sort();
        logger::debug!("{:?}", &values);
        assert_eq!(&values[0].get_last_mut().number, "12");
    }

    #[test]
    fn test_ord_3()
    {
        logger::StructLogger::new_default();
        let n1 = Number
        {
            number: "а".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "1".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        let n2 = Number
        {
            number: "а".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "2".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        let n3 = Number
        {
            number: "б".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "4".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: None,
                next: None
            }))
            
        };
        let mut values = vec![n2, n1, n3];
        values.sort();
        logger::debug!("{:?}", &values);
        assert_eq!(&values[2].get_last_mut().number, "4");
    }
    #[test]
    fn test_ord_4()
    {
        logger::StructLogger::new_default();
        let n1 = Number
        {
            number: "а".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "1".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        let n2 = Number
        {
            number: "а".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "12".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: Some("-".to_owned()),
                next: Some(Box::new(Number 
                {
                    number: "2".to_owned(),
                    postfix: Some(")".to_owned()),
                    number_index: NumberIndex::Superscript,
                    next: None
                }))
            }))
            
        };
        let n3 = Number
        {
            number: "б".to_owned(),
            number_index: crate::objects::number::NumberIndex::Normal,
            postfix: None,
            next: Some(Box::new(Number
            {
                number: "4".to_owned(),
                number_index: NumberIndex::Superscript,
                postfix: None,
                next: None
            }))
            
        };
        let mut values = vec![n2, n1, n3];
        values.sort();
        logger::debug!("{:?}", &values);
        assert_eq!(&values[2].get_last_mut().number, "4");
    }
   

    #[test]
    fn test_parsers_1()
    {
        logger::StructLogger::new_default();
        let val = "1-2^2-3:";
        let parsed = super::Number::parse(val).unwrap();
        logger::debug!("{:?}", parsed);
    }
    #[test]
    fn test_parsers_2()
    {
        let val = "а)";
        let parsed = super::is_alpha_number(val).unwrap();
        assert_eq!(parsed.1, "а");
    }
    #[test]
    fn test_parsers_3()
    {
        let val = "1)";
        let parsed = super::is_digit_number(val).unwrap();
        assert_eq!(parsed.1, "1");
    }
     #[test]
    fn test_parsers_4()
    {
        let val = "1)";
        
        let parsed = super::is_digit_number(val).unwrap();
        assert_eq!(parsed.1, "1");
    }
     #[test]
    fn test_parsers_5()
    {
        logger::StructLogger::new_default();
        let val = "дополнить";
        let parsed = super::Number::parse(val);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_to_markdown_1()
    {
         let mut n1 = Number::new("а", NumberIndex::Normal, None);
        let n1_1 = Number::new("24", NumberIndex::Subscript, Some(")".to_owned()));
        n1.apply(n1_1);
        let mut n2 = Number::new("а", NumberIndex::Normal, None);
        let n2_1 = Number::new("23", NumberIndex::Superscript, Some(")".to_owned()));
        n2.apply(n2_1);
        
        let num1 = n1.as_markdown();
        let num2 = n2.as_markdown();
        assert_eq!(&num1, "а<sub>24</sub>)");
        assert_eq!(&num2, "а<sup>23</sup>)");
    }
     #[test]
    fn test_to_markdown_2()
    {
        logger::StructLogger::new_default();
        let mut n1 = Number::new("а", NumberIndex::Normal, None);
        let n1_1 = Number::new("1", NumberIndex::Subscript, Some("-".to_owned()));
        let n1_2 = Number::new("24", NumberIndex::Subscript, Some(")".to_owned()));
        n1.apply(n1_1);
        n1.apply(n1_2);
        logger::debug!("{:?}", &n1);
        let num1 = n1.as_markdown();
        assert_eq!(&num1, "а<sub>1-24</sub>)");
    }
}