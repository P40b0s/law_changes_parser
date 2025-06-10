use format_constructor::DateTimeFormat;
use format_parser::TextExtractor;
use format_structure::Date;
use logger::info;
use nom::{IResult, branch::alt, bytes::complete::{tag, tag_no_case, take_until, take_until1}, character::complete::digit1, sequence::{tuple, pair, separated_pair}, combinator::map};
use serde::{Serialize, Deserialize};
use crate::{error::CustomNomError, json_path_creator::{JsonPathCreator, FormatPath, FormatPathItem}, parsers::{paths, ChangeAction}, nom_error, ChangesHierarchy};
use super::{space0, space1};
use std::fmt::Display;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TargetDocument
{
    ///вид документа в который вносятся изменения
    pub document_type: DocumentType,
    pub target_path : Option<FormatPathItem>,
    pub root_path: Option<JsonPathCreator>,
}

impl TargetDocument
{
    ///TODO необходимо проверить, бывает что в строке с реквизитами документа может быть простое изменение из списка, типа заменить слова на слова или признать утратившим силу итд.
    pub fn check_target_annotation(obj: &TextExtractor) -> Option<Self>
    {
        if let Ok(target) = target_document_info(&obj.get_text())
        {
            return Some(TargetDocument 
                { 
                    document_type: target.1.0,
                    root_path: target.1.1,
                    target_path: target.1.2,
                });
        }
        None
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DocumentType
{
    ///дата: 30 ноября 2020 года
    /// номер
    FederalLaw(String, String),
    Kodex(String)
}

impl Display for DocumentType
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            DocumentType::FederalLaw(date, number) => write!(f, "ФЗ от {} {}", date, number),
            DocumentType::Kodex(name) => f.write_str(name)    
        }
    }
}


fn vnesti_fz(s: &str) -> IResult<&str, (Option<JsonPathCreator>, DocumentType), CustomNomError<&str>>
{
    let vnes = vnesti_str(s);
    //статью 22
    if let Ok(path) = paths(vnes)
    {
        logger::info!("текущий путь->{}", path.1);
        //Федерального закона от 7 июля 2003 года № 126-ФЗ
        if let Ok(fz) = fz_requisites(path.0)
        {
            logger::info!("{}", fz.1);
            return Ok((fz.0, (Some(path.1), fz.1)));
        }
    }
    let fz = fz_requisites(vnes)?;
    logger::info!("{}", fz.1);
    Ok((fz.0, (None, fz.1)))
}

fn vnesti_str(s: &str) -> &str
{
    let vnes: IResult<&str, (&str, &str, &str, &str)> = tuple((tag("Внести"), space1, tag("в"), space1))(s);
    if vnes.is_ok()
    {
        vnes.unwrap().0
    }
    else
    {
        s
    }
}


fn vnesti_kodex(s: &str) -> IResult<&str, (Option<JsonPathCreator>, DocumentType), CustomNomError<&str>>
{
    let vnes = vnesti_str(s);
    //статью 22
    if let Ok(path) = paths(vnes)
    {
        logger::info!("текущий путь->{}", path.1);
        //Федерального закона от 7 июля 2003 года № 126-ФЗ
        if let Ok(kodex) = kodex_requisites(path.0)
        {

            logger::info!("{}", kodex.1);
            return Ok((kodex.0, (Some(path.1), kodex.1)));
        }
    }
    // кодекс об адмистративных правонарушениях
    // лесной кодекс
    let kodex = kodex_requisites(vnes)?;
    logger::info!("{}", kodex.1);
    Ok((kodex.0, (None, kodex.1)))
}


