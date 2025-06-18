use nom::
{
    branch::{alt, permutation}, bytes::complete::{is_a, tag, take_until}, character::complete::anychar, combinator::{eof, map}, multi::{many1, many_till}, sequence::{delimited, pair, preceded, separated_pair, tuple}, IResult, Parser
};
use crate::{change_action::ChangeAction, change_path::TargetPath, error::ParserError, objects::number::Number};

use super::{chars::end_indent_char, lost_power, paths, space1, tags::{next_is_content_not_eof, in_new_edition_not_eof}, consts, space0};

enum LocalOperation
{
    Replace,
    Apply,
    Exclude
}

///(после)? (слова | слово | слов)
fn searched_words(s: &str) -> IResult<&str, &str, ParserError>
{
    let mut last = space0(s)?;
    let after_tok: IResult<&str, &str, ParserError> = preceded(tag("после"), tag(" ")).parse(last.0);
    if after_tok.is_ok()
    {
        last = (after_tok.unwrap().0, "");
    }
    let words = alt((tag("слово"), tag("слова"), tag("слов"))).parse(last.0)?;
    let ws = space0(words.0)?;
    Ok((ws.0, words.1))
}
///`"текст в кавычках"`
fn quoted_text(s: &str) -> IResult<&str, &str, ParserError>
{
    //возможно ошибка что error не реализует clone? но это нереально, thiserror пробрасывает и другие типы ошибок которые не реализуют clone
    let (remains, (_, searched_word, _)) = permutation::<&str, ParserError, _>((tag("\""), take_until("\""), tag("\""))).parse(s)?;
    Ok((remains, searched_word))
}
fn quote_pair(s: &str) -> IResult<&str, &str, ParserError>
{
    //let (remains, quotes) = many_till(take_until("\""), pair(is_a(ALPHA_NUMERIC), tag("\"")))(s)?;
    let (remains, quotes) = 
    ((
        pair(tag("\""), is_a(super::ALPHA_NUMERIC)),
        many1(many_till(
            anychar,
            pair(is_a(super::ALPHA_NUMERIC), tag("\""))
        ))
    )).parse(s)?;
    Ok((remains, ""))
}
///`исключить;`
fn exclude(s: &str) -> IResult<&str, LocalOperation, ParserError>
{
    let ws = space0(s)?;
    let exclude = tag("исключить")(ws.0)?;
    let last = alt((tag(";"), tag(","), eof, space1)).parse(exclude.0)?;
    Ok((last.0, LocalOperation::Exclude))
    
}

///`заменить словами`
fn change(s: &str) -> IResult<&str, LocalOperation, ParserError>
{
    let ws = space0(s)?;
    let (remains, (_, _)) = separated_pair(tag("заменить"), space1, alt((tag("словами"), tag("словом")))).parse(ws.0)?;
    let ws = space0(remains)?;
    Ok((ws.0, LocalOperation::Replace))
}
///`дополнить словами`
fn add(s: &str) -> IResult<&str, LocalOperation, ParserError>
{
    let ws = space0(s)?;
    let (remains, (_, _)) = separated_pair(tag("дополнить"), space1, alt((tag("словами"), tag("словом")))).parse(ws.0)?;
    let ws = space0(remains)?;
    Ok((ws.0, LocalOperation::Apply))
}

fn all_operations(s: &str) -> IResult<&str, LocalOperation, ParserError>
{
    alt((add, change, exclude)).parse(s)
}

///`слово \"(назначения)\" и слова \"эти слова исключить\" исключить;`
fn exclude_words(s: &str) -> IResult<&str, Vec<ChangeAction>, ParserError>
{
    let mut actions: Vec<ChangeAction> = vec![];
    //let (remain, words) = many1(permutation((searched_words, quoted_text, alt((tag(" и "), tag(", "), tag(" "))))))(s)?;
    let (remain, words) = 
    many1(
    (
                        
                searched_words,
                quoted_text,
                space0,
                alt((tag(", "), tag("и "), space0, eof))
                        
            )
    ).parse(s)?;
    let exclude_cmd = exclude(remain)?;
    for (_, quote, _, _) in words
    {
        let action = ChangeAction::ExcludeWords(quote.to_owned());
        actions.push(action);
    }
    Ok((exclude_cmd.0, actions))
}

