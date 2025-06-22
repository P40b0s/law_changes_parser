use logger::{info, debug};
use nom::
{
    branch::alt, bytes::complete::{is_a, tag, tag_no_case}, combinator::{eof, map, verify}, error::ParseError, multi::many1, sequence::{delimited, pair, separated_pair, tuple}, IResult, Parser
};
use crate::{change_path::{ChangePath, TargetPath}, error::ParserError, objects::{header_type::HeaderType, item_type::ItemType, number::Number}};
use super::{HEADER_NUMBER, INDENT_NUMBERS, consts::{ALPHA_L, ALPHA_L_VEC}, chars::definition};
use super::{space0, space1};

// pub trait ReversePath
// {
//     fn try_reverse_path(&mut self);
// }

// impl ReversePath for Vec<ChangePath>
// {
//     fn try_reverse_path(&mut self) 
//     {
//         self.reverse();
//         for jp in self
//         {
//             jp.reverse();
//         }
//     }
// }

///`подпункте 1 пункте 3 статьи 2^2` (до конца номера, без пробела)
/// на выходе получим строку jsonpath: $.body.headers[?(@.number.val == "2^2")].items[?(@.number.val == "3")].items[?(@.number.val == "1")]
pub fn paths(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let mut parser =   many1(                 
        alt((header_sequence, item_sequence, indent_sequence)),
    );
    let (remain,  words) = parser.parse(s)?;
    let tp = TargetPath::flatten(words);
    //let mut w: Vec<ChangePath> =  words.into_iter().flatten().collect();
    //let path = words.into_iter().sum();
    Ok((remain, tp))
}
///`1) в статье 2^2:`
pub fn only_path_definition(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remains, (_,_,_,_,p,_,_)) = ((Number::parse, space1, tag("в"), space1, paths, definition, eof)).parse(s)?;
    Ok((remains, p))
}


pub fn separate_tags(s: &str) -> IResult<&str, &str, ParserError>
{
    alt((tag(", "), tag(", "), tag(" и "), tag(" и "), tag(" - "), tag(" - "))).parse(s)
}
pub fn separate_range_tags(s: &str) -> IResult<&str, &str, ParserError>
{
    alt((tag(" - "), tag(" - "))).parse(s)
}
///Вычисляем последовательность номеров в том числе с индексами 2^2
pub fn number_sequence(s: &str) -> IResult<&str, Vec<String>, ParserError>
{
    let mut numbers: Vec<String> = vec![];
    let (remain, num) = 
    many1(
        (
                ((verify_number, separate_tags, verify_number)),
                alt((tag(", "), space0)),
                
                )
    ).parse(s)?;
    let num = num as Vec<((&str, &str, &str), &str)>;
    for ((n1, sep, n2), _) in num
    {
        let n1_splitted = n1.split_once("^");
        let n2_splitted = n2.split_once("^");
        let n1_first_usize = n1_splitted.and_then(|p| p.0.parse::<usize>().ok())
                                                    .or(n1.parse::<usize>().ok());
        let n1_second_usize = n1_splitted.and_then(|p| p.1.parse::<usize>().ok());
        let n2_first_usize = n2_splitted.and_then(|p| p.0.parse::<usize>().ok())
                                                    .or(n2.parse::<usize>().ok());
        let n2_second_usize = n2_splitted.and_then(|p| p.1.parse::<usize>().ok());
        let second_range = if n2_splitted.is_some()
        {
            if n1_second_usize.is_some()
            {
                Some(*n1_second_usize.as_ref().unwrap()..n2_second_usize.unwrap()+1)
            }
            else
            {
                Some(0..n2_second_usize.unwrap()+1)
            }
        }
        else if n1_splitted.is_some()
        {
            //let num = n1.to_owned();
            //debug!("Распознан номерованный список № {}", &num);
            //numbers.push(num);
            None
        }
        else {None};
        if n1_first_usize.is_some() && n2_first_usize.is_some() && sep.contains("-")
        {
            let range = n1_first_usize.unwrap()..n2_first_usize.unwrap()+1;
            let range_start = range.start;
            for n in range
            {
                if second_range.is_some()
                {
                    for second_number in second_range.clone().unwrap()
                    {
                        if second_number == 0
                        {
                            debug!("Распознан номерованный список № {}", &n);
                            numbers.push(n.to_string());
                        }
                        else
                        {
                            let num = [n.to_string(), "^".to_owned(), second_number.to_string()].concat();
                            debug!("Распознан номерованный список № {}", &num);
                            numbers.push(num);
                        }
                    }
                }
                else
                {
                    if n == range_start && n1_second_usize.is_some()
                    {
                        //оставим пустой так как уже все добавлено выше
                        //let num = [n.to_string(), "^".to_owned(), n1_second_usize.as_ref().unwrap().to_string()].concat();
                        //debug!("Распознан номерованный список № {}", &num);
                        //numbers.push(num);
                    }
                    else
                    {
                        debug!("Распознан номерованный список № {}", &n);
                        numbers.push(n.to_string());
                    }
                }
            }
            continue;
        }
        let chars = get_chars_range(n1, n2);
        numbers.extend(chars);
        //TODO возможно понадобится реализация для 1^1  и 1_1 пока таких примеров не видел просто дабавляем их без ренджа
        numbers.push(n1.to_string());
        numbers.push(n2.to_string()); 
    }
    Ok((remain, numbers))
}