pub enum TargetDocumentOperation 
{
    ///Все нижеприведенные изменения относятся к этому документу
    ApplyAllBelowChanes,


}
///тип документа / относительный путь для глобальных изменений / абсолютный путь когда указывается на конкретный итем, например дополнить пунктом 8 следующего содержания
fn target_document_info(s: &str) -> IResult<&str, (DocumentType, Option<JsonPathCreator>, Option<FormatPathItem>), CustomNomError<&str>>
{
    let vnes = alt((vnesti_fz, vnesti_kodex))(s)?;
    if is_next_changes(vnes.0)
    {
      
        info!("Внести в {} (ПУТЬ:{}) все нижеприведенные изменения",vnes.1.1, vnes.1.0.as_ref().unwrap_or(&JsonPathCreator::default()));
        return Ok((vnes.0, (vnes.1.1, vnes.1.0, None)));
    }
    else if is_new_edition(vnes.0)
    {
        info!("Элемент {} изложить в новой редакции", vnes.1.0.as_ref().unwrap());
        return Ok((vnes.0, (vnes.1.1, vnes.1.0, None)));
    }
    else 
    {
        let path = is_add_new_item(vnes.0)?;
        if let Some(item) = path.1.last_path()
        {
            match item.get_item_type()
            {
                FormatPath::Item =>
                {
                    //создаем структуру и добавляем заполненую в задачи
                    //получается по пути vnes.1 надо сделать операцию добавления нового элемента
                    info!("Добавить по пути {} элемент {} № {}", vnes.1.0.as_ref().unwrap(), item.get_item_type(), item.get_item_number().unwrap());
                    return Ok((vnes.0, (vnes.1.1, vnes.1.0, Some(item.clone()))));
                    //let action = ChangeAction::ApplyItems(vec![]);
                }   
                FormatPath::Header(_) =>
                {
                    return Ok((vnes.0, (vnes.1.1, vnes.1.0, Some(item.clone()))));
                } 
                FormatPath::Indent =>
                {
                    return Ok((vnes.0, (vnes.1.1, vnes.1.0, Some(item.clone()))));
                }
            }
        }
        info!("{}", path.1);
    }
    // let err = "Не обнаружен абзац с определением целевого правового акта".to_owned();
    //error!("{}->{}",&err, s);
    // let e = nom::Err::Error(CustomNomError::Error(err, s));
     //return Err(e);
    let error = nom_error!("Не обнаружен абзац с определением целевого правового акта", s);
    Err(error)
}


fn fz_requisites(s: &str) -> IResult<&str, DocumentType, CustomNomError<&str>>
{
    let ws = space0(s)?;
    let fz = alt((
        tag_no_case("Федерального закона"),
        tag_no_case("Федеральный закон"),
        tag_no_case("Закона Российской Федерации")
    ))(ws.0)?;
    let (remain, (_, _, _, date,_, month, _, year, _, _, _, _, _, number, _ )) = tuple((
    space1,
    tag("от"),
    space1,
    digit1,
    space1,
    alt((tag("января"),
        tag("февраля"),
        tag("марта"),
        tag("апреля"),
        tag("мая"),
        tag("июня"),
        tag("июля"),
        tag("августа"),
        tag("сентября"),
        tag("октября"),
        tag("ноября"),
        tag("декабря"))),
    space1,
    digit1,
    space1,
    alt((tag("года"), tag("г."))),
    space1,
    tag("№"),
    space0,
    take_until(" "),
    space1
    ))(fz.0)?;
    let date_str = [date, " ", month, " ", year].concat();
    let res = Date::parse(&date_str).unwrap();
    Ok((remain, DocumentType::FederalLaw(res.val, number.to_owned())))
    
}
fn kodex_requisites(s: &str) -> IResult<&str, DocumentType, CustomNomError<&str>>
{
    let ws = space0(s)?;
    let k = alt((
            alt((
                tag_no_case("Семейного кодекса"),
                tag_no_case("Бюджетный кодекс"),
                tag_no_case("Бюджетного кодекса"),
                tag_no_case("Градостроительный кодекс"),
                tag_no_case("Градостроительного кодекса"),
                tag_no_case("Таможенный кодекс Таможенного союза"),
                tag_no_case("Таможенного кодекса Таможенного союза"),
                tag_no_case("Кодекс административного судопроизводства"),
                tag_no_case("Кодекса административного судопроизводства"),
                tag_no_case("Уголовно-исполнительный кодекс"),
                tag_no_case("Уголовно-исполнительного кодекса"),
                tag_no_case("Лесной кодекс"),
                tag_no_case("Лесного кодекса"),
                tag_no_case("Водный кодекс"),
                tag_no_case("Водного кодекса"),
                tag_no_case("Воздушный кодекс"),
                tag_no_case("Воздушного кодекса"),
                tag_no_case("Кодекс торгового мореплавания"),
                tag_no_case("Кодекса торгового мореплавания"),
                tag_no_case("Кодекс внутреннего водного транспорта"),
                tag_no_case("Кодекса внутреннего водного транспорта")
                )),
            alt((
                tag_no_case("Гражданский кодекс"),
                tag_no_case("Гражданского кодекса"),
                tag_no_case("Трудовой кодекс"),
                tag_no_case("Трудового кодекса"),
                tag_no_case("Налоговый кодекс"),
                tag_no_case("Налогового кодекса"),
                tag_no_case("Кодекс об административных правонарушениях"),
                tag_no_case("Кодекса об административных правонарушениях"),
                tag_no_case("Уголовный кодекс"),
                tag_no_case("Уголовного кодекса"),
                tag_no_case("Гражданский процессуальный кодекс"),
                tag_no_case("Гражданского процессуального кодекса"),
                tag_no_case("Уголовно-процессуальный кодекс"),
                tag_no_case("Уголовно-процессуального кодекса"),
                tag_no_case("Арбитражный процессуальный кодекс"),
                tag_no_case("Арбитражно процессуального кодекса"),
                tag_no_case("Земельный кодекс"),
                tag_no_case("Земельного кодекса"),
                tag_no_case("Жилищный кодекс"),
                tag_no_case("Жилищного кодекса"),
                tag_no_case("Семейный кодекс"),
                ))
        ))(ws.0)?;
    let ws = space1(k.0)?;
    Ok((ws.0, DocumentType::Kodex(k.1.to_owned())))
}


