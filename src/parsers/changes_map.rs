use std::fmt::Display;

use format_parser::{ItemParser, HeaderParser, TextExtractor, StringExtractor};
use logger::{info, debug, error, warn};
use nom::IResult;
use serde::{Serialize, Deserialize};

use crate::json_path_creator::FormatPathItem;
use crate::{ChangesHierarchyItem, ChangesHierarchy};
use crate::error::{CustomNomError, ChangesParserError};
use crate::{parsers::ChangeAction, json_path_creator::JsonPathCreator};
use crate::parsers::Deserializer;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemainTokens
{
    source_string: String,
    remains_tokens: String
}
impl RemainTokens
{
    pub fn new(source_string: &str, remains: &str) -> Self
    {
        RemainTokens { source_string: source_string.to_owned(), remains_tokens: remains.to_owned()}
    }
}
impl Display for RemainTokens
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        if self.remains_tokens.len() > 0
        {
            let error = ["Есть не полностью обработанная строка ->", &self.source_string, "| остаток токенов ->", &self.remains_tokens, "|"].concat();
            f.write_str(&error)
        }
        else 
        {
            f.write_str("")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChangesMap
{
    ///путь текущего элемента
    pub path: JsonPathCreator,
    pub operations: Vec<ChangeAction>,
}

impl Default for ChangesMap
{
    fn default() -> Self 
    {
        ChangesMap { path: JsonPathCreator::default(), operations: vec![] }
    }
}


impl ChangesMap
{
    pub fn new(path: JsonPathCreator, operations: Vec<ChangeAction>) -> Self
    {
        ChangesMap { path, operations }
    }
    pub fn with_path(path: JsonPathCreator) -> Self
    {
        ChangesMap { path, operations: vec![] }
    }
    pub fn add_operation(&mut self, operation: ChangeAction)
    {
        self.operations.push(operation);
    }
    pub fn add_operations(&mut self, operations: Vec<ChangeAction>)
    {
        self.operations.extend(operations);
    }
    pub fn parse<'a>(groups: ChangesHierarchy) -> Result<(Vec<ChangesMap>, Vec<RemainTokens>), ChangesParserError<'a>>
    {
        let mut changes: Vec<ChangesMap> = vec![];
        let mut errors: Vec<String> = vec![];
        let mut remains: Vec<RemainTokens> = vec![];
        //если у таргета есть изменения которые указаны точно, то разбираем их
        //если есть итемы то rootitems пустые
        if groups.get_items().len() == 0
        {
            if let Some(target) = groups.get_target_document()
            {
                if let Some(item) = groups.get_root_items()
                {
                    parse_group_item(item, &mut changes, &mut errors, &mut remains, target.root_path.clone(), true)
                }
            }
        }
        for item in groups.get_items()
        {
            parse_group_item(item, &mut changes, &mut errors, &mut remains, None, false);
        }
        //info!("{}", serde_json::to_string_pretty(&changes).unwrap());
        if errors.len() > 0
        {
            logger::error!("Есть ошбки обработки строк, последовательность токенов не найдена! ->{}",serde_json::to_string_pretty(&errors).unwrap());
            return Err(ChangesParserError::TokensParseErrors(errors));
        }
        if remains.len() > 0
        {
            for r in &remains
            {
                logger::warn!("{}", r.to_string());
            }
        }
        Ok((changes, remains))
    }

    ///Преобразуем вектор в `Vec<TextExtractor>` и потом извлекаем карту изменеий
    pub fn get_changes_map(text: Vec<&str>) -> Result<(Vec<ChangesMap>, Vec<RemainTokens>), ChangesParserError>
    {
        let mut extractor: Vec<TextExtractor> = vec![];
        for l in text
        {
            let is_change = crate::parsers::check_if_change(l);
            let ex = <TextExtractor as StringExtractor>::extract(l, is_change);
            extractor.push(ex);
        }
        if let Ok(hierarchy) = ChangesHierarchy::get_changes_hierarchy(&extractor)
        {
            let map = ChangesMap::parse(hierarchy);
            map
        }
        else 
        {
            return Err(ChangesParserError::Error("Ошибка получения иерархии изменений, подробности в лог файле.".to_owned()))    
        }
        
    }
}

fn load_file() 
{
    let changes_groups = *crate::ChangesHierarchy::load("/hard/xar/projects/rust/format/format_struct/changes_parser/items_with_changes_2.json").unwrap();
    let mut changes: Vec<ChangesMap> = vec![];
    let mut errors: Vec<String> = vec![];
    let mut remain_tokens: Vec<RemainTokens> = vec![];
    for item in changes_groups.get_items()
    {
        parse_group_item(item, &mut changes, &mut errors, &mut remain_tokens, None, false);
    }
    info!("{}", serde_json::to_string_pretty(&changes).unwrap());
    if remain_tokens.len() > 0
    {
        logger::warn!("Возможно есть не полностью обработанные строки, необходима проверка оператором ->{}",serde_json::to_string_pretty(&remain_tokens).unwrap());
    }
    if errors.len() > 0
    {
        logger::error!("Есть ошбки обработки строк, последовательность токенов не найдена! ->{:?}",&errors);
    }
}

fn parse_group_item(item: &ChangesHierarchyItem,
    changes: &mut Vec<ChangesMap>,
    errors: &mut Vec<String>,
    remain_tokens: &mut Vec<RemainTokens>,
    parent_path: Option<JsonPathCreator>,
    path_with_requisites: bool)
{
    //отрезаем от строки с реквизитами то что не касается операций (внести в часть 14 федерального закона ... итд)
    let txt = match path_with_requisites 
    {
        true =>
        {
            if let Some(l) = item.text.get_text().rsplit_once(")").into_iter().last()
            {
                l.1.to_owned()
            }
            else if let Some(l) = item.text.get_text().rsplit_once("\"").into_iter().last()
            {
                l.1.to_owned()
            }
            else
            {
                item.text.get_text()
            }
        },
        false =>
        {
            item.text.get_text()
        } 
    };
    //1) в статье 23: тут мы только берем путь, и пробрасываем его дальше
    if let Ok(a) = super::only_path_definition(&txt) as  IResult<&str, JsonPathCreator, CustomNomError<&str>>
    {
        debug!("определение, после него идут изменения: {}| путь ->{:?}",&txt, a);
        let par_path = match parent_path
        {
            Some(p) => p + a.1,
            None => a.1
        };
        for subitem in &item.subitems
        {
            parse_group_item(subitem, changes, errors, remain_tokens, Some(par_path.clone()), false);
        }
        if a.0.len() > 0
        {
            remain_tokens.push(RemainTokens::new(&txt, a.0));
        }
    }
    //тут у нас дополнения с изменениями которые занимают несколько абзацев, их берем из полей changes
    else if let Ok(a) = super::apply_all(&txt) as  IResult<&str, (Option<JsonPathCreator>, JsonPathCreator), CustomNomError<&str>>
    {
        let parsed = parse_items(a.0, a.1.0, a.1.1,item, parent_path, false);
        if let Ok(ch) = parsed
        {
            changes.push(ch);
        }
        else 
        {
            errors.push(parsed.err().unwrap().to_string())
        }
        if a.0.len() > 0
        {
            remain_tokens.push(RemainTokens::new(&txt, a.0));
        }
    }
    //тут изменения в пределах абзаца дополнить словами заменить словами итд.
    else if let Ok(w) = super::words::words_operations(&txt) as IResult<&str, (JsonPathCreator, Vec<ChangeAction>), CustomNomError<&str>>
    {
        let par_path = match parent_path
        {
            Some(p) => p + w.1.0,
            None => w.1.0
        };
        debug!("адрес: {}| операции со словами -> {}", &par_path, serde_json::to_string_pretty(&w.1.1).unwrap());
        let mut change = ChangesMap::with_path(par_path);
        change.add_operations(w.1.1);
        changes.push(change);
        if w.0.len() > 0
        {
            remain_tokens.push(RemainTokens::new(&txt, w.0));
        }
    }
    else if let Ok(r) = super::replace_all(&txt) as IResult<&str, JsonPathCreator, CustomNomError<&str>>
    {
        let parsed = parse_items(r.0, None, r.1, item, parent_path, true);
        if let Ok(ch) = parsed
        {
            changes.push(ch);
        }
        else 
        {
            errors.push(parsed.err().unwrap().to_string())
        }
        if r.0.len() > 0
        {
            remain_tokens.push(RemainTokens::new(&txt, r.0));
        }
        //logger::warn!("НЕ РЕАЛИЗОВАНО! {} -> {:?}",&txt, r.1);
    }
    else 
    {
        errors.push(txt);    
    }
}
///по сути путь всегда будет из одной катекрории итемов потому что идет перечисление, пункты 1 2 и 10
/// надо вычислить уникальные значения и их использовать
/// выдаем селф если там больше 1 уникальных значений пути
fn group_self_path(self_path: &JsonPathCreator) -> JsonPathCreator
{
    let self_items = self_path.get_path_items();
    let mut paths = JsonPathCreator::default();
    if self_items.len() > 1
    {
        let mut items: Vec<FormatPathItem> = vec![];
        for item in self_items
        {
            if !items.iter().any(|a| a.get_item_type() == item.get_item_type())
            {
                items.push(item.clone());
            }
        }
        let items = &items[..items.len() -1];
        paths.add(items);
    }
    return paths;
}

fn parse_items<'a>(remain_tokens: &str,
target_path: Option<JsonPathCreator>,
self_path: JsonPathCreator,
item: &ChangesHierarchyItem,
parent_path: Option<JsonPathCreator>,
replace: bool) -> Result<ChangesMap, ChangesParserError<'a>>
{
    let par_path = match parent_path
    {
        Some(p) => p + target_path.unwrap_or(JsonPathCreator::default()),
        None => target_path.unwrap_or(JsonPathCreator::default())
    };
    let target_path_from_self = group_self_path(&self_path);
    let par_path = par_path + target_path_from_self;
    let mut change_struct = ChangesMap::with_path(par_path.clone());
    if let Some(last_path) =  self_path.last_path()
    {
        match last_path.get_enum().0
        {
            crate::json_path_creator::FormatPath::Header(_) =>
            {
                let mut ip = ItemParser::new();
                let mut  hp = HeaderParser::new();
                for (i, change) in item.changes.iter().enumerate()
                {
                    let mut change = change.clone();
                    change.delete_start_end_quotes();
                    if i == 0
                    {
                        let _r = hp.parse(&change, Some(&mut ip));
                    }
                    else 
                    {
                        let _r = ip.parse(&change);
                    } 
                }
                ip.set_is_changed(true);
                //logger::info!("{}", serde_json::to_string_pretty(&ip.items).unwrap());
                hp.check_items(&mut ip);
                let mut changed_headers = hp.get_headers().unwrap();
                for ch in changed_headers.iter_mut()
                {
                    ch.set_is_change(true);
                }
                if replace
                {
                    if changed_headers.len() > 1
                    {
                        error!("На замещение найдено больше 1 заголовка: -> {:?}", &changed_headers);
                    }
                    else 
                    {
                        change_struct.add_operation(ChangeAction::HeaderInNewEdition(changed_headers[0].clone()));
                    }
                }
                else
                {
                    change_struct.add_operation(ChangeAction::ApplyHeaders(changed_headers));
                    debug!("Найдены измененные заголовки: -> {:?}", &change_struct);
                }
            },
            crate::json_path_creator::FormatPath::Item  =>
            {
                let mut ip = ItemParser::new();
                for change in &item.changes
                {
                    let mut change = change.clone();
                    change.delete_start_end_quotes();
                    let _r = ip.parse(&change);
                }
                ip.set_is_changed(true);
                let changed_items = ip.get_items().unwrap();
                if replace
                {
                    //if changed_items.len() > 1
                    //{
                    //    error!("На замещение найдено больше 1 итема: -> {:?}", &changed_items);
                    //}
                    change_struct.add_operation(ChangeAction::ItemsInNewEdition(changed_items));
                }
                else
                {
                    change_struct.add_operation(ChangeAction::ApplyItems(changed_items));
                    debug!("Найдены измененные нумерованные списки: -> {:?}", &change_struct);
                }
               
            },
            crate::json_path_creator::FormatPath::Indent =>
            {
                let mut ip = ItemParser::new();
                for change in &item.changes
                {
                    let mut change = change.clone();
                    change.delete_start_end_quotes();
                    let _r = ip.parse(&change);
                }
                //может быть случай когда изменения вносятся а первый абзац с указанием пункта:
                //а) абзац первый пункта 1 изложить в следующей редакции:
                //"1. Конфиденциальная информация 
                if ip.get_indents().is_none()
                {
                    if let Some(itm) = ip.get_items()
                    {
                        let ind = itm.first().as_ref().unwrap().indents.as_ref().unwrap().first().unwrap().clone();
                        change_struct.add_operation(ChangeAction::IndentsInNewEdition(vec![ind]));
                    }
                    else
                    {
                        error!("Ошибка извлечения структуры изменений из ->{:?}{:?} ", par_path, last_path);
                    }
                    
                }
                else
                {
                    let mut changed_indents = ip.get_indents().unwrap();
                    for ch in changed_indents.iter_mut()
                    {
                        ch.set_is_change(true);
                    }
                    if replace
                    {
                        //if changed_indents.len() > 1
                        //{
                        //    error!("На замещение найдено больше 1 абзаца: -> {:?}", &changed_indents);
                        //}
                        change_struct.add_operation(ChangeAction::IndentsInNewEdition(changed_indents));
                    }
                    else 
                    {
                        change_struct.add_operation(ChangeAction::ApplyIndents(changed_indents));
                        debug!("Найдены измененные абзацы: -> {:?}", &change_struct);
                    } 
                }
                
            }
        }
    }
    else 
    {
        //своего пути нет, значит это название берем последний итем из пути (это 99.9% хедер)
        if let Some(last) = par_path.last_path()
        {
            if &item.changes.len() != &1
            {
                return Err(ChangesParserError::Error(format!("Для изменения {} количество абзацев превышает 1!", remain_tokens)));
            }
            let change = &mut item.changes[0].clone();
            change.delete_start_end_quotes();
            match last.get_enum().0
            {
                crate::json_path_creator::FormatPath::Header(_) =>
                {
                    let mut  hp = HeaderParser::new(); 
                    let _r = hp.parse(&change, None).map_err(|e| ChangesParserError::Error(e.to_string()))?;
                    let changed_headers = hp.get_headers().unwrap();
                    change_struct.add_operation(ChangeAction::HeaderNameInNewEdition(changed_headers[0].clone()));
                },
                _  =>
                {
                    return Err(ChangesParserError::Error(format!("Изменение наименование реализовано только для заголовков!->{}| не является заголовком!", remain_tokens )));
                },
            }
        }
    }
    Ok(change_struct)
}


