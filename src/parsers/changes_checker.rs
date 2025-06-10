use nom::
{
    branch::alt, bytes::complete::{is_a, tag, tag_no_case}, combinator::map, sequence::pair, IResult, Parser
};

use crate::{change_action::ChangeAction, change_path::TargetPath, error::ParserError, objects::{number::Number, remain_tokens::RemainTokens}};

use super::{ITEM_NUMBER, paths, next_is_content, in_new_edition};
use super::{space0, space1};



// fn list_number(s: &str) -> IResult<&str, &str, ParserError>
// {
//     let (remain, _) =  is_a(ITEM_NUMBER)(s)?;
//     let end = alt((tag(")"), tag("."))).parse(remain)?;
//     let ws = space0(end.0)?;
//     Ok((ws.0, ""))
// }

///`в подпункте 1 пункте 3 статьи 2^2`
/// абзац первый пункта 2 статьи 4
fn item_path(s: &str) -> IResult<&str, &str, ParserError>
{
    let v = alt((map((space0, tag_no_case("в"), space1), |_| ""), space0)).parse(s)?;
    let p = paths(v.0)?;
    Ok((p.0, ""))
}

///`Наименование изложить в следующей редакции...`
/// нечто связанное с наименованием
fn item_name(s: &str) -> IResult<&str, &str, ParserError>
{
    let num = Number::parse(s)?;
    let v = alt((map(pair(space0, alt((tag_no_case("наименование"), tag_no_case("в наименовании")))), |_| ""), space0)).parse(num.0)?;
    let (remain, _) = 
    (
        (
            space0,
            alt((map(paths, |m| ""), tag("после слов"), tag("слова"), tag("слово"), tag("изложить"))),
            space0
        )
    ).parse(v.0)?;
    Ok((remain, ""))
}
///`в подпункте 1 пункте 3 статьи 2^2`
/// абзац первый пункта 2 статьи 4
fn item_path2(s: &str) -> IResult<&str, &str, ParserError>
{
    let v = alt((map(pair(Number::parse, item_path), |_|""), item_path)).parse(s)?;
    Ok((v.0, ""))
}

///`в подпункте 1 пункте 3 статьи 2^2`
/// абзац первый пункта 2 статьи 4
fn apply(s: &str) -> IResult<&str, &str, ParserError>
{
    let app = ((
        
        space0,
        tag("дополнить"),
        space1,
        alt((
            tag_no_case("новыми"),
            tag_no_case("новым"),
            tag_no_case("словом"),
            tag_no_case("словами"),
            space0
        )),
        space0,
        alt((
            map(paths, |_m|""),
            map((
                tag("абзацем"),
                space1,
                next_is_content
            ), |_m|"")
        ))
    )).parse(s)?;
    Ok((app.0, ""))
}


///`в подпункте 1 пункте 3 статьи 2^2`
/// абзац первый пункта 2 статьи 4
fn apply2(s: &str) -> IResult<&str, &str, ParserError>
{
    let v = alt((
        map((Number::parse, paths, apply), |_|""),
        map((Number::parse, paths), |_|""),
        map(pair(Number::parse, apply), |_|""),
        map((paths, tag("считать"), space1, paths), |_|""),
        map(pair(paths, apply), |_|""),
        map((paths, in_new_edition), |_|""),
        apply
    )).parse(s)?;
    Ok((v.0, ""))
}