/// `следующие изменения:`
/// далее будет перечень изменеий их может быть много
fn is_next_changes(s: &str) -> bool
{
    let ch : IResult<&str, &str, CustomNomError<&str>> = take_until("следующие изменения:")(s);
    if ch.is_ok()
    {
        return true;
    }
    return false;
}

/// `изложить в следующей редакции:`
/// далее (статья/абзац/итем) будет изложен в новой редакции
/// отличие от `следующие изменения:` тут используем только абзацы текущего итема (есть варимнт что это абзац оО и после него идет хрен знает что)
fn is_new_edition(s: &str) -> bool
{
    let ch: IResult<&str, &str, CustomNomError<&str>> = take_until1("изложить в следующей редакции:")(s);
    if ch.is_ok()
    {
        return true;
    }
    return false;
}

/// `дополнить пунктом 1 следующего содержания:`
/// далее (статья/абзац/итем)
fn is_add_new_item(s: &str) -> IResult<&str, JsonPathCreator, CustomNomError<&str>>
{
    let ch = pair(take_until1("дополнить "), tag("дополнить "))(s)?;
    let path = paths(ch.0)?;
    let end = separated_pair(tag("следующего"), space1, tag("содержания:"))(path.0)?;
    Ok((end.0, path.1))
}

///`в подпункте 1 пункте 3 статьи 2^2`
/// абзац первый пункта 2 статьи 4
fn fz(s: &str) -> IResult<&str, &str, CustomNomError<&str>>
{
    let v = alt((
        map(
            pair(
                space0, 
                alt((tag_no_case("дополнить новыми"),
                            tag_no_case("дополнить новым"),
                            tag_no_case("дополнить")
                            ))
                        ),
        |_| ""),
        space0))(s)?;
    let (remain, _) = 
    paths(v.0)?;
    let end = alt((space1, tag(":")))(remain)?;
    Ok((end.0, ""))
}