#[cfg(test)]
mod tests
{
    use std::{fs, path::{Path, PathBuf}};

    use format_structure::{write_json, to_json};
    use insta::assert_json_snapshot;

    use crate::{json_path_creator::JsonPathCreator, parsers::ChangeAction};

    use super::load_file;
    const CHANGES_MAP_TEST_PATH : &str = "/home/phobos/projects/rust/format/format_struct/changes_parser/tests/changes_map_tests/";
    #[test]
    fn test1()
    {
        logger::StructLogger::initialize_logger();
        load_file();
    }
    fn get_path(name: &str) -> PathBuf
    {
        Path::new(CHANGES_MAP_TEST_PATH).join(name)
    }
    #[test]
    fn test2()
    {
        logger::StructLogger::initialize_logger();
        let mut path = JsonPathCreator::default();
        path.add_header("статья", Some("1"))
        .add_item(Some("2"))
        .add_item(Some("3"));
        let res = super::group_self_path(&path);
        assert_eq!(res.get_path_items().iter().next().unwrap().get_item_number().unwrap(), "1");
    }

    #[test]
    fn test3()
    {
        //logger::StructLogger::initialize_logger();
        let txt_file = get_path("test1.txt");
        let json_file = get_path("test1_map");
        let test_file = fs::read_to_string(txt_file).unwrap();
        let test_data = test_file.lines().map(|m| m).collect::<Vec<&str>>();
        let groups = super::ChangesMap::get_changes_map(test_data).unwrap();
        let lst = groups.0.last().unwrap().operations.last().unwrap();
        let last_action = ChangeAction::ReplaceWords("лет;".to_owned(), "лет. Законом субъекта Российской Федерации может быть предусмотрено, что такие граждане должны состоять на учете в качестве нуждающихся в жилых помещениях или иметь основания для постановки на данный учет, а также требование об отсутствии у таких граждан права собственности на иные земельные участки, предоставленные для индивидуального жилищного строительства или ведения личного подсобного хозяйства в данном муниципальном образовании;".to_owned());
        assert_eq!(lst, &last_action);
        write_json(groups.0, json_file);
    }
    #[test]
    fn test4()
    {
        let txt_file = get_path("test2.txt");
        let json_file = get_path("test2_map");
        let test_file = fs::read_to_string(txt_file).unwrap();
        let test_data = test_file.lines().map(|m| m).collect::<Vec<&str>>();
        let groups = super::ChangesMap::get_changes_map(test_data).unwrap();
        write_json(groups.0, json_file);
    }
    #[test]
    fn test5()
    {
        let txt_file = get_path("test3.txt");
        let json_file = get_path("test3_map");
        let test_file = fs::read_to_string(txt_file).unwrap();
        let test_data = test_file.lines().map(|m| m).collect::<Vec<&str>>();
        let groups = super::ChangesMap::get_changes_map(test_data).unwrap();
        write_json(groups.0, json_file);
    }
    #[test]
    fn test6()
    {
        let txt_file = get_path("test4.txt");
        let json_file = get_path("test4_map");
        let test_file = fs::read_to_string(txt_file).unwrap();
        let test_data = test_file.lines().map(|m| m).collect::<Vec<&str>>();
        let groups = super::ChangesMap::get_changes_map(test_data).unwrap();
        write_json(groups.0, json_file);
    }
    #[test]
    fn test7()
    {
        let txt_file = get_path("test5.txt");
        let json_file = get_path("test5_map");
        let test_file = fs::read_to_string(txt_file).unwrap();
        let test_data = test_file.lines().map(|m| m).collect::<Vec<&str>>();
        let groups = super::ChangesMap::get_changes_map(test_data).unwrap();
        write_json(groups.0, json_file);
    }
}