pub fn number(s: &str) -> IResult<&str, Vec<String>, ParserError>
{
    let mut parser = pair(
        verify_number,
            space0,
        );
    let (remain, (num, _)) = parser.parse(s)?;
    Ok((remain,  vec![num.to_owned()]))
}


///Для строк Rust использует кодировку UTF-8. В этой кодировке кириллица кодируется с помощью 2 байт. Соответственно длинна строки для русских букв будет в 2 раза больше.
fn get_chars_range(n1: &str, n2: &str) -> Vec<String>
{
    let mut numbers: Vec<String> = vec![];
    if n1.trim().chars().count() == 1 && n2.trim().chars().count() == 1
    {
        let n1_c = n1.as_bytes()[0];
        let n1_char = n1_c as char;
        let n2_c = n2.as_bytes()[0];
        let n2_char = n2_c as char;
        if n1_char.is_alphabetic() && n2_char.is_alphabetic()
        {
            let n1_index = ALPHA_L_VEC.iter().position(|p| p == &n1);
            let n2_index = ALPHA_L_VEC.iter().position(|p| p == &n2);
            if n1_index.is_some() && n2_index.is_some()
            {
                let slice = &ALPHA_L_VEC[n1_index.unwrap()..=n2_index.unwrap()];
                for n in slice
                {
                    info!("Распознан номерованный список № {}", &n);
                    numbers.push(n.to_string());
                }
            }
           
        }
    }
    numbers
}
///Верификация номера: 1/1^1/2_1/"а"
pub fn verify_number(s: &str) -> IResult<&str, &str, ParserError>
{
    let word_name_item = verify(is_a(ALPHA_L), |s: &str| s.chars().count() == 1);
    let w_number = delimited(tag("\""), word_name_item, tag("\""));
    let number = is_a(HEADER_NUMBER);
    let (remain, result) = alt((w_number, number)).parse(s)?;
    return Ok((remain, result));
}



pub fn header_sequence(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let mut tp = TargetPath::new();
    let mut parser = many1(
        
                        (
                            separated_pair(
                                alt((
                                    tag_no_case("статьями"),
                                    tag_no_case("статьей"),
                                    tag_no_case("статье"),
                                    tag_no_case("статьи"),
                                    tag_no_case("статью"),
                                    tag_no_case("главами"),
                                    tag_no_case("главой"),
                                    tag_no_case("глава"),
                                    tag_no_case("главы"),
                                    tag_no_case("главу"),
                            )),
                            space1,
                            alt((number_sequence, number))),
                            alt((tag(", "), tag("и "), space0, eof))
                        )
                    
    );
    let (remain, num) = parser.parse(s)?;
    let num = num as Vec<((&str, Vec<String>), &str)>;
    for ((h_type, number), _ ) in num
    {
        for n in number
        {
            let _ = tp.add_header(h_type, &n).map_err(|e| e.into())?;
        }
    }
    Ok((remain, tp))
}