///`дополнить предложением следующего содержания: "новое предложение"`
fn add_sentence(s: &str) -> IResult<&str, Vec<ChangeAction>, ParserError>
{
    let ws = space0(s)?;
    let mut actions: Vec<ChangeAction> = vec![];
    let (remain, (_,_,_,_,_,_, sentence)) = 
    
    (
        tag("дополнить"),
        space1,
        alt((tag("предложениями"), tag("предложением"))),
        space1,
        next_is_content_not_eof,
        space1,
        quoted_text,
    ).parse(ws.0)?;
    let action = ChangeAction::ApplySentence(sentence.to_owned());
    actions.push(action);
    Ok((remain, actions))
}
///`первое предложение изложить в следующей редакции: "новое предложение"`
fn replace_sentence(s: &str) -> IResult<&str, (TargetPath, Vec<ChangeAction>), ParserError>
{
    let prefix = map(delimited(space1, tag("в"), space1), |_m| "");
    let mut actions: Vec<ChangeAction> = vec![];
    let (remain, (_,_,path,number,_,_,_,_,_, sentence, _)) = 
    
    (
        Number::parse,
        alt((prefix, space1)),
        paths,
        alt((
            tag("первое"),
            tag("второе"),
            tag("третье"),
            tag("четвёртое"),
            tag("пятое"),
            tag("шестое"),
            tag("седьмое"),
            tag("восьмое"),
            tag("девятое"),
            tag("десятое"),
            tag("одиннадцатое"),
            tag("двенадцатое"),
            tag("тринадцатое"),
            tag("четырнадцатое"),
            tag("пятнадцатое"),
            tag("шестнадцатое"),
            )),
        space1,
        tag("предложение"),
        space1,
        in_new_edition_not_eof,
        space1,
        quoted_text,
        end_indent_char
    ).parse(s)?;
    //тут точно есть до 16 числа пока что
    let number = consts::INDENT_NUMBERS.get(&number).unwrap();
    let action = ChangeAction::ReplaceSentence { number: *number as u32, text: sentence.to_owned() };
    actions.push(action);
    Ok((remain, (path, actions)))
}

///`слова "(назначения)" заменить словами "эти слова исключить";`
fn add_or_replace_words(s: &str) -> IResult<&str, Vec<ChangeAction>, ParserError>
{
    let mut actions: Vec<ChangeAction> = vec![];
    //let (remain, words) = many1(permutation((searched_words, quoted_text, change, quoted_text, alt((tag(" и "), tag(", "), tag(" "), tag(";"), eof)))))(s)?;
    let (remain, words) = 
    many1(
    (
                searched_words,
                quoted_text,
                all_operations,
                quoted_text,
                space0,
                alt((tag(", "), tag("и "), tag(";"), tag("."), space0, eof))
    )).parse(s)?;
    for (_, old, oper, new, _, _) in words
    {
        match  oper
        {
            LocalOperation::Apply =>
            {
                let action = ChangeAction::AddWords {after: Some(old.to_owned()), words: new.to_owned()};
                actions.push(action);
            },
            LocalOperation::Replace =>
            {
                let action = ChangeAction::ReplaceWords { old_words: old.to_owned(), new_words: new.to_owned() };
                actions.push(action);
            }
            _ => ()
        }
    }
    //let end = end_tag_terminate(remain);
    Ok((remain, actions))
}

///`дополнить словами "дополняемые слова";`
fn add_words_1(s: &str) -> IResult<&str, Vec<ChangeAction>, ParserError>
{
    let mut actions: Vec<ChangeAction> = vec![];
    //let (remain, (_, new, _)) = permutation((add, quoted_text, alt((tag(" "), tag(";"), eof))))(s)?;
    let (remain, (_,txt,_)) = 
    (
        add,
        quoted_text,
        alt((tag(", "), tag("и "), tag(";"), space0, eof))
    ).parse(s)?;
    let action = ChangeAction::AddWords { after: None, words: txt.to_owned()};
    actions.push(action);
    //let end = end_tag_terminate(remain);
    Ok((remain, actions))
}