#[cfg(test)]
mod tests
{
    use logger::info;
    use nom::{ bytes::complete::{take_until, tag, is_a, tag_no_case}, combinator::map, IResult, multi::{many0, many1, many_till}, sequence::{pair, tuple}, character::complete::{anychar, alpha0, alpha1, alphanumeric1, digit1}};
    use crate::{error::CustomNomError, parsers::{space1, space0}};

   
    #[test]
    fn test_vnesti()
    {
        logger::StructLogger::initialize_logger();
        let check_vnesti_str : Vec<&str> = vec![
            r#"Внести в статью 46 Федерального закона от 7 июля 2003 года № 126-ФЗ "О связи" (Собрание законодательства Российской Федерации, 2003, № 28, ст. 2895; 2007, № 7, ст. 835; 2010, № 7, ст. 705; № 31, ст. 4190; 2012, № 31, ст. 4328; № 53, ст. 7578; 2013, № 48, ст. 6162; 2014, № 19, ст. 2302; № 30, ст. 4273; № 49, ст. 6928; 2015, № 29, ст. 4383; 2016, № 15, ст. 2066; № 27, ст. 4213; № 28, ст. 4558; 2017, № 31, ст. 4742, 4794) следующие изменения:"#,
            "Внести в Федеральный закон от 30 декабря 2015 года № 431-ФЗ \"О геодезии, картографии и пространственных данных и о внесении изменений в отдельные законодательные акты Российской Федерации\" (Собрание законодательства Российской Федерации, 2016, № 1, ст. 51; 2018, № 32, ст. 5135; 2021, № 24, ст. 4198; 2022, № 1, ст. 18) следующие изменения:",
            "Внести в Земельный кодекс Российской Федерации (Собрание законодательства Российской Федерации, 2001, № 44, ст. 4147; 2018, № 32, ст. 5134, 5135; 2019, № 52, ст. 7795; 2022, № 1, ст. 14; № 29, ст. 5251; 2023, № 14, ст. 2373) следующие изменения:",
            "Внести в статью 4 Федерального закона от 30 декабря 2021 года № 448-ФЗ \"О публично-правовой компании \"Роскадастр\" (Собрание законодательства Российской Федерации, 2022, № 1, ст. 17; № 52, ст. 9376) следующие изменения:",
            "Внести в статью 14 Федерального закона от 30 декабря 2021 года № 449-ФЗ \"О внесении изменений в отдельные законодательные акты Российской Федерации\" (Собрание законодательства Российской Федерации, 2022, № 1, ст. 18) следующие изменения:",
            "Внести в Земельный кодекс Российской Федерации (Собрание законодательства Российской Федерации, 2001, № 44, ст. 4147; 2005, № 30, ст. 3128; 2007, № 21, ст. 2455; № 31, ст. 4009; 2008, № 30, ст. 3597; 2011, № 30, ст. 4594; № 50, ст. 7343; № 51, ст. 7448; 2014, № 26, ст. 3377; № 30, ст. 4218, 4225; 2015, № 1, ст. 40; № 10, ст. 1418; № 27, ст. 3997; № 29, ст. 4339, 4350, 4378; 2016, № 18, ст. 2495; № 26, ст. 3890; № 27, ст. 4267, 4269, 4282, 4298, 4306; 2017, № 27, ст. 3938; № 31, ст. 4765, 4766; 2018, № 1, ст. 90; № 27, ст. 3947, 3954; № 28, ст. 4139, 4149; № 32, ст. 5133, 5134, 5135; № 53, ст. 8411; 2019, № 26, ст. 3317; № 31, ст. 4442; № 52, ст. 7820; 2020, № 29, ст. 4504, 4512; № 42, ст. 6505; № 52, ст. 8581; 2021, № 1, ст. 33; № 17, ст. 2878; № 27, ст. 5054, 5101; 2022, № 1, ст. 5, 18, 45, 47; № 18, ст. 3009; № 22, ст. 3537; № 29, ст. 5220, 5279, 5283; № 41, ст. 6947; 2023, № 12, ст. 1890; № 14, ст. 2373; № 25, ст. 4417, 4433; № 26, ст. 4675) следующие изменения:",
            "Статью 98 Земельного кодекса Российской Федерации (Собрание законодательства Российской Федерации, 2001, № 44, ст. 4147; 2009, № 11, ст. 1261; 2016, № 26, ст. 3875) изложить в следующей редакции:",
            "Часть 4 статьи 5 Федерального закона от 29 декабря 2006 года № 264-ФЗ \"О развитии сельского хозяйства\" (Собрание законодательства Российской Федерации, 2007, № 1, ст. 27; 2013, № 27, ст. 3477; 2015, № 1, ст. 20; 2018, № 1, ст. 8; № 49, ст. 7518) дополнить пунктом 8 следующего содержания:",
            "Часть 1 статьи 44 Федерального закона от 21 декабря 2021 года № 414-ФЗ \"Об общих принципах организации публичной власти в субъектах Российской Федерации\" (Собрание законодательства Российской Федерации, 2021, № 52, ст. 8973; 2023, № 1, ст. 7; № 16, ст. 2766; № 25, ст. 4433, 4434; Российская газета, 2023, 18 июля) дополнить пунктом 17^1 следующего содержания:",
            r#"Внести в Земельный кодекс Российской Федерации (Собрание законодательства Российской Федерации, 2001, № 44, ст. 4147; 2005, № 30, ст. 3128; 2007, № 21, ст. 2455; № 31, ст. 4009; 2008, № 30, ст. 3597; 2011, № 30, ст. 4594; № 50, ст. 7343; № 51, ст. 7448; 2014, № 26, ст. 3377; № 30, ст. 4218, 4225; 2015, № 1, ст. 40; № 10, ст. 1418; № 27, ст. 3997; № 29, ст. 4339, 4350, 4378; 2016, № 18, ст. 2495; № 26, ст. 3890; № 27, ст. 4267, 4269, 4282, 4298, 4306; 2017, № 27, ст. 3938; № 31, ст. 4765, 4766; 2018, № 1, ст. 90; № 27, ст. 3947, 3954; № 28, ст. 4139, 4149; № 32, ст. 5133, 5134, 5135; № 53, ст. 8411; 2019, № 26, ст. 3317; № 31, ст. 4442; № 52, ст. 7820; 2020, № 29, ст. 4504, 4512; № 42, ст. 6505; № 52, ст. 8581; 2021, № 1, ст. 33; № 17, ст. 2878; № 27, ст. 5054, 5101; 2022, № 1, ст. 5, 18, 45, 47; № 18, ст. 3009; № 22, ст. 3537; № 29, ст. 5220, 5279, 5283; № 41, ст. 6947; 2023, № 12, ст. 1890; № 14, ст. 2373; № 25, ст. 4417, 4433; № 26, ст. 4675) следующие изменения:"#,
            r#"Внести в статью 22 Закона Российской Федерации от 14 мая 1993 года № 4973-I "О зерне" (Ведомости Съезда народных депутатов Российской Федерации и Верховного Совета Российской Федерации, 1993, № 22, ст. 799; Собрание законодательства Российской Федерации, 2021, № 1, ст. 59; № 24, ст. 4188) следующие изменения:"#,
            r#"Часть 4 статьи 5 Федерального закона от 29 декабря 2006 года № 264-ФЗ "О развитии сельского хозяйства" (Собрание законодательства Российской Федерации, 2007, № 1, ст. 27; 2013, № 27, ст. 3477; 2015, № 1, ст. 20; 2018, № 1, ст. 8; № 49, ст. 7518) дополнить пунктом 8 следующего содержания:"#
        ];
        for s in check_vnesti_str
        {
            
            let p = super::target_document_info(s).unwrap();
            info!("тест строки ->{} остаток токенов ->{}", s, p.0);
        }
    }
    #[test]
    fn test_vnesti2()
    {
        logger::StructLogger::initialize_logger();
        let check_vnesti_str : Vec<&str> = vec![
            "Статью 52 Федерального закона от 31 июля 2020 года № 248-ФЗ \"О государственном контроле (надзоре) и муниципальном контроле в Российской Федерации\" (Собрание законодательства Российской Федерации, 2020, № 31, ст. 5007) дополнить частями 10 - 13 следующего содержания:",
        ];
        for s in check_vnesti_str
        {
            
            let p = super::target_document_info(s).unwrap();
            info!("тест строки ->{} остаток токенов ->{}", s, p.0);
        }
    }