pub fn item_sequence(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remain, num) = 
    many1(
        (
                        
                    separated_pair(
                        alt((
                        tag_no_case("подпунктами"),
                        tag_no_case("подпунктом"), 
                        tag_no_case("подпункте"), 
                        tag_no_case("подпункты"), 
                        tag_no_case("подпункт"), 
                        tag_no_case("пунктами"), 
                        tag_no_case("пунктом"), 
                        tag_no_case("пунктов"), 
                        tag_no_case("пункте"), 
                        tag_no_case("пункта"),
                        tag_no_case("пункты"), 
                        tag_no_case("пункт"), 
                        tag_no_case("частями"), 
                        tag_no_case("частью"), 
                        tag_no_case("часть"), 
                        tag_no_case("части")
                    )),
                    space1,
                    alt((
                        number_sequence,
                        number,
                        map(indent_number_sequence, |m| m.into_iter().map(|n| n.to_string()).collect::<Vec<String>>()),
                        map(indent_number, |m|m.into_iter().map(|n| n.to_string()).collect::<Vec<String>>())))),
                    alt((tag(", "), tag("и "), space0, eof))
                        
                )
    ).parse(s)?;
    let mut tp = TargetPath::new();
    let num = num as Vec<((&str, Vec<String>), &str)>;
    for ((n_type, number), _ ) in num
    {
        for n in number
        {
            tp.add_item(&n, n_type).map_err(|e| e.into())?;
        }
    }
    Ok((remain, tp))
}



pub fn verify_indent_number(s: &str) -> IResult<&str, &str, ParserError>
{
    let (remain, result) = is_a(ALPHA_L)(s)?;
    if result.len() > 5 && INDENT_NUMBERS.contains_key(result)
    {
        return Ok((remain, result));
    }
    else
    {
        //working
        //return Err(nom::Err::Error(Error::append(remain, nom::error::ErrorKind::Tag, Error::IndentQueueTokensError(s.to_owned()))))
        //return Err(nom::Err::Error(Error::IndentQueueTokensError(s.to_owned())))
        //Error::err(Error::IndentQueueTokensError(s.to_owned()))
        ParserError::IndentQueueTokensError(s.to_owned()).into()
        // return nom::Err::Error(Error::IndentQueueTokensError(s.to_owned()), )
        // let err = "Неправильная последовательность токенов номера абзаца";
        // let e =  nom_warn!(err, s);
        // return Err(e);        
    }
}

pub fn indent_number_sequence(s: &str) -> IResult<&str, Vec<usize>, ParserError>
{
    let mut numbers: Vec<usize> = vec![];
    let (remain, num) = 
    many1(
        (
                
                    separated_pair(verify_indent_number, separate_tags, verify_indent_number),
                    alt((tag(", "), space0)),
                
            )
    ).parse(s)?;
    let num = num as Vec<((&str, &str), &str)>;
    for ((n1, n2), _) in num
    {
        let range = *INDENT_NUMBERS.get(n1).unwrap()..=*INDENT_NUMBERS.get(n2).unwrap();
        for n in range
        {
            numbers.push(n);
        }
    }
    Ok((remain, numbers))
}
pub fn indent_number(s: &str) -> IResult<&str, Vec<usize>, ParserError>
{
    let (remain, (num, _)) = 
    pair(
        verify_indent_number,
            space0,
        ).parse(s)?;
    Ok((remain,  vec![(*INDENT_NUMBERS.get(num).unwrap())]))
}

pub fn indent_sequence(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remain, num) = 
    many1(
        (
                        
                            separated_pair(alt((
                                tag_no_case("абзацами"),
                                tag_no_case("абзацем"),
                                tag_no_case("абзаце"),
                                tag_no_case("абзацы"),
                                tag_no_case("абзац")
                            )),
                            space1,
                            alt((indent_number_sequence, indent_number))),
                            alt((tag(", "), tag("и "), space0, eof))
                        
                    )
    ).parse(s)?;
    let mut tp = TargetPath::new();
    for ((_, number), _ ) in num
    {
        for n in number
        {
            tp.add_indent(n as u32);
        }
    }
    Ok((remain, tp))
}





#[cfg(test)]
mod tests
{
    use logger::{error, info};
    use crate::parsers::path_plus_number::item_sequence;

    use super::paths;