///`слово "(назначения)" и слова "эти слова исключить" исключить;`<br>
///или<br>
///`слова "какие слова ищем" заменить словами "слова для замены"`
fn change_or_add_or_exclude_words(s: &str) -> IResult<&str, Vec<ChangeAction>, ParserError>
{
    let (remain, result) = many1(
        alt((add_or_replace_words, add_words_1, exclude_words, add_sentence))
    ).parse(s)?;
    Ok((remain, result.into_iter().flat_map(|m| m).collect::<Vec<ChangeAction>>()))
    //alt((change_words, add_words, add_words2, exclude_words))(s)
}

///"а) наименование после слов \"Федеральная государственная\" дополнить словом \"географическая\";"
fn change_name(s: &str) -> IResult<&str, Vec<ChangeAction>, ParserError>
{
    let name = 
    (
        Number::parse,
        space1,
        tag("наименование"),
        space1,
        add_or_replace_words
    ).parse(s)?;
    Ok((name.0, vec![ChangeAction::HeaderNameOperations(name.1.4)]))
}

///`слово "(назначения)" и слова "эти слова исключить" исключить;`<br>
///или<br>
///`слова "какие слова ищем" заменить словами "слова для замены"`
pub fn words_operations(s: &str) -> IResult<&str, (TargetPath, Vec<ChangeAction>), ParserError>
{
    //1) в? пункте 1...
    let prefix = map(delimited(space1, tag("в"), space1), |_m| "");
    if let Ok((remains, (_,_, path, op))) = 
        ((Number::parse, alt((prefix, space1)), paths, change_or_add_or_exclude_words)).parse(s)
    {
        Ok((remains, (path, op)))
    }
    //в пункте 1 ....
    else if let Ok((remains, (_, paths, actions))) = 
    ((
            map(pair(tag("в"),space1), |_m| ""),
            paths,
            change_or_add_or_exclude_words
    )).parse(s)
    {
        Ok((remains, (paths, actions)))
    }
    //слова "..." заменить словами "..."
    else if let Ok((remains,(_, actions))) = separated_pair(Number::parse, space1, change_or_add_or_exclude_words).parse(s)
    {
        Ok((remains, (TargetPath::new(), actions)))
    }
    //абзац первый ....
    else if let Ok(without_number) = ((paths, change_or_add_or_exclude_words)).parse(s)
    {
        Ok((without_number.0, (without_number.1.0, without_number.1.1)))
    }
    else if let Ok(lp) = number_items_with_number_lost_power(s)
    {
        let mut actions: Vec<ChangeAction> = vec![];
        let items = lp.1.get_numbers();
        for n in items
        {
            actions.push(ChangeAction::LostPower(n))
        }
        Ok((lp.0, (TargetPath::new(), actions)))
    }
    else if let Ok(ex) = number_items_with_number_exclude(s)
    {
        let mut actions: Vec<ChangeAction> = vec![];
        let items = ex.1.get_numbers();
        for n in items
        {
            actions.push(ChangeAction::Exclude(n))
        }
        Ok((ex.0, (TargetPath::new(), actions)))
    }
    else if let Ok(repl_sent) = replace_sentence(s)
    {
        Ok(repl_sent)
    }
    else 
    {
        let name = change_name(s)?;
        Ok((name.0, (TargetPath::new(), name.1)))
    }
}

///1)(части №, статьи №, абзацы №) признать утратившими силу;
fn number_items_with_number_lost_power(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remains, (_,_,p,_)) = ((
        Number::parse,
        space1,
        paths,
        lost_power
    )).parse(s)?;
    Ok((remains, p))
}
fn number_items_with_number_exclude(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remains, (_,_,p,_)) = ((
        Number::parse,
        space1,
        paths,
        exclude
    )).parse(s)?;
    Ok((remains, p))
}