    #[test]
    fn test_vnesti3()
    {
        logger::StructLogger::initialize_logger();
        let check_vnesti_str : Vec<&str> = vec![
            "Часть 1 статьи 44 Федерального закона от 21 декабря 2021 года № 414-ФЗ \"Об общих принципах организации публичной власти в субъектах Российской Федерации\" (Собрание законодательства Российской Федерации, 2021, № 52, ст. 8973; 2023, № 1, ст. 7; № 16, ст. 2766; № 25, ст. 4433, 4434; Российская газета, 2023, 18 июля) дополнить пунктом 171 следующего содержания:",
        ];
        for s in check_vnesti_str
        {
            
            let p = super::target_document_info(s).unwrap();
            info!("тест строки ->{} остаток токенов ->{}", s, p.0);
        }
    }

    

    #[test]
    fn fz_test()
    {
        logger::StructLogger::initialize_logger();
        let test_str = " Федеральный закон от 30 декабря 2015 года № 431-ФЗ ";
        let p = super::fz(test_str).unwrap();
        info!("тест строки ->{} остаток токенов ->{} документ->{}", test_str, p.0, p.1);
    }

    #[test]
    fn error_test()
    {
        //logger::StructLogger::initialize_logger();
        let test_str = "тестирование вывода ошибки через макрос";
        let p = super::target_document_info(test_str).err().unwrap().map(|m|
            {
                match m
                {
                    
                    CustomNomError::Error(e, i)=>
                    {
                        info!("{} {}",e,i);
                    },
                    CustomNomError::Nom(e, i)=>
                    {
                        info!("{} {}",e,i.description());
                    }
                }
            }
            
        );
    }

    

}