    #[test]
    fn test_path()
    {
        logger::StructLogger::new_default();
        let s = "подпункте 1 пункте 3 статьи 2^2 ";
        let p = paths(s).unwrap();
        //assert_eq!(&p.1.get_body_jsonpath(), "$.body.headers[?(@.number.val == \"2^2\")].items[?(@.number.val == \"3\")].items[?(@.number.val == \"1\")]");
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);

    }
    #[test]
    fn test_long_numbering()
    {
         logger::StructLogger::new_default();
        let s = "пунктов 1, 3, 5 - 7, 9 - 11, 14 и 17 статьи 1";
        let p = paths(s);
        if p.is_err()
        {
            error!("{:?}", p);
        }
        else 
        {
            let p = p.unwrap();
            info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
        }
        

    }

    #[test]
    fn test_only_path()
    {
         logger::StructLogger::new_default();
        let check_all_strings : Vec<&str> = vec![
            r#"1) в статье 40:"#,
            r#"п) в пункте 16:"#,
            r#"1) в статье 4^12:"#,
            r#"п) в пункте 1_6:"#,
            r#"п) в пункте 1^1-1:"#,
        ];
        for s in check_all_strings
        {
            let p = super::only_path_definition(s).unwrap();
            info!("тест строки ->{} остаток токенов ->{:?}", p.0, p.1);
        }
    }

    
    #[test]
    fn test_items1()
    {
         logger::StructLogger::new_default();
        let s = "части 10, 11, 13 - 17 ";
        let p = item_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_items2()
    {
         logger::StructLogger::new_default();
        let s = "пункт 10, 11, 13 и 14, 15-17 ";
        let p = item_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_items3()
    {
         logger::StructLogger::new_default();
        let s = "подпункты \"з\" - \"ю\" исключить";
        let p = item_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| перечисление->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_items4()
    {
         logger::StructLogger::new_default();
        let s = "части 2^2 и 2^9 ";
        let p = item_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_items5()
    {
         logger::StructLogger::new_default();
        let s = "части 2^2 - 2^9 ";
        let p = item_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_items6()
    {
         logger::StructLogger::new_default();
        let s = "части 2 - 2^4 ";
        let p = item_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_numbers()
    {
         logger::StructLogger::new_default();
        let s = "10, 11, 13 - 17 ";
        let p = super::number_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_indents_numbers()
    {
         logger::StructLogger::new_default();
        let s = r#"абзацами седьмым - десятым и абзацами одиннадцатым и двенадцатым следующего содержания:"#;
        let p = super::indent_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_indents_numbers2()
    {
         logger::StructLogger::new_default();
        let s = "абзацами одиннадцатым и двенадцатым";
        let p = super::indent_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_indents_numbers3()
    {
         logger::StructLogger::new_default();
        let s = "абзац одиннадцатый следующего содержания:";
        let p = super::indent_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_indents_numbers4()
    {
         logger::StructLogger::new_default();
        let s = "абзацем следующего содержания:";
        let p = super::indent_sequence(s).err().unwrap();
        info!("тест строки ->{} ошибка ->{}", s, p.to_string());
    }
     #[test]
    fn test_indents_numbers5()
    {
        logger::StructLogger::new_default();
        let s = r#"абзацем третьим следующего содержания:"#;
        let p = super::indent_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
     #[test]
    fn test_indents_numbers6()
    {
        logger::StructLogger::new_default();
        let s = r#"абзац второй следующего содержания:"#;
        let p = super::indent_sequence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->`{}`| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_items_numbers6()
    {
         logger::StructLogger::new_default();
        let s = "статьями 9^1 и 9^2 следующего содержания:";
        let p = super::header_sequence(s).unwrap();
        info!("тест строки ->{} статьи->{:?}", s, p.1);
    }

    #[test]
    fn test_items_numbers7()
    {
         logger::StructLogger::new_default();
        let s = "абзацем девятым и изложить его в следующей редакции:";
        let p = super::indent_sequence(s).unwrap();
        info!("тест строки ->{} статьи->{:?}", s, p.1);
    }
    #[test]
    fn test_items_numbers8()
    {
         logger::StructLogger::new_default();
        let s = "абзацы пятый и шестой изложить в следующей редакции:";
        let p = super::indent_sequence(s).unwrap();
        info!("тест строки ->{} статьи->{:?}", s, p.1);
    }

    
    
    #[test]
    fn test_chars_range()
    {
         logger::StructLogger::new_default();
        let p = super::get_chars_range("а", "ж");
        info!("срез подмножества символов ->{:?}", p);
    }


}