use nom::
{
    branch::alt, bytes::complete::{is_a, tag, tag_no_case}, combinator::map, sequence::pair, IResult, Parser
};

use crate::{change_action::ChangeAction, change_path::{ChangePath, TargetPath}, error::ParserError, objects::{number::Number, remain_tokens::RemainTokens}, parsers::{paths, space0}};

struct Changes
{
    target_path: TargetPath,
    changes: Vec<ChangeAction>,
    action_after_path: Option<TargetPath>
}


pub fn new_checker(s: &str, all_paths: &mut Vec<ChangePath>) -> Option<RemainTokens>
{
    //только путь, дальше идет уточнение
    //генерируем глобальный путь для всех изменений
    let only_path_definitions_directive: IResult<&str, TargetPath, ParserError> = super::only_path_definition(s);
    if let Ok((remains, path)) = only_path_definitions_directive
    {
        let current_paths = path.get_paths();
        if let Some(all_last) = all_paths.last()
        {
            if let Some(cp_last) = current_paths.last()
            {
                if cp_last.get_lvl() <= all_last.get_lvl()
                {
                    all_paths.clear();
                    all_paths.extend(current_paths.clone());
                }
                else 
                {
                    all_paths.push(cp_last.clone());
                }
            }
        }
        else 
        {
            if let Some(cp_last) = current_paths.last()
            {
                all_paths.push(cp_last.clone()); 
            }
        }
        return Some(RemainTokens::new(s, remains));
    }
    //тут у нас дополнения с изменениями которые занимают несколько абзацев, их берем из полей changes
    let apply_directive: IResult<&str, (Option<TargetPath>, TargetPath), ParserError> =  super::apply_all(s);
    if let Ok((remains, (after, target))) = apply_directive
    {
        //TODO надо подумать когда добавлять глобальный путь а когда нет, например если в локальном пути уже есть уровень статьи и выше то не добавляем
        if !all_paths.is_empty()
        {
            let mut tp = target;
            tp.sort();
            tp.insert_paths(&all_paths);
            logger::debug!("apply directive: {:?}", tp)
        }
        return Some(RemainTokens::new(s, remains));
    }
    //тут изменения в пределах абзаца дополнить словами заменить словами итд.
    let words_directive: IResult<&str, (TargetPath, Vec<ChangeAction>), ParserError> = super::words::words_operations(s);
    if let Ok((remains, (path, actions))) = words_directive
    {
        if !all_paths.is_empty()
        {
            let mut tp = path;
            tp.sort();
            //сортирока не сработала
            //words directive: TargetPath([Header { number: Number { number: "242", va_number: None, postfix: None }, header_type: Article }, Indent(1), Item { number: Number { number: "3", va_number: None, postfix: None }, item_type: Item }])
            tp.insert_paths(&all_paths);
            logger::debug!("words directive: {:?}", tp)
        }
        return Some(RemainTokens::new(s, remains));
    }
    //замена чего либо (c нового абзаца и далее)
    let replace_directive: IResult<&str, TargetPath, ParserError> = super::replace_all(s);
    if let Ok((remains, path)) = replace_directive
    {
         if !all_paths.is_empty()
        {
            let mut tp = path;
            tp.sort();
            tp.insert_paths(&all_paths);
            logger::debug!("replace directive: {:?}", tp)
        }
        return Some(RemainTokens::new(s, remains));
    }
    //хз не помню для чего это
    let item_name_directive: IResult<&str, &str, ParserError> = item_name(s);
    if let Ok((remains, xz)) = item_name_directive
    {
        return Some(RemainTokens::new(s, remains));
    }
    //ни один кейс не прошел значит это изменение на отдельной строке
    return None;
    
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
#[cfg(test)]
mod tests
{
    use crate::parsers::changes_parser::TEST_DATA;

    #[test]
    fn test_changes_parser()
    {
        logger::StructLogger::new_default();
        let mut all_paths = Vec::new();
        for ln in TEST_DATA.lines()
        {
            let result = super::new_checker(ln, &mut all_paths);
            if let Some(r) = result
            {
                logger::debug!("input string `{}` remains tokens `{:?}`", ln, r);
                logger::debug!("current global paths {:?}`", all_paths);
            }
            
        }
    }
}

const TEST_DATA: &'static str = r#"1) пункт 5 статьи 7 изложить в следующей редакции:

"5. Если иное не предусмотрено федеральными законами или договором, а также нормативным актом Банка России в отношении операций по зачислению ценных бумаг, депозитарий не вправе совершать операции с ценными бумагами депонента иначе как по поручению депонента. Основания, порядок, срок и условия проведения депозитарием операций по зачислению ценных бумаг на счет депо без поручения депонента, а также основания отказа депозитария в их проведении устанавливаются нормативным актом Банка России. Депозитарий вправе отказать в исполнении поручений депонента на проведение операций по счетам депо в случае наличия задолженности депонента по оплате услуг депозитария, если иное не предусмотрено депозитарным договором.";
2) в статье 20:
а) в пункте 2:
абзац второй изложить в следующей редакции:
"Документы для государственной регистрации выпуска (дополнительного выпуска) эмиссионных ценных бумаг представляются в Банк России в электронной форме (в форме электронных документов). Банк России взаимодействует с лицами, представляющими указанные документы, посредством информационных ресурсов, размещенных на официальном сайте Банка России в информационно-телекоммуникационной сети "Интернет", в том числе путем предоставления таким лицам доступа к личному кабинету.";
дополнить абзацем третьим следующего содержания:
"Документы для регистрации выпуска (дополнительного выпуска) эмиссионных ценных бумаг могут быть представлены в регистрирующие организации, указанные в статье 201 настоящего Федерального закона, в электронной форме (в форме электронных документов). Регистрирующие организации взаимодействуют с лицами, представляющими указанные документы, посредством информационных ресурсов, размещенных на официальных сайтах регистрирующих организаций в информационно-телекоммуникационной сети "Интернет", в том числе путем предоставления таким лицам доступа к личному кабинету.";
б) в пункте 3 второе предложение изложить в следующей редакции: "При этом указанные документы представляются в порядке, предусмотренном абзацем вторым пункта 2 настоящей статьи, и могут быть представлены без их утверждения уполномоченным органом эмитента.";
3) в статье 242:
а) пункт 1 после слов "в Банк России" дополнить словами "в порядке, предусмотренном абзацем вторым пункта 2 статьи 20 настоящего Федерального закона";
б) абзац первый пункта 3 после слов "в Банк России" дополнить словами "в порядке, предусмотренном абзацем вторым пункта 2 статьи 20 настоящего Федерального закона";
4) пункт 7 статьи 25 изложить в следующей редакции:
"7. Отчет об итогах выпуска (дополнительного выпуска) эмиссионных ценных бумаг и документы для его государственной регистрации или уведомление об итогах выпуска (дополнительного выпуска) эмиссионных ценных бумаг представляются в Банк России в порядке, предусмотренном абзацем вторым пункта 2 статьи 20 настоящего Федерального закона."."#;