use nom::Parser;
use nom::
{
    IResult,
    branch::alt,
    bytes::complete::tag,
};
use crate::change_path::TargetPath;

use crate::objects::Number;
use crate::parsers::paths;
use crate::{error::ParserError};

use super::{ next_is_content, apply};
use super::space1;

///`1) статью 1 дополнить частью 1`
/// первый path это куда вносятся изменения: статью 22
/// второй path это что туда нужно дополнить: частью 1
fn number_target_path_apply_path(s: &str) -> IResult<&str, (TargetPath, TargetPath), ParserError>
{
    let (remains, (_,_, p1, _, p2, _)) = ((Number::parse, space1, paths, apply, paths, next_is_content)).parse(s)?;
    Ok((remains, (p1, p2)))
}
///`1) дополнить частью 1`
/// path это что туда нужно дополнить: частью 1
fn number_apply_items_with_number(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remains, (_, _, p, _)) = ((Number::parse, apply, paths, next_is_content)).parse(s)?;
    Ok((remains, p))
}
///`дополнить абзацем`
/// path это что туда нужно дополнить: частью 1
fn apply_indent(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remains, (_,_, _, _)) = ((apply, tag("абзацем"), space1, next_is_content)).parse(s)?;
    Ok((remains, TargetPath::new()))
}
///`дополнить абзацем`
/// path это что туда нужно дополнить: частью 1
fn apply_indent_with_number(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remains, (_,_, path, _, _)) = ((apply, tag("абзацем"), paths, space1, next_is_content)).parse(s)?;
    Ok((remains, path))
}

///`дополнить  (частью №, статьей №, абзацем №) `
fn apply_items_with_number(s: &str) -> IResult<&str, TargetPath, ParserError>
{
    let (remains, (_,p, _)) = ((apply, paths, next_is_content)).parse(s)?;
    Ok((remains, p))
}
///всевозможные дополнения<br>
/// `"4) статью 22 дополнить пунктом 7 следующего содержания:",`<br>
/// `"з) дополнить частями 9 - 11 следующего содержания:",`<br>
/// первый результ это куда добавляем изменения, второй результ это что добавляем: дополнить пунктом 1:
/// итд
pub fn apply_all(s: &str) -> IResult<&str, (Option<TargetPath>, TargetPath), ParserError>
{
    let result = alt((
        apply_items_with_number,
        apply_indent,
        apply_indent_with_number,
        number_apply_items_with_number,
    )).parse(s) as IResult<&str, TargetPath, ParserError>;
    if let Ok(r) = result
    {
        Ok((r.0, (None, r.1)))
    }
    else
    {
        //думать что делать с путем ЧЕМ дополнить!
        let long = number_target_path_apply_path(s)?;
        Ok((long.0, (Some(long.1.0), long.1.1)))
    }
}



#[cfg(test)]
mod tests
{
    use logger::info;

    #[test]
    fn test_check_add()
    {
        logger::StructLogger::new_default();
        let check_all_strings : Vec<&str> = vec![
            "4) статью 22 дополнить пунктом 7 следующего содержания:",
            "з) дополнить частями 9 - 11 следующего содержания:",
            r#"2) дополнить пунктом 8 следующего содержания:"#,
            "дополнить новым абзацем девятым следующего содержания:",
            r#"дополнить абзацем следующего содержания:"#,
            r#"дополнить абзацем третьим следующего содержания:"#,
            r#"дополнить новыми абзацами седьмым - десятым и абзацами одиннадцатым и двенадцатым следующего содержания:"#,
            r#"г) дополнить пунктами 16 - 22 следующего содержания:"#,
            r#"7) главу 2 дополнить статьями 9^1 и 9^2 следующего содержания:"#,
        ];
        for s in check_all_strings
        {
            
            let p = super::apply_all(s).unwrap();
            info!("тест строки ->{} остаток токенов ->{}| данные->{:?}", s, p.0, p.1.1);
        }
    }

    #[test]
    fn test_apply()
    {
        logger::StructLogger::new_default();
        let s = "4) статью 22 дополнить пунктом 7 следующего содержания:";
        let p = super::number_target_path_apply_path(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{} путь->{:?}/{:?}", s, p.0, p.1.0, p.1.1);
    }

    #[test]
    fn test_apply2()
    {
        logger::StructLogger::new_default();
        let s = "з) дополнить частями 9 - 11 следующего содержания:";
        let p = super::number_apply_items_with_number(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{} перечисление->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_apply2_1()
    {
        logger::StructLogger::new_default();
        let s = r#"г) дополнить пунктами 16 - 22 следующего содержания:"#;
        let p = super::number_apply_items_with_number(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{} перечисление->{:?}", s, p.0, p.1);
    }
  
    #[test]
    fn test_apply3()
    {
        logger::StructLogger::new_default();
        let s = r#"2) дополнить пунктом 8 следующего содержания:"#;
        let p = super::number_apply_items_with_number(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{} путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_apply4()
    {
        logger::StructLogger::new_default();
        let s =  "дополнить новым абзацем девятым следующего содержания:";
        let p = super::apply_items_with_number(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{} путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_apply5()
    {
        logger::StructLogger::new_default();
        let s =  "дополнить абзацем следующего содержания:";
        let p = super::apply_indent(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{} путь->{:?}", s, p.0, p.1);
    }
     #[test]
    fn test_apply8()
    {
        logger::StructLogger::new_default();
        let s =  "дополнить абзацем третьим следующего содержания:";
        let p = super::apply_indent(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{} путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_apply5_1()
    {
        logger::StructLogger::new_default();
        let s =  "дополнить предложениями следующего содержания: \"Публично-правовая компания проводит экспертизу таких отчета и каталога на их соответствие установленным на основании настоящего Федерального закона требованиям в порядке, предусмотренном частью 10 настоящей статьи. Отчет о создании геодезической сети специального назначения и каталог координат геодезических пунктов указанной сети включаются в федеральный фонд пространственных данных при наличии положительного заключения указанной экспертизы.\";";
        let p = super::apply_indent(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{} путь->{:?}", s, p.0, p.1);
    }

    
    #[test]
    fn test_apply6()
    {
        logger::StructLogger::new_default();
        let s =   r#"дополнить новыми абзацами седьмым - десятым и абзацами одиннадцатым и двенадцатым следующего содержания:"#;
        let p = super::apply_items_with_number(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}", s, p.0, p.1);
    }
    #[test]
    fn test_apply7()
    {
        logger::StructLogger::new_default();
        let s =  r#"7) главу 2 дополнить статьями 9^1 и 9^2 следующего содержания:"#;
        let p = super::number_target_path_apply_path(s).unwrap();
        info!("тест строки ->{} остаток токенов ->{}| путь->{:?}/{:?}", s, p.0, p.1.0, p.1.1);
    }

}