#[cfg(test)]
mod tests
{
    use std::collections::HashMap;
    use logger::info;
    use nom::{IResult, sequence::{tuple, delimited}, combinator::map, branch::alt};
    use serde::Serialize;
    use crate::{parsers::{words::{searched_words, change_or_add_or_exclude_words}, space1,  paths}, };
    use super::exclude_words;

    #[test]
    fn test_lost_power()
    {
        logger::StructLogger::new_default();
        let test = "1) статью 2^2 признать утратившей силу;";
        let t1 = super::number_items_with_number_lost_power(test).unwrap();
        write_result(t1, test);
    }

    #[test]
    fn test_lost_power2()
    {
        logger::StructLogger::new_default();
        let s = r#"е) части 10, 11, 13 - 17 признать утратившими силу;"#;
        let p = super::number_items_with_number_lost_power(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_lost_power3()
    {
        logger::StructLogger::new_default();
        let s = r#"е) части 10, 11, 13 - 17 признать утратившими силу;"#;
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
    #[test]
    fn test_lost_power4()
    {
        logger::StructLogger::new_default();
        let s = r#"а) части 2 - 2^4 признать утратившими силу;"#;
        let mut p = super::number_items_with_number_lost_power(s).unwrap();
        p.1.sort();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
    #[test]
    fn test_sentence()
    {
        logger::StructLogger::new_default();
        let s = r#"дополнить предложениями следующего содержания: "Публично-правовая компания проводит экспертизу таких отчета и каталога на их соответствие установленным на основании настоящего Федерального закона требованиям в порядке, предусмотренном частью 10 настоящей статьи. Отчет о создании геодезической сети специального назначения и каталог координат геодезических пунктов указанной сети включаются в федеральный фонд пространственных данных при наличии положительного заключения указанной экспертизы.";"#;
        let p = super::add_sentence(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
    #[test]
    fn test_sentence2()
    {
        logger::StructLogger::new_default();
        let s = r#"а) в части 1 первое предложение изложить в следующей редакции: "Федеральная государственная информационная система в области семеноводства сельскохозяйственных растений создается в целях реализации полномочий в области семеноводства сельскохозяйственных растений федеральным органом исполнительной власти, осуществляющим функции по выработке государственной политики и нормативно-правовому регулированию в области семеноводства сельскохозяйственных растений, обеспечения прослеживаемости оборота семян сельскохозяйственных растений, учета семян сельскохозяйственных растений при их производстве, хранении, транспортировке, реализации, включая оказание услуг в области семеноводства, при осуществлении сделок с семенами сельскохозяйственных растений, а также в целях анализа, обработки представленных в эту систему сведений и информации и контроля за достоверностью таких сведений и информации.";"#;
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }

    #[test]
    fn test_words_operation7()
    {
        logger::StructLogger::new_default();
        let s = r#"в пункте 1 слова "и использование" исключить, дополнить словами ", а также ввоз семян сельскохозяйственных растений в Российскую Федерацию и вывоз семян сельскохозяйственных растений из Российской Федерации";"#;
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
    
    #[test]
    fn test_sentence3()
    {
        logger::StructLogger::new_default();
        let s = r#"а) в части 7 слова "пунктов указанной сети" заменить словами "геодезических пунктов указанной сети", дополнить предложениями следующего содержания: "Публично-правовая компания проводит экспертизу таких отчета и каталога на их соответствие установленным на основании настоящего Федерального закона требованиям в порядке, предусмотренном частью 10 настоящей статьи. Отчет о создании геодезической сети специального назначения и каталог координат геодезических пунктов указанной сети включаются в федеральный фонд пространственных данных при наличии положительного заключения указанной экспертизы.";"#;
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
    
    #[test]
    fn test_words_operation1()
    {
        logger::StructLogger::new_default();
        let s = r#"3) в подпункте 5 статьи 395 после слов "в указанный период в соответствии с" дополнить словами "его целевым назначением и", слова "муниципальном образовании и по специальности" заменить словами "муниципальном образовании, определенном законом субъекта Российской Федерации, и по профессии, специальности";"#;
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }

    #[test]
    fn test_words_operation2()
    {
        logger::StructLogger::new_default();
        let s = r#"а) в части 2 слово "вправе" заменить словом "обязаны";"#;
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
    #[test]
    fn test_words_operation3()
    {
        logger::StructLogger::new_default();
        let s = r#"а) часть 1 после слов "а также по созданию" дополнить словами ", модернизации и (или) обследованию";"#;
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }

    #[test]
    fn test_words_operation4()
    {
        logger::StructLogger::new_default();
        let s = r#"абзац первый после слова "государственная" дополнить словом "географическая";"#;
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }

    #[test]
    fn test_words_operation5()
    {
        logger::StructLogger::new_default();
        let s = r#"а) в части 1 слова "и использования" исключить, после слов ", если на" дополнить словом "такие";"#;
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
    #[test]
    fn test_words_operation6()
    {
        logger::StructLogger::new_default();
        let s =  "а) слова \"Сорта и гибриды\" заменить словами \"1. Сорта и гибриды\";";
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }
    #[test]
    fn test_words_operation8()
    {
        logger::StructLogger::new_default();
        let s = r#"4) в абзаце пятом пункта 6 статьи 36^12-1 слово "квалифицированного" исключить, слова "единой системы идентификации и аутентификации" заменить словами "информационной системы головного удостоверяющего центра, функции которого осуществляет уполномоченный федеральный орган исполнительной власти";"#;
        let p = super::words_operations(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{}", s, p.0, serde_json::to_string_pretty(&p.1).unwrap());
    }

    #[test]
    fn test_repeat_exclude_words_fn()
    {
        logger::StructLogger::new_default();
        let test_many_exclude = " слово \"(назначения)\" и слово \"хз что\" и слово \"неразборчиво написано\" исключить;";
        let test_one_exclude = " слово \"(назначения)\" исключить;";
        let test_many_exclude_comma = " слово \"(назначения)\", слово \"хз что\" и слово \"неразборчиво написано\" исключить;";
        let t1 = exclude_words(test_many_exclude).unwrap();
        write_result(t1, test_many_exclude);
        let t2 = exclude_words(test_one_exclude).unwrap();
        write_result(t2, test_one_exclude);
        let t3 = exclude_words(test_many_exclude_comma).unwrap();
        write_result(t3, test_many_exclude_comma);
    }
    // #[test]
    // fn test_repeat_change_words_fn()
    // {
    //     logger::StructLogger::initialize_logger();
    //     let test_one_change = "слова \"в области связи\" заменить словами \", осуществляющий функции по контролю и надзору в сфере средств массовой информации, массовых коммуникаций, информационных технологий и связи,\"";
    //     let test_many_change = "слова \"в области связи\" заменить словами \", осуществляющий функции по контролю и надзору в сфере средств массовой информации, массовых коммуникаций, информационных технологий и связи,\" и слова \"эти слова\" заменить словами \"меняем на эти слова\"";
    //     let test_many_change_comma = "слова \"в области связи\" заменить словами \", осуществляющий функции по контролю и надзору в сфере средств массовой информации, массовых коммуникаций, информационных технологий и связи,\", слова \"эти слова\" заменить словами \"меняем на эти слова\"";
    //     let t1 = change_words(test_one_change).unwrap();
    //     write_result(t1, test_one_change);
    //     let t2 = change_words(test_many_change).unwrap();
    //     write_result(t2, test_many_change);
    //     let t3 = change_words(test_many_change_comma).unwrap();
    //     write_result(t3, test_many_change_comma);
    // }

    #[test]
    fn test_change_or_exclude_words()
    {
        logger::StructLogger::new_default();
        let test_one_change = "слова \"в области связи\" заменить словами \", осуществляющий функции по контролю и надзору в сфере средств массовой информации, массовых коммуникаций, информационных технологий и связи,\"";
        let test_many_change = "слова \"в области связи\" заменить словами \", осуществляющий функции по контролю и надзору в сфере средств массовой информации, массовых коммуникаций, информационных технологий и связи,\" и слова \"эти слова\" заменить словами \"меняем на эти слова\"";
        let test_many_change_comma = r#"3) в подпункте 5 статьи 395 после слов "в указанный период в соответствии с" дополнить словами "его целевым назначением и", слова "муниципальном образовании и по специальности" заменить словами "муниципальном образовании, определенном законом субъекта Российской Федерации, и по профессии, специальности";"#;
        let t1 = change_or_add_or_exclude_words(test_one_change).unwrap();
        write_result(t1, test_one_change);
        let t2 = change_or_add_or_exclude_words(test_many_change).unwrap();
        write_result(t2, test_many_change);
        let t3 = change_or_add_or_exclude_words(test_many_change_comma).unwrap();
        write_result(t3, test_many_change_comma);
        let test_many_exclude = " слово \"(назначения)\" и слово \"хз что\" и слово \"неразборчиво написано\" исключить;";
        let test_one_exclude = " слово \"(назначения)\" исключить;";
        let test_many_exclude_comma = " слово \"(назначения)\", слово \"хз что\" и слово \"неразборчиво написано\" исключить;";
        let t1 = change_or_add_or_exclude_words(test_many_exclude).unwrap();
        write_result(t1, test_many_exclude);
        let t2 = change_or_add_or_exclude_words(test_one_exclude).unwrap();
        write_result(t2, test_one_exclude);
        let t3 = change_or_add_or_exclude_words(test_many_exclude_comma).unwrap();
        write_result(t3, test_many_exclude_comma);
        let test1 = "после слова \"власти\" дополнить словами \", осуществляющим функции\", после слов \"заявленным в абонентских договорах,\" дополнить словами \"в случае прекращения деятельности абонентом - юридическим лицом (за исключением случаев реорганизации юридического лица) либо прекращения физическим лицом деятельности в качестве индивидуального предпринимателя, являющегося абонентом,\";";
        let test2 = "после слов \"заявленным в абонентских договорах,\" дополнить словами \"в случае прекращения деятельности абонентом - юридическим лицом (за исключением случаев реорганизации юридического лица) либо прекращения физическим лицом деятельности в качестве индивидуального предпринимателя, являющегося абонентом,\";";
        let test3 = "после слов \"статьи 46\" дополнить словами \", пунктом 2 статьи 56^2\";";
        let test4 = "дополнить словами \", пунктом 2 статьи 56^2\";";
        let t1 = change_or_add_or_exclude_words(test1).unwrap();
        write_result(t1, test1);
        let t2 = change_or_add_or_exclude_words(test2).unwrap();
        write_result(t2, test2);
        let t3 = change_or_add_or_exclude_words(test3).unwrap();
        write_result(t3, test3);
        let t4 = change_or_add_or_exclude_words(test4).unwrap();
        write_result(t4, test4);

        let test_full = r#"слова "по специальностям," заменить словами "по профессиям, специальностям,", слово "лет;" заменить словами "лет. Законом субъекта Российской Федерации может быть предусмотрено, что такие граждане должны состоять на учете в качестве нуждающихся в жилых помещениях или иметь основания для постановки на данный учет, а также требование об отсутствии у таких граждан права собственности на иные земельные участки, предоставленные для индивидуального жилищного строительства или ведения личного подсобного хозяйства в данном муниципальном образовании;""#;
        let tf = change_or_add_or_exclude_words(test_full).unwrap();
        write_result(tf, test_full);
    }
    
    fn write_result<T: Serialize, P: AsRef<str>>(res: (P,T), txt: P)
    {
        let json = serde_json::to_string_pretty(&res.1).unwrap();
        info!("По тексту |{}| найдены задачи {} остаток токенов-> |{}|", txt.as_ref(), json, res.0.as_ref());
    }
    
}