pub fn check_if_change(s: &str) -> bool
{
    let result = new_checker(s);
    if !result.0
    {
        if let Some(r) = result.1.as_ref()
        {
            logger::warn!("{}", r);
        }
        return false;
    }
    else
    {
        return true;
    }
    
}
pub fn check_if_change_result(s: &str) -> IResult<&str, &str, ParserError>
{
    alt((item_path2, apply2, item_name)).parse(s)
}
///функция возвращает false если строка не подходит под условия парсеров и выдает остаток нераспознаных токенов
///и возвращает true если полностью совпадает
pub fn new_checker(s: &str) -> (bool, Option<RemainTokens>)
{
    let mut errors: Vec<String> = vec![];
    if let Err(a) = super::only_path_definition(s) as  IResult<&str, TargetPath, ParserError>
    {
        a.map(|m| 
        {
            match &m
            {
                ParserError::NomError { input, code } =>
                {
                    //logger::warn!("{}", i);
                    errors.push(input.to_string());
                },
                _ => ()
            }
        });
    }
    else 
    {
        return (true, None);
    }
    //тут у нас дополнения с изменениями которые занимают несколько абзацев, их берем из полей changes
    if let Err(a) = super::apply_all(s) as  IResult<&str, (Option<TargetPath>, TargetPath), ParserError>
    {
        a.map(|m| 
            {
                match &m
                {
                    ParserError::NomError { input, code } =>
                    {
                        //logger::warn!("{}", i);
                        errors.push(input.to_string());
                    },
                    _ => ()
                }
            });
    }
    else 
    {
        return (true, None);
    }
    //тут изменения в пределах абзаца дополнить словами заменить словами итд.
    if let Err(w) = super::words::words_operations(s) as IResult<&str, (TargetPath, Vec<ChangeAction>), ParserError>
    {
        w.map(|m| 
            {
                match &m
                {
                    ParserError::NomError { input, code } =>
                    {
                        //logger::warn!("{}", i);
                        errors.push(input.to_string());
                    },
                    _ => ()
                }
            });
    }
    else 
    {
        return (true, None);
    }
    if let Err(r) = super::replace_all(s) as IResult<&str, TargetPath, ParserError>
    {
        r.map(|m| 
            {
                match &m
                {
                    ParserError::NomError { input, code } =>
                    {
                        //logger::warn!("{}", i);
                        errors.push(input.to_string());
                    },
                    _ => ()
                }
            });
    }
    else 
    {
        return (true, None);
    }
    if let Err(r) = item_name(s) as IResult<&str, &str, ParserError>
    {
        r.map(|m| 
            {
                match &m
                {
                    ParserError::NomError { input, code } =>
                    {
                        //logger::warn!("{}", i);
                        errors.push(input.to_string());
                    },
                    _ => ()
                }
            });
    }
    else 
    {
        return (true, None);
    }
    errors.sort_by(|a, b| a.len().partial_cmp(&b.len()).unwrap());
    let first = errors.first().unwrap();
    if first.split(" ").count() <= s.split(" ").count() - 2
    {
        return (false, Some(RemainTokens::new(s, first)));
    }
    else 
    {
        return (false, None);
    }
    // alt((
    //     map(super::only_path_definition, |m| ""),
    //     map(super::apply_all, |m| ""),
    //     map(super::words::words_operations, |m| ""),
    //     map(super::replace_all, |m| ""),
    // ))(s)
}


#[cfg(test)]
mod tests
{
    use logger::info;
    use nom::
    {
        branch::permutation, bytes::complete::{is_a, tag}, character::complete::anychar, multi::{many1, many_till}, sequence::pair, IResult, Parser
    };

    use crate::error::ParserError;

    #[test]
    fn test_checker_remains_on_error()
    {
        logger::StructLogger::new_default();
        //let s = r#"7) в абзаце первом пункта 2 статрьи 25 слово "(назначения)" исключить;"#;
        let s = r#"1. статью 5 в сотрудничество Российской Федерации в области семеноводства сельскохозяйственных растений осуществляется в соответствии с международными договорами Российской Федерации и законодательством Российской Федерации."#;
        let result = super::new_checker(s);
        info!("accept: {} -> {:?}", result.0, result.1);
    }

    #[test]
    fn test_checker_remains_on_error_2()
    {
        logger::StructLogger::new_default();
        //let s = r#"7) в абзаце первом пункта 2 статрьи 25 слово "(назначения)" исключить;"#;
        let s = r#"13) наименование главы 5 изложить в следующей редакции:"#;
        let result = super::new_checker(s);
        info!("{}", result.1.unwrap());
    }

    #[test]
    fn test_check_path()
    {
        logger::StructLogger::new_default();
        let check_path_strings : Vec<&str> = vec![
            r#"1) абзац девятый пункта 1 дополнить словами ", а также в случае предотвращения и пресечения преступлений с использованием сетей связи и средств связи";"#,
            r#"п) в пункте 16:"#,
            r#"п) в статье 2:"#,
            r#"а) в подпункте 71:"#,
            "1) в статье 2:",
            "1) в статье 2:",
            r#"7) в абзаце первом пункта 2 статьи 25 слово "(назначения)" исключить;"#,
            r#"в абзаце третьем слова "в области связи" заменить словами ", осуществляющим функции по контролю и надзору в сфере средств массовой информации, массовых коммуникаций, информационных технологий и связи,";"#,
            r#"м) пункт 12 признать утратившим силу; "#,
            r#"л) в пункте 11^1 слово "выданные" заменить словом "оформленные";"#,
            r#"Пункт 4 статьи 1 Федерального закона от 29 июля 2017 года № 245-ФЗ "О внесении изменений в Федеральный закон "О связи" (Собрание законодательства Российской Федерации, 2017, № 31, ст. 4794) изложить в следующей редакции:"#,
        ];

        for s in check_path_strings
        {
            info!("тест строки ->{}", s);
            let p = super::item_path2(s).unwrap();
            info!("остаток токенов ->{}",p.0);
        }
    }

