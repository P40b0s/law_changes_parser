use nom::
{
    branch::alt, bytes::complete::tag, sequence::{pair}, IResult, Parser
};

use crate::{change_path::TargetPath, error::ParserError, objects::number::Number};

use super::{paths, in_new_edition};
use super::space1;

///`абзац седьмой считать абзацем восьмым и изложить его в следующей редакции`
/// path это что туда нужно дополнить: частью 1
fn replace_indent(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remains, (repl ,_,_,_new_num,_)) = 
    ((
        paths,
        tag("считать"),
        space1,
        paths,
        in_new_edition
    )).parse(s)?;
    Ok((remains, repl))
}
///`абзац седьмой считать абзацем восьмым и изложить его в следующей редакции`
/// path это что туда нужно дополнить: частью 1
fn replace_name(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remains, (_,_,_)) = 
    ((
        tag("наименование"),
        space1,
        in_new_edition
    )).parse(s)?;
    Ok((remains, TargetPath::new()))
}

///(части №, статьи №, абзацы №) изложить в следующей редакции: "#,
/// так же встречались: части 2 и 3 статьи 23 изложить в следующей редакции:
fn items_with_number_new_edition(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remains, (p, _)) = ((paths, in_new_edition)).parse(s)?;
    Ok((remains, p))
}


///`1) статью 1 дополнить частью 1`
///(части №, статьи №, абзацы №) изложить в следующей редакции: "#,
pub fn replace_all(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let mut rpl = alt((
        items_with_number_new_edition,
        replace_indent,
        replace_name
    ));
    if let Ok(nmb) = pair(Number::parse, space1).parse(s)
    {
        rpl.parse(nmb.0)
    }
    else 
    {
        rpl.parse(s)
    }
}

#[cfg(test)]
mod tests
{
    use logger::info;
    use nom::
    {
        IResult,
        branch::{permutation, alt},
        bytes::complete::{tag, tag_no_case, take_until, take_till, is_a},
        character::complete::{digit0, char, alphanumeric1, space0, alpha1, one_of, anychar},
        sequence::{tuple, pair, separated_pair, terminated, preceded, delimited},
        multi::{many0, many1, many_till},
        combinator::{value, not, eof, map, opt}
    };

    #[test]
    fn test_check_add()
    {
        logger::StructLogger::new_default();
        let check_all_strings : Vec<&str> = vec![
            r#"абзац восьмой считать абзацем девятым и изложить его в следующей редакции:"#,
            r#"абзацы пятый и шестой изложить в следующей редакции:"#,
            r#"абзац второй изложить в следующей редакции:"#,
            "а) наименование изложить в следующей редакции:",
            "19) части 2 и 3 статьи 32 изложить в следующей редакции:"
        ];
        for s in check_all_strings
        {
            
            let p = super::replace_all(s).unwrap();
            info!("тест строки ->{} остаток токенов ->{}", p.0, serde_json::to_string_pretty(&p.1).unwrap());
        }
    }

    #[test]
    fn test_apply9()
    {
        logger::StructLogger::new_default();
        let s = r#"абзац восьмой считать абзацем девятым и изложить его в следующей редакции:"#;
        let p = super::replace_indent(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }

    #[test]
    fn test_apply8()
    {
        logger::StructLogger::new_default();
        let s = r#"абзацы пятый и шестой изложить в следующей редакции:"#;
        let p = super::items_with_number_new_edition(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
     #[test]
    fn test_replace_11()
    {
        logger::StructLogger::new_default();
        let s = r#"абзац второй изложить в следующей редакции:"#;
        let p = super::items_with_number_new_edition(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
      #[test]
    fn test_replace_12()
    {
        logger::StructLogger::new_default();
        let s = r#"дополнить абзацем третьим следующего содержания:"#;
        let p = super::items_with_number_new_edition(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }

    #[test]
    fn test_replace8()
    {
        logger::StructLogger::new_default();
        let s = r#"7) статью 13 изложить в следующей редакции:"#;
        let p = super::replace_all(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
    // #[test]
    // fn test_r()
    // {
    //     logger::StructLogger::new_default();
    //     let mut items = Vec::new();
    //     let item:u64 = 1000000000;
    //     //for i in 1..item
    //     for i in 1..((item/2)+1)
    //     {
    //         if item % i == 0
    //         {
    //             items.push(i);
    //         }
    //     }
    //     logger::info!("{:?}", items);
    // }

    // #[test]
    // fn test_r_2()
    // {
    //     logger::StructLogger::new_default();
    //     let mut items = 1..100;
    //     let mut sum = 0;
    //     for i in 1..=100
    //     {
    //         sum +=i
    //     }
    //     logger::debug!("{}", sum);
    // }

    
}