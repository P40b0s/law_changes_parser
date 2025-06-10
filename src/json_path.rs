use std::path::Path;
use format_structure::{Item, Indent, Header, Root};
use logger::{error, info};
use serde::Serialize;
use serde_json::Value;
use jsonpath_rust::{JsonPathFinder, JsonPathQuery, JsonPathInst, JsonPathValue};
use std::str::FromStr;

use crate::error::ChangesParserError;

pub struct JsonPathSearch
{
    document: Root,
    value: Value,
}

impl JsonPathSearch
{
    pub fn new(doc: Root) -> Self
    {
        JsonPathSearch { value: doc.get_value(), document: doc }
    }
    //TODO предподготовка, взять файл как строку и сделать на нем jsonpath_lib::replace_with 
    //чтобы удать все флаги is change
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Option<Self>
    {
        let root = Root::from_file(file_path.as_ref())?;
        Some(JsonPathSearch::new(root))
    }
    pub fn add_header_after_self<P: AsRef<str>>(&mut self, path: P, header: Header)
    {
        let value = self.get_parent_array(path);
        match value
        {
            SearchResult::Headers(i, vec) => 
            {
                info!("Искомый элемент обнаружен по индексу {} в векторе {:?}", i, serde_json::to_string_pretty(vec).unwrap());
                vec.insert(i+1, header);
                info!("Добавлен новый элемент в вектор {:?}", serde_json::to_string_pretty(vec).unwrap());
            },
            SearchResult::NotFound(s) => info!("ошибка поиска элемента {}", s),
            _ => info!("Метод обработки поискового запроса не определен!"),
        }
    }
    pub fn replace_header<'a>(&mut self, header: &Header) -> Result<(), ChangesParserError<'a>>
    {
        let replacement = self.document.body.as_mut()
        .and_then(|a| a.headers.as_mut()
        .and_then(|h| h.iter_mut().find(|f| &f.number == &header.number && &f.header_type == &header.header_type)));
        if let Some(repl) = replacement
        {
            repl.replace(&header);
            logger::info!("Заменен заголовок {}", serde_json::to_string_pretty(&repl.number).unwrap());
            Ok(())
        }
        else
        {
            let error = format!("Заголовок № {:?} типа {} не найден в документе для внесения изменений", header.number, header.header_type);
            error!("{}", &error);
            Err(ChangesParserError::OperationError(error))
        }
    }
    pub fn add_item_after_self<P: AsRef<str>>(&mut self, path: P, item: Item)
    {
        let value = self.get_parent_array(path);
        match value
        {
            SearchResult::Items(i, vec) => 
            {
                info!("Искомый элемент обнаружен по индексу {} в векторе {:?}", i, serde_json::to_string_pretty(vec).unwrap());
                vec.insert(i+1, item);
                info!("Добавлен новый элемент в вектор {:?}", serde_json::to_string_pretty(vec).unwrap());
            },
            SearchResult::NotFound(s) => info!("ошибка поиска элемента {}", s),
            _ => info!("Метод обработки поискового запроса не определен!"),
        }
    }
    pub fn add_indent_after_self<P: AsRef<str>>(&mut self, path: P, indent: Indent)
    {
        let value = self.get_parent_array(path);
        match value
        {
            SearchResult::Indents(i, vec) => 
            {
                info!("Искомый элемент обнаружен по индексу {} в векторе {:?}", i, serde_json::to_string_pretty(vec).unwrap());
                vec.insert(i+1, indent);
                info!("Добавлен новый элемент в вектор {:?}", serde_json::to_string_pretty(vec).unwrap());
            },
            SearchResult::NotFound(s) => info!("ошибка поиска элемента {}", s),
            _ => info!("Метод обработки поискового запроса не определен!"),
        }
    }
    pub fn get_document(&self) -> &Root
    {
        &self.document
    }
}