    #[test]
    fn test_check_add()
    {
        logger::StructLogger::new_default();
        let check_add_strings : Vec<&str> = vec![
            "4) статью 22 дополнить пунктом 7 следующего содержания:",
            "з) дополнить частями 9 - 11 следующего содержания:",
            r#"2) дополнить пунктом 8 следующего содержания:"#,
            "дополнить новым абзацем девятым следующего содержания:",
            r#"дополнить абзацем следующего содержания:"#,
            r#"дополнить новыми абзацами седьмым - десятым и абзацами одиннадцатым и двенадцатым следующего содержания:"#,
            r#"г) дополнить пунктами 16 - 22 следующего содержания:"#,
            r#"7) главу 2 дополнить статьями 9^1 и 9^2 следующего содержания:"#,
            r#"абзац восьмой считать абзацем девятым и изложить его в следующей редакции:"#,
            r#"абзацы пятый и шестой изложить в следующей редакции:"#,
            r#"е) части 10, 11, 13 - 17 признать утратившими силу;"#,
        ];
        for s in check_add_strings
        {
            info!("тест строки ->{}", s);
            let p = super::apply2(s).unwrap();
            info!("остаток токенов ->{}", p.0);
        }
    }

    #[test]
    fn test_check_item_name()
    {
        logger::StructLogger::new_default();
        let check_path_strings : Vec<&str> = vec![
            r#"а) наименование после слов "Федеральная государственная" дополнить словом "географическая";"#,
            r#"9) наименование главы 6 изложить в следующей редакции:"#,
            "а) в наименовании слово \"(назначение)\" исключить;",
            "13) наименование главы 5 изложить в следующей редакции:",
            "а) наименование изложить в следующей редакции:"
        ];

        for s in check_path_strings
        {
            
            let p = super::item_name(s).unwrap();
            info!("тест строки ->{} остаток токенов ->{}", s, p.0);
        }
    }

    #[test]
    fn coplex_test()
    {
        logger::StructLogger::new_default();
        let check_all_strings : Vec<&str> = vec![
            r#"2) дополнить пунктом 8 следующего содержания:"#,
            r#"п) в пункте 16:"#,
            r#"7) в абзаце первом пункта 2 статьи 25 слово "(назначения)" исключить;"#,
            r#"в абзаце третьем слова "в области связи" заменить словами ", осуществляющим функции по контролю и надзору в сфере средств массовой информации, массовых коммуникаций, информационных технологий и связи,";"#,
            r#"9) наименование главы 6 изложить в следующей редакции:"#,
            r#"10) статьи 29 - 35 изложить в следующей редакции:"#,
            r#"м) пункт 12 признать утратившим силу; "#,
            r#"л) в пункте 11^1 слово "выданные" заменить словом "оформленные";"#,
            r#"дополнить абзацем следующего содержания:"#,
            r#"абзац восьмой считать абзацем девятым и изложить его в следующей редакции:"#,
            r#"дополнить новыми абзацами седьмым - десятым и абзацами одиннадцатым и двенадцатым следующего содержания:"#,
            r#"г) дополнить пунктами 16 - 22 следующего содержания:"#,
            r#"абзацы пятый и шестой изложить в следующей редакции: "#,
            r#"е) части 10, 11, 13 - 17 признать утратившими силу;"#,
            r#"7) главу 2 дополнить статьями 9^1 и 9^2 следующего содержания:"#,
            r#"а) наименование после слов "Федеральная государственная" дополнить словом "географическая";"#,
            "3. Часть 7 статьи 12, части 6 - 8, 10 и 11 статьи 20 настоящего Федерального закона вступают в силу с 1 сентября 2025 года.\".",
            r#"1) часть четвертую после слов "границу Российской Федерации" дополнить словами "и на складах временного хранения";"#
        ];
        for s in check_all_strings
        {
            
            let p = super::check_if_change_result(s);
            info!("тест строки ->{} остаток токенов ->{}", s, p.unwrap().0);
        }

    }


    fn quote_pair(s: &str) -> IResult<&str, &str, ParserError>
    {
        //let (remains, quotes) = many_till(take_until("\""), pair(is_a(ALPHA_NUMERIC), tag("\"")))(s)?;
        let (remains, quotes) = permutation::<&str, ParserError, _>((pair(tag("\""), is_a(super::super::ALPHA_NUMERIC)), many1(many_till(anychar, pair(is_a(super::super::ALPHA_NUMERIC), tag("\"")))))).parse(s)?;
        Ok((remains, ""))
    }
    //нихера не выходит с этими кавычками
    #[test]
    fn test_quoted()
    {
        logger::StructLogger::new_default();
        let test_str =  r#""О публично-правовой компании "Роскадастр" продолжается "тест второй кавычки" текст в кавычках и все"  потом будет идти какой то текст"#;
        let q  = quote_pair(test_str).unwrap();
        info!("тест строки ->{} остаток токенов ->{}", test_str, q.0);

    }

}