pub trait Search
{
    fn search<'a, P: AsRef<str>>(&mut self, path: P) -> SearchResult;
    fn get_parent_array<'a, P: AsRef<str>>(&mut self, path: P) -> SearchResult;
}
pub enum SearchResult<'a>
{
    Header(&'a mut Header),
    Headers(usize, &'a mut Vec<Header>),
    Item(&'a mut Item),
    Items(usize, &'a mut Vec<Item>),
    Indent(&'a mut Indent),
    Indents(usize, &'a mut Vec<Indent>),
    NotFound(String)
}

impl Search for JsonPathSearch
{
    fn search<'a, P: AsRef<str>>(&mut self, path: P) -> SearchResult
    {
        let mut searched_val : SearchResult = SearchResult::NotFound(["По запросу ", path.as_ref(), " не найдено ни одного элемента"].concat());
        let selector = self.value.clone().path(path.as_ref());
        if selector.is_err()
        {
            let err = ["Ошибка запроса ", path.as_ref(), " ", &selector.err().unwrap().to_string()].concat();
            error!("{}", &err);
            return SearchResult::NotFound(err);
        }
        let jp_result = selector.unwrap();
        if let Some(jp_array) = jp_result.as_array()
        {
            if jp_array.len() == 1
            {
                let itm = jp_array.last().unwrap();
                let id = itm["id"].as_str();
                if id.is_none()
                {
                    let err = ["Ошибка запроса ", path.as_ref(), " отсуствует поле id!"].concat();
                    error!("{}", &err);
                    return SearchResult::NotFound(err);
                }
                let id = id.unwrap();
                if let Some(body) = self.document.body.as_mut()
                {
                    if let Some(i) = body.items.as_mut()
                    {
                        if let Some(item) = search_item(i, id)
                        {
                            searched_val = item;
                        }
                    }
                    if let Some(i) = body.indents.as_mut()
                    {
                        if let Some(indent) = search_indent(i, id)
                        {
                            searched_val = SearchResult::Indent(indent);
                        }
                    }
                    if let Some(headers) =  body.headers.as_mut()
                    {
                        for h in headers.iter_mut()
                        {
                            if &h.id == id
                            {
                                searched_val = SearchResult::Header(h);
                                break;
                            }    
                            if let Some(i) = h.items.as_mut()
                            {
                                if let Some(item) = search_item(i, id)
                                {
                                    searched_val = item;
                                    break;
                                }
                            }
                            if let Some(i) = h.indents.as_mut()
                            {
                                if let Some(indent) = search_indent(i, id)
                                {
                                    searched_val = SearchResult::Indent(indent);
                                    break;
                                }
                            }
                        }
                    }
                }
                return searched_val;
            }
            else if jp_array.len() == 0
            {
                let err = ["По запросу ", path.as_ref(),  " не найдено ни одного объекта, уточните запрос"].concat();
                error!("{} {:?}", &err, &jp_array);
                return SearchResult::NotFound(err);
            }
            else 
            {
                let err = ["По запросу ", path.as_ref(),  " найдено больше одного объекта, уточните запрос"].concat();
                error!("{} {:?}", &err, &jp_array);
                return SearchResult::NotFound(err);    
            }
        }
        else
        {
            return SearchResult::NotFound(["По запросу ", path.as_ref(), " не обнаружен массив элементов!"].concat());   
        }
        
    }

    fn get_parent_array<'a, P: AsRef<str>>(&mut self, path: P) -> SearchResult
    {
        let selector = self.value.clone().path(path.as_ref());
        if selector.is_err()
        {
            let err = ["Ошибка запроса ", path.as_ref(), " ", &selector.err().unwrap().to_string()].concat();
            error!("{}", &err);
            return SearchResult::NotFound(err);
        }
        let jp_val = selector.unwrap();
        if !jp_val.is_array()
        {
            let err = ["Ошибка запроса ", path.as_ref(), " полученный объект не является массивом!->", &jp_val.to_string()].concat();
            error!("{}", &err);
            return SearchResult::NotFound(err);
        }
        let itm = jp_val.as_array().unwrap().last().unwrap();
        let id = itm["id"].as_str();
        if id.is_none()
        {
            let err = ["Ошибка запроса ", path.as_ref(), " отсуствует поле id!"].concat();
            error!("{}", &err);
            return SearchResult::NotFound(err);
        }
        let id = id.unwrap();
        if let Some(body) = self.document.body.as_mut()
        {
            if let Some(i) = body.items.as_mut()
            {
                if let Some(item) = search_item_array(i, id)
                {
                    return item;
                }
            }
            if let Some(i) = body.indents.as_mut()
            {
                if let Some(indent) = search_indent_array(i, id)
                {
                    return indent;
                }
            }
            if let Some(headers) =  body.headers.as_mut()
            {
                if let Some(h_pos) = headers.iter_mut().position(|p|&p.id == id)
                {
                    return SearchResult::Headers(h_pos, headers);
                }
                for h in headers.iter_mut()
                {
                    if let Some(i) = h.items.as_mut()
                    {
                        if let Some(item) = search_item_array(i, id)
                        {
                            return item;
                        }
                    }
                    if let Some(i) = h.indents.as_mut()
                    {
                        if let Some(indent) = search_indent_array(i, id)
                        {
                            return indent;
                        }
                    }
                }
            }
        }
        let err = ["Id ", id,  " не обнаружено ни в одном массиве"].concat();
        error!("{}", &err);
        return SearchResult::NotFound(err);
    }
}

fn search_item_array<'a>(items: &'a mut Vec<Item>, id: &str) -> Option<SearchResult<'a>>
{
    if let Some(pos) = items.iter_mut().position(|p|&p.id == id)
    {
        return Some(SearchResult::Items(pos, items));
    }
    for i in items.iter_mut().enumerate()
    {
        if let Some(ind) = i.1.indents.as_mut()
        {
            let ind_array = search_indent_array(ind, id);
            if ind_array.is_some()
            {
                return ind_array;
            }
        }
        if i.1.items.is_some()
        {
            let s = search_item_array(i.1.items.as_mut().unwrap(), id);
            if s.is_some()
            {
                return s;
            }
        }
    }   
    None
}


fn search_item<'a>(items: &'a mut Vec<Item>, id: &str) -> Option<SearchResult<'a>>
{
    for i in items.iter_mut()
    {
        if &i.id == &id
        {
            return Some(SearchResult::Item(i));
        }
        if let Some(ind) = i.indents.as_mut()
        {
            if let Some(ind) = search_indent(ind, id)
            {
                return Some(SearchResult::Indent(ind));
            }
        }
        if i.items.is_some()
        {
            let s = search_item(i.items.as_mut().unwrap(), id);
            if s.is_some()
            {
                return s;
            }
        }
    }
    None
}

fn search_indent<'a>(indents: &'a mut Vec<Indent>, id: &str) -> Option<&'a mut Indent>
{
    for i in indents.iter_mut()
    {
        if &i.id == &id
        {
            return Some(i);
        }
    }
    None
}
fn search_indent_array<'a>(indents: &'a mut Vec<Indent>, id: &str) -> Option<SearchResult<'a>>
{
    for i in indents.iter_mut().enumerate()
    {
        if &i.1.id == &id
        {
            return Some(SearchResult::Indents(i.0, indents));
        }
    }
    None
}





#[cfg(test)]
mod test
{
   
    use std::{io::Write, path::Path};

    use format_constructor::StructureAdder;
    use format_structure::{Root, to_json, Number, Indent, Item, write_json};
    use logger::info;
    use serde::Serialize;
    use serde_json::Value;
    use crate::{json_path::{Search, SearchResult}, json_path_creator::JsonPathCreator};
    use super::{JsonPathSearch};
    use jsonpath_rust::{JsonPathFinder, JsonPathQuery, JsonPathInst, JsonPathValue};
    

    
    #[test]
    fn test_macros()
    {
        logger::StructLogger::initialize_logger();
        let r = Root::from_file("/hard/xar/projects/rust/format/format_struct/format_parser/test_document.json").unwrap();
        let root_value = r.get_value();
        let selector = root_value.path(r#"$.body.headers[?(@.number.val == '5')].items[?(@.number.val == '6')].items[?(@.number.val == '2')].items[?(@.number.val == 'б')]"#);
        //let jsonpath_exp =  jsonpath_lib::select(&root_value, r#"$.body.headers[?(@.number.val == 5)].items[?(@.number.val == 6)].items[?(@.number.val == 2)].items[?(@.number.val == "б")]"#);
        info!("{:?}", selector.unwrap());
    }

    #[test]
    fn test_struct()
    {
        logger::StructLogger::initialize_logger();
        let r = Root::from_file("/hard/xar/projects/rust/format/format_struct/format_parser/test_document.json").unwrap();
        let mut searcher = JsonPathSearch::new(r);
        let mut path = JsonPathCreator::new_with_header("статью", Some("5"));
        path
        .add_item(Some("6"))
        .add_item(Some("2"))
        .add_item(Some("б"))
        .add_indent(Some(1));
        let p = path.get_body_jsonpath(); 
        let root_value = searcher.search(p);
        match root_value
        {
            SearchResult::Header(h) => info!("id {} обнаружен в хедере", h.id),
            SearchResult::Indent(i) => info!("id {} обнаружен в абзаце", i.id),
            SearchResult::Item(i) => info!("id {} обнаружен в итеме", i.id),
            SearchResult::NotFound(s) => info!("ошибка поиска элемента {}", s),
            _ => info!("Дальше ничего не интересно в этом тесте",),
        }
    }

    #[test]
    fn test_search()
    {
        logger::StructLogger::initialize_logger();
        //let r = Root::from_file("/home/phobos/projects/rust/universal_format/format_constructor/changes_parser/actualized.json").unwrap();
        let r = Root::from_file("/hard/xar/projects/rust/format/format_struct/changes_parser/actualized.json").unwrap();
        //let json: Value = serde_json::from_str(&r.get_json()).unwrap();
        let json: Value = serde_json::from_str(&r.get_json()).unwrap();
        //let v = json.path("$.body.headers[?(@.number.val =='1' && @.type == 'статья')]").unwrap();
        let v = json.path("$.body.headers[?(@.number.val =='1')]").unwrap();
        assert_eq!(true, v.as_array().unwrap().len() == 2);
        //let mut searcher = JsonPathSearch::new(r);
        //let root_value = searcher.search("$.body.headers[?(@.number.val == \"1\" && @.type == \"статья\")]");
        //match root_value
        //{
        //    SearchResult::Header(h) => info!("id {} обнаружен в хедере", h.id),
        //    SearchResult::Indent(i) => info!("id {} обнаружен в абзаце", i.id),
        //    SearchResult::Item(i) => info!("id {} обнаружен в итеме", i.id),
        //    SearchResult::NotFound(s) => info!("ошибка поиска элемента {}", s),
        //    _ => info!("Дальше ничего не интересно в этом тесте",),
        //}
    }

    #[test]
    fn test_search_header_array()
    {
        logger::StructLogger::initialize_logger();
        let mut searcher = JsonPathSearch::from_file("/hard/xar/projects/rust/format/format_struct/format_parser/test_document.json").unwrap();
        //let r = Root::from_file("/hard/xar/projects/rust/format/format_struct/format_parser/test_document.json").unwrap();
        //let mut searcher = JsonPathSearch::new(r);
        let mut path = JsonPathCreator::new_with_header("статью", Some("5"));
        path
        .add_item(Some("6"))
        .add_item(Some("2"))
        .add_item(Some("б"));
        let p = path.get_body_jsonpath(); 
        let root_value = searcher.get_parent_array(p);
        match root_value
        {
            SearchResult::Items(i, vec) => info!("Искомый элемент обнаружен по индексу {} в векторе {:?}", i, serde_json::to_string_pretty(vec).unwrap()),
            SearchResult::NotFound(s) => info!("ошибка поиска элемента {}", s),
            _ => info!("Дальше ничего не интересно в этом тесте",),
        }
    }

    #[test]
    fn test_add_indent_after_self()
    {
        logger::StructLogger::initialize_logger();
        let mut searcher = JsonPathSearch::from_file("/hard/xar/projects/rust/format/format_struct/format_parser/test_document.json").unwrap();
        //после пункта б дополнить пунктом б^1 следующего содержания:
        //вставка тестового пункта
        let number = Number::new("б^1", ")");
        let mut indent = Indent::default();
        indent.add_text("вставка ТЕСТОВОГО содержимого");
        let mut path = JsonPathCreator::new_with_header("статью", Some("5"));
        path
        .add_item(Some("6"))
        .add_item(Some("2"))
        .add_item(Some("б"));
        let p = path.get_body_jsonpath(); 
        let hierarchy = path.get_hierarchy_item_query();
        let mut item = Item::new(number, hierarchy, true, None);
        item.add_element(indent);
        searcher.add_item_after_self(p, item);
        // let root_value = searcher.search_array(p);
        // match root_value
        // {
        //     SearchResult::Items(i, vec) => 
        //     {
        //         info!("Искомый элемент обнаружен по индексу {} в векторе {:?}", i, serde_json::to_string_pretty(vec).unwrap());
        //         vec.insert(i+1, item);
        //         info!("Добавлен новый элемент в вектор {:?}", serde_json::to_string_pretty(vec).unwrap());
        //     },
        //     SearchResult::NotFound(s) => info!("ошибка поиска элемента {}", s),
        //     _ => info!("Дальше ничего не интересно в этом тесте",),
        // }
        write_json(searcher.document, "adding_new_item.json");
    }
}