use std::sync::Mutex;

use format_constructor::DateTimeFormat;
use format_parser::{TextExtractor, IpsSource, Parser, ItemParser, ItemsHierarchy, HtmlExtractor, StringExtractor};
use format_structure::{Root, write_json, Date, Header, Item, Indent};
use logger::{LevelFilter, error, backtrace};
use nom::{sequence::{separated_pair, pair}, character::complete::{space0, digit1, space1}, bytes::complete::tag, IResult, branch::alt, combinator::eof};
use once_cell::sync::Lazy;
use scraper::{Selector, Element, selector::CssLocalName};
use serde::{Serialize, de::Error};

use crate::{error::{CustomNomError, ChangesParserError, self}, parsers::{TargetDocument, Deserializer, ChangeAction, ChangesMap, RemainTokens}, ChangesHierarchy, json_path::{JsonPathSearch, Search, SearchResult}, json_path_creator::JsonPathCreator, format_extensions::ChangeOperations};

pub trait ActualizerExtension
{
    fn apply_changes_map<'a>(self, changes_map: Vec<ChangesMap>) -> Result<Self, Vec<String>> where Self: Sized;
}

impl ActualizerExtension for Root
{
    fn apply_changes_map<'a>(self, changes_map: Vec<ChangesMap>) -> Result<Self, Vec<String>>
    {
        let mut jp = JsonPathSearch::new(self);
        let mut errors: Vec<String> = vec![];
        for change in &changes_map
        {
            let path = &change.path;
            for opertaion in &change.operations
            {
                match opertaion 
                {
                    ChangeAction::ReplaceWords(from, to) =>
                    {
                        //все ерроры перевел в строку, потому что не даст вернуть еррор с тем же временем жизни что и мутабл jsonpath
                        //это уже конечная функция так что не принципиально
                        let r = replace_words(&mut jp, path, from, to);
                        if r.is_err()
                        {
                            errors.push(r.err().unwrap().to_string());
                        }
                    },
                    ChangeAction::AddWords(from, to) =>
                    {
                        let r = add_words(&mut jp, path, from, to);
                        if r.is_err()
                        {
                            errors.push(r.err().unwrap().to_string());
                        }
                    },
                    ChangeAction::ExcludeWords(ex) =>
                    {
                        let r = exclude_words(&mut jp, path, ex);
                        if r.is_err()
                        {
                            errors.push(r.err().unwrap().to_string());
                        }
                    },
                    ChangeAction::LostPower(i) =>
                    {
                        //TODO сделать метку is_lost_power!!
                        let todo = "";
                    },
                    ChangeAction::Exclude(number) =>
                    {
                        let r = exclude_items(&mut jp, path, number, true);
                        if r.is_err()
                        {
                            errors.push(r.err().unwrap().to_string());
                        }
                    }
                    ChangeAction::HeaderInNewEdition(h) =>
                    {
                        //изменение происходит сразу в теле доку, так что можно просто найти его и поменять без jsonpath
                        let r = replace_header(&mut jp, h);
                        if r.is_err()
                        {
                           errors.push(r.err().unwrap().to_string());
                        }
                    },
                    ChangeAction::HeaderNameInNewEdition(name)=>
                    {
                        let header = jp.search(&path.get_body_jsonpath());
                        match header
                        {
                            SearchResult::Header(h)=>
                            {
                                logger::info!("Заменено наименование заголовка с |{}| на |{}|",&h.name, &name.name);
                                h.name = name.name.clone();
                                h.is_change = true; 
                            },
                            SearchResult::NotFound(e)=>
                            {
                                let err = format!("Ошибка поиска заголовка {} для замены наименования {}",&path.get_body_jsonpath(), &name.name);
                                logger::error!("{}", &err);
                                errors.push(err);
                            }
                            _=> () //errors.push(ChangesParserError::Error(format!("Для объекта не определена логика обработки->{}", backtrace!())))
                        }
                    },
                    ChangeAction::ApplyItems(i)=>
                    {
                        let r = add_items(&mut jp, path, i);
                        if r.is_err()
                        {
                            errors.push(r.err().unwrap().to_string());
                        }
                    }
                    ChangeAction::ItemsInNewEdition(i)=>
                    {
                        let r = replace_items(&mut jp, path, i);
                        if r.is_err()
                        {
                            errors.push(r.err().unwrap().to_string());
                        }
                    }
                    ChangeAction::IndentsInNewEdition(i)=>
                    {
                        let r = replace_indents(&mut jp, path, i);
                        if r.is_err()
                        {
                            errors.push(r.err().unwrap().to_string());
                        }
                    }
                    ChangeAction::ApplySentence(s) =>
                    {
                        let r = add_sentence(&mut jp, path, s);
                        if r.is_err()
                        {
                            errors.push(r.err().unwrap().to_string());
                        }
                    },
                    _ => ()
                }
            }
        }
        let doc = jp.get_document();
        Ok(doc.to_owned())
    }
}


pub struct Actualizer
{

}

impl Actualizer
{
    ///получение списка изменений из изменяющего документа
    fn get_changes_hierarchy_from_ips_document(number: &str, date: &str) -> Vec<ChangesHierarchy>
    {
        let result = Parser::get_html::<IpsSource>(number, date, None).unwrap();
        let elements = result.get_main_element_from_html();
        let mut headers : Vec<Vec<TextExtractor>> = vec![];
        let t_classes_selector = Selector::parse("p:not(.C):not(.T):not(.I)").unwrap();
        let header_classes_selector = Selector::parse("p.H").unwrap();
        let have_headers = elements.select(&header_classes_selector).count() > 0;
        for selected_element in elements.select(&t_classes_selector)
        {
            let raw_text = &selected_element.text().collect::<Vec<_>>().join("");
            if !raw_text.trim().is_empty()
            {
                let is_change = crate::parsers::check_if_change(raw_text);
                let obj = <TextExtractor as HtmlExtractor>::extract(&selected_element, is_change);
                if !have_headers && headers.len() == 0
                {
                    headers.push(vec![]);
                }
                if selected_element.has_class(&CssLocalName::from("H"), scraper::CaseSensitivity::CaseSensitive) && change_article(&raw_text)
                {
                    headers.push(vec![]);
                    continue;
                }
                headers.last_mut().unwrap().push(obj);
            }
        }
        let mut errors: Vec<Vec<TextExtractor>> = vec![];
        let mut hierarhy: Vec<ChangesHierarchy> = vec![];
        let mapped = headers.iter().map(|m| ChangesHierarchy::get_changes_hierarchy(m)).collect::<Vec<Result<ChangesHierarchy, Vec<TextExtractor>>>>();
        for m in mapped
        {
            if m.is_ok()
            {
                hierarhy.push(m.unwrap());
            }
            else 
            {
                errors.push(m.err().unwrap());    
            }
        }
        logger::info!("В документе №{} от {} найдено {} частей с изменениями, из них распознаны как изменения {} частей", number, date, hierarhy.len() + errors.len(), hierarhy.len());
        hierarhy
    }

    ///получение карты изменений из указанного текcтового вектора, на случай если это ручная вставка оператором
    // pub fn get_partial_changes_map(text: Vec<&str>) -> Result<(Vec<ChangesMap>, Vec<RemainTokens>), ChangesParserError>
    // {
    //     let mut extractor: Vec<TextExtractor> = vec![];
    //     for l in text
    //     {
    //         let is_change = crate::parsers::check_if_change(l);
    //         let ex = <TextExtractor as StringExtractor>::extract(l, is_change);
    //         extractor.push(ex);
    //     }
    //     if let Ok(hierarchy) = ChangesHierarchy::get_changes_hierarchy(&extractor)
    //     {
    //         let map = ChangesMap::parse(hierarchy);
    //         map
    //     }
    //     else 
    //     {
    //         return Err(ChangesParserError::Error("Ошибка получения иерархии изменений, подробности в лог файле.".to_owned()))    
    //     }
        
    // }
    ///Получение полной карты изменений из указанного документа
    pub fn get_full_changes_map<'a>(number: &str, date: &str) -> Result<Vec<(Option<TargetDocument>, Vec<ChangesMap>, Vec<RemainTokens>)>, ChangesParserError<'a>>
    {
        let mut changes : Vec<(Option<TargetDocument>, Vec<ChangesMap>, Vec<RemainTokens>)> = vec![];
        let source_document = Actualizer::get_changes_hierarchy_from_ips_document(number, date);
        for change in source_document
        {
            let target = change.get_target_document().cloned();
            let structure = ChangesMap::parse(change)?;
            changes.push((target, structure.0, structure.1));
        }
        Ok(changes)
    }
    pub fn apply_changes_map_from_file<'a>(path: &'a str) -> Result<Root, Vec<String>>
    {
        let mut errors: Vec<String> = vec![];
        let des = Deserializer::load(path) as Result<Box<(Option<TargetDocument>, Vec<ChangesMap>)>, Box<dyn std::error::Error>>;
        //logger::error!("{}", des.as_ref().err().unwrap().to_string());
        if let Ok(map) = des
        {
            if let Some(target) = map.0
            {
                if let Ok(dt) = match target.document_type 
                {
                    crate::parsers::DocumentType::FederalLaw(date, number) =>
                    {
                        let date = *Date::parse(&date).unwrap();
                        let dot_date = date.write(format_structure::DateFormat::DotDate);
                        //454-ФЗ e,bhftv -ФЗ
                        let number = number.split_once("-").unwrap().0;
                        //Parser::parse_html::<IpsSource>("454", "30.12.2021", None)
                        Parser::parse_html::<IpsSource>(number, &dot_date, None)
                    }
                    crate::parsers::DocumentType::Kodex(_name) =>
                    {
                        Parser::parse_html::<IpsSource>("000", "30.12.2021", None)
                    }
                }
                {
                    return dt.document.apply_changes_map((*map).1);
                }
                else 
                {
                    let err = ChangesParserError::TargetDocumentInfo(path);
                    error!("{}", &err);
                    errors.push(err.to_string());
                }
            }
            else 
            {
                let err = ChangesParserError::TargetDocumentInfo(path);
                error!("{}", &err);
                errors.push(err.to_string());
            }   
        }
        else 
        {
            let err = ChangesParserError::ChangesMapFilePath(path);
            error!("{}", &err);
            errors.push(err.to_string());
        }
        return Err(errors);
    }
    // pub fn actualize_local_map_file(number: &str, date: &str)
    // {
    //     let source_document = Actualizer::get_source_document_changes(number, date);
    //     for change in source_document
    //     {
    //         let structure = parse_changes(change);

    //     }
    // }
}




fn replace_words<'a>(searcher: &mut JsonPathSearch, path: &JsonPathCreator, from: &str, to: &str) 
-> Result<(), ChangesParserError<'a>>
{
    let jp_item = searcher.search(&path.get_body_jsonpath());
    let error_string = ["Операция замены слов ", "|", from, "|", " на ", "|", to, "| по адресу ", &path.get_body_jsonpath(), " не выполнена!"].concat();
    match jp_item
    {
        SearchResult::Header(h) =>
        {
            if let Some(replaced) = h.replace_words(from, to)
            {
                logger::info!("В {}| слова {} заменены словами {}",&path.get_body_jsonpath(), from, to);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
        },
        SearchResult::Item(i) =>
        {
            if let Some(replaced) = i.replace_words(from, to)
            {
                logger::info!("В {}| слова {} заменены словами {}",&path.get_body_jsonpath(), from, to);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
        },
        SearchResult::Indent(i) =>
        {
            if let Some(replaced) = i.replace_words(from, to)
            {
                logger::info!("В {}| слова {} заменены словами {}",&path.get_body_jsonpath(), from, to);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
            
        },
        SearchResult::NotFound(e) =>
        {
            let err = ChangesParserError::JsonpathNotFound(e);
            error!("{}->{}", &err, backtrace!());
            Err(err)
        }
        _ => Err(ChangesParserError::Error(format!("Для объекта не определена логика обработки->{}", backtrace!())))
    }
}
//FIXME  это необходитмо отладить!
fn replace_indents<'a>(searcher: &mut JsonPathSearch, path: &JsonPathCreator, indents: &Vec<Indent>) 
-> Result<(), ChangesParserError<'a>>
{
    let jp_item = searcher.search(&path.get_body_jsonpath());
    let error_string = ["Операция замены абзацев", " по адресу ", &path.get_body_jsonpath(), " не выполнена!"].concat();
    match jp_item
    {
        SearchResult::Indent(i) =>
        {
            logger::info!("ТЕСТ ТАРГЕТА АБЗАЦА ДЛЯ ЗАМЕНЫ {}, {}",&path.get_body_jsonpath(), &i.body);
            Ok(())
            // if let Some(replaced) = i.replace_words(from, to)
            // {
            //     logger::info!("В {}| слова {} заменены словами {}",&path.get_body_jsonpath(), from, to);
            //     Ok(())
            // }
            // else
            // {
            //     error!("{}->{}", &error_string, backtrace!());
            //     let err = ChangesParserError::OperationError(error_string);
            //     Err(err)
            // }
            
        },
        SearchResult::NotFound(e) =>
        {
            let err = ChangesParserError::JsonpathNotFound(e);
            error!("{}->{}", &err, backtrace!());
            Err(err)
        }
        _ => Err(ChangesParserError::Error(format!("Для объекта не определена логика обработки->{}", backtrace!())))
    }
}

fn add_words<'a>(searcher: &mut JsonPathSearch, path: &JsonPathCreator, from: &Option<String>, to: &str) 
-> Result<(), ChangesParserError<'a>>
{
    let jp_item = searcher.search(&path.get_body_jsonpath());
    let error_string = match from
    {
        Some(f) => ["Операция добавления слов ", "|", to, "|", " после слов ", "|", f, "| по адресу ", &path.get_body_jsonpath(), " не выполнена!"].concat(),
        None => ["Операция добавления слов ", "|", to, "|", " по адресу ", &path.get_body_jsonpath(), " не выполнена!"].concat()
    };
    match jp_item
    {
        SearchResult::Header(h) =>
        {
            if let Some(added) = h.add_words(from, to)
            {
                logger::info!("Заголовок {}->{} дополнен словами {}",&path.get_body_jsonpath(), h.number.to_string(), to);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
        },
        SearchResult::Item(i) =>
        {
            if let Some(added) = i.add_words(from, to)
            {
                logger::info!("Пункт {}->{} дополнен словами {}",&path.get_body_jsonpath(), i.number.to_string(), to);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
        },
        SearchResult::Indent(i) =>
        {
            if let Some(added) = i.add_words(from, to)
            {
                logger::info!("Абзац {} дополнен словами {}",&path.get_body_jsonpath(), to);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
        },
        SearchResult::NotFound(e) =>
        {
            let err = ChangesParserError::JsonpathNotFound(e);
            error!("{}->{}", &err, backtrace!());
            Err(err)
        }
        _ => Err(ChangesParserError::Error(format!("Для объекта не определена логика обработки->{}", backtrace!())))
    }
}
fn add_sentence<'a>(searcher: &mut JsonPathSearch, path: &JsonPathCreator, sentence: &str) 
-> Result<(), ChangesParserError<'a>>
{
    let jp_item = searcher.search(&path.get_body_jsonpath());
    let error_string = ["Операция добавления предложения ", "|", sentence, "|", " по адресу ", &path.get_body_jsonpath(), " не выполнена!"].concat();
    match jp_item
    {
        SearchResult::Header(h) =>
        {
            if let Some(added) = h.add_words(&None, sentence)
            {
                logger::info!("Заголовок {}->{} дополнен предложением {}",&path.get_body_jsonpath(), h.number.to_string(), sentence);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
        },
        SearchResult::Item(i) =>
        {
            if let Some(added) = i.add_words(&None, sentence)
            {
                logger::info!("Пункт {}->{} дополнен предложением {}",&path.get_body_jsonpath(), i.number.to_string(), sentence);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
        },
        SearchResult::Indent(i) =>
        {
            if let Some(added) = i.add_words(&None, sentence)
            {
                logger::info!("Абзац {} дополнен предложением {}",&path.get_body_jsonpath(), sentence);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
        },
        SearchResult::NotFound(e) =>
        {
            let err = ChangesParserError::JsonpathNotFound(e);
            error!("{}->{}", &err, backtrace!());
            Err(err)
        }
        _ => Err(ChangesParserError::Error(format!("Для объекта не определена логика обработки->{}", backtrace!())))
    }
}
//после замены надо проверить не итем ли это и переместить в итемы если он стал итемом
//18) в статье 31:
//а) слова "Сорта и гибриды" заменить словами "1. Сорта и гибриды";
//б) дополнить частью 2 следующего содержания:
fn exclude_words<'a>(searcher: &'a mut JsonPathSearch, path: &JsonPathCreator, ex: &'a str) 
-> Result<(), ChangesParserError<'a>>
{
    let jp_item = searcher.search(&path.get_body_jsonpath());
    let error_string = ["Операция исключения слов ", "|", ex, "|", " по адресу ", &path.get_body_jsonpath(), " не выполнена!"].concat();
    return match jp_item
    {
        SearchResult::Header(h) =>
        {
            if let Some(excluded) = h.exclude_words(ex, true)
            {
                logger::info!("Из статьи {}->{} слова {} исключены",&path.get_body_jsonpath(), h.number.to_string(), ex);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
        },
        SearchResult::Item(i) =>
        {
            if let Some(excluded) = i.exclude_words(ex, true)
            {
                logger::info!("Из пункта {}->{} слова {} исключены",&path.get_body_jsonpath(), i.number.to_string(), ex);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }
        },
        SearchResult::Indent(i) =>
        {
            if let Some(excluded) = i.exclude_words(ex, true)
            {
                logger::info!("Из абзаца {} слова {} исключены",&path.get_body_jsonpath(), ex);
                Ok(())
            }
            else
            {
                error!("{}->{}", &error_string, backtrace!());
                let err = ChangesParserError::OperationError(error_string);
                Err(err)
            }   
        },
        SearchResult::NotFound(e) =>
        {
            let err = ChangesParserError::JsonpathNotFound(e);
            error!("{}->{}", &err, backtrace!());
            Err(err)
        }
        _ => Err(ChangesParserError::Error(format!("Для объекта не определена логика обработки->{}", backtrace!())))
    }
}

fn replace_header<'a>(searcher: &'a mut JsonPathSearch, header: &'a Header) -> Result<(), ChangesParserError<'a>>
{
    searcher.replace_header(header)
}

fn add_items<'a>(searcher: &'a mut JsonPathSearch, path: &JsonPathCreator, items: &'a Vec<Item>) 
-> Result<(), ChangesParserError<'a>>
{
    let jp_item = searcher.search(&path.get_body_jsonpath());
    let numbers = items.iter().map(|itm| itm.number.to_string()).collect::<Vec<String>>().join(" ");
    //let error_string = ["Операция добавления пунктов ", "|", &numbers, "|", " по адресу ", &path.get_body_jsonpath(), " не выполнена!"].concat();
    match jp_item
    {
        SearchResult::Header(h) =>
        {
            match h.items.as_mut()
            {
                Some(i) =>
                {
                    i.extend(items.clone());
                    i.sort();
                    logger::info!("Дополнены пункты {} по адресу: {}.", &numbers, &path.get_body_jsonpath());
                    Ok(())
                },
                None => 
                {
                    h.items = Some(items.clone());
                    Ok(())
                }
            }
        },
        SearchResult::NotFound(e) =>
        {
            let err = ChangesParserError::JsonpathNotFound(e);
            error!("{}->{}", &err, backtrace!());
            Err(err)
        }
        _ => Err(ChangesParserError::Error(format!("Для объекта не определена логика обработки->{}", backtrace!())))
    }
}

fn replace_items<'a>(searcher: &'a mut JsonPathSearch, path: &JsonPathCreator, items: &'a Vec<Item>) 
-> Result<(), ChangesParserError<'a>>
{
    let jp_item = searcher.search(&path.get_body_jsonpath());
    let numbers = items.iter().map(|itm| itm.number.to_string()).collect::<Vec<String>>().join(" ");
    let error_string = ["Операция замены пунктов ", "|", &numbers, "|", " по адресу ", &path.get_body_jsonpath(), " не выполнена!"].concat();
    return match jp_item
    {
        SearchResult::Header(h) =>
        {
            let jsonpath = &path.get_body_jsonpath();
            for item in items
            {
                if let Some(itm) = h.items.as_mut().and_then(|i| i.into_iter().find(|f| f == &item))
                {
                    itm.replace(item);
                    logger::info!("Заменен пункт {}->{}",jsonpath, &item.number);
                }
                else 
                {
                    error!("{}->{}", &error_string, backtrace!());
                    let err = ChangesParserError::OperationError(error_string);
                    return Err(err);
                }
            }
            Ok(())
        },
        SearchResult::NotFound(e) =>
        {
            let err = ChangesParserError::JsonpathNotFound(e);
            error!("{}->{}", &err, backtrace!());
            Err(err)
        }
        _ => Err(ChangesParserError::Error(format!("Для объекта не определена логика обработки->{}", backtrace!())))
    }
}

fn exclude_items<'a>(searcher: &'a mut JsonPathSearch, path: &JsonPathCreator, number: &str, mark: bool) 
-> Result<(), ChangesParserError<'a>>
{
    let jp_item = searcher.search(&path.get_body_jsonpath());
    let error_string = ["Операция исключения объекта ", "|", &number, "|", " по адресу ", &path.get_body_jsonpath(), " не выполнена!"].concat();
    return match jp_item
    {
        SearchResult::Header(h) =>
        {
            let jsonpath = &path.get_body_jsonpath();
            if mark
            {
                if let Some(itm) = h.items.as_mut().and_then(|i| i.into_iter().find(|f| &f.number.val == &number))
                {
                    itm.set_is_lost_power(true);
                    logger::info!("Пункт {} утратил силу->{}",&number, jsonpath);
                }
                else 
                {
                    error!("{}->{}", &error_string, backtrace!());
                    let err = ChangesParserError::OperationError(error_string);
                    return Err(err);
                }
            }
            else 
            {
                if let Some(vecs) = h.items.as_mut().and_then(|a|Some(a))
                {
                    vecs.retain(|r| &r.number.val != number);
                }
                else 
                {
                    error!("{}->{}", &error_string, backtrace!());
                    let err = ChangesParserError::OperationError(error_string);
                    return Err(err);
                }
            }
            Ok(())
        },
        SearchResult::NotFound(e) =>
        {
            let err = ChangesParserError::JsonpathNotFound(e);
            error!("{}->{}", &err, backtrace!());
            Err(err)
        }
        _ => Err(ChangesParserError::Error(format!("Для объекта не определена логика обработки->{}", backtrace!())))
    }
}


fn change_article(s: &str) -> bool
{
    let p: IResult<&str, (&str, &str), CustomNomError<&str>> = separated_pair(tag("Статья"), space0, digit1)(s);
    if p.is_ok()
    {
        let s: IResult<&str, &str, CustomNomError<&str>> = alt((eof, space1))(p.as_ref().unwrap().0);
        return s.is_ok();
    }
    false
}

#[cfg(test)]
mod tests
{
    use std::collections::HashMap;

    use format_constructor::RootExtension;
    use format_parser::{IpsSource, Parser, HeaderParser, ItemParser, TextExtractor};
    use format_structure::{write_json, Root};
    use logger::LevelFilter;
    use scraper::{Selector, selector::CssLocalName, Element};

    use crate::parsers::TargetDocument;

    use super::ActualizerExtension;
    #[test]
    fn test_parse_changed_doc()
    {
        logger::StructLogger::init(LevelFilter::Info);
        let groups = super::Actualizer::get_changes_hierarchy_from_ips_document("485", "04.08.2023");
        for (i, item) in groups.iter().enumerate()
        {
            let filename = ["items_with_changes_", &i.to_string()].concat();
            write_json(item, &filename);
        }
    }

    #[test]
    fn test_get_changes_map()
    {
        logger::StructLogger::init(LevelFilter::Info);
        let number = "488";
        let groups = super::Actualizer::get_full_changes_map(&number, "04.08.2023").unwrap();
        for (i, item) in groups.iter().enumerate()
        {
            let filename = ["changes_map_", &number, "_", &i.to_string()].concat();
            write_json((&item.0, &item.1), &filename);
        }
    }
    #[test]
    fn test_actualizer()
    {
        logger::StructLogger::init(LevelFilter::Info);
        //let path = "/hard/xar/projects/rust/format/format_struct/changes_parser/changes_map_0.json";
        let path = "/home/phobos/projects/rust/universal_format/format_constructor/changes_parser/changes_map_488_0.json";
        let actualized = super::Actualizer::apply_changes_map_from_file(path);
        let filename = "actualized".to_owned();
        let doc = actualized.unwrap();
        write_json(doc, &filename);
    }

    #[test]
    fn test_actualizer1()
    {
        logger::StructLogger::init(LevelFilter::Info);
        let test_data = vec![
            "19) части 2 и 3 статьи 32 изложить в следующей редакции:",
            "\"2. Части 1, 3 - 8 статьи 19, части 1 - 5 и 9 статьи 20, статья 21 и части 5 и 6 статьи 22 настоящего Федерального закона вступают в силу с 1 сентября 2024 года.",
            "3. Часть 7 статьи 12, части 6 - 8, 10 и 11 статьи 20 настоящего Федерального закона вступают в силу с 1 сентября 2025 года.\"."
        ];
        let groups = super::ChangesMap::get_changes_map(test_data).unwrap();
        let target_doc = Root::deserialize("/hard/xar/projects/rust/format/format_struct/changes_parser/source/454_30_12_2021.json").unwrap();
        //let target_doc = Root::deserialize("/home/phobos/projects/rust/universal_format/format_constructor/changes_parser/source/454_30_12_2021.json").unwrap();
        let act = target_doc.apply_changes_map(groups.0);
        if let Ok(r) = act
        {
            write_json(r, "test_actualizer");
        }
        else 
        {
            write_json(act.err().unwrap(), "test_actualizer1_errors");    
        }
    }
    //TODO сделать что то вроде тестового вывода для флага is_shange
    //потому что он не выводит ошибки парсера просто тупо ставит флаг что это не изменение,
    //поэтому надо чтобы оператор отслеживал все ли флаги проставлены верно
    #[test]
    fn test_actualizer2()
    {
        logger::StructLogger::init(LevelFilter::Info);
        let test_data = vec![
            "6) в статье 12:",
            "а) наименование изложить в следующей редакциии:",
            "\"Статья 12. Производство семян сельскохозяйственных растений\";",
            "б) часть 7 изложить в следующей редакции:",
            "\"7. Пространственная изоляция к сельскохозяйственному производству, за исключением установленной правом Евразийского экономического союза, утверждается федеральным органом исполнительной власти, осуществляющим функции по выработке государственной политики и нормативно-правовому регулированию в области семеноводства сельскохозяйственных растений.\";",
            "в) дополнить частью 8 следующего содержания:",
            "\"8. Для производства семян сельскохозяйственных растений могут устанавливаться специальные семеноводческие зоны в порядке, определенном законом субъекта Российской Федерации.",
            "В отношении земельных участков, принадлежащих на праве собственности физическим лицам, в том числе индивидуальным предпринимателям, или юридическим лицам или предоставленным им в пользование, специальные семеноводческие зоны устанавливаются на основании заявлений указанных лиц.\";"
        ];
        let groups = super::ChangesMap::get_changes_map(test_data).unwrap();
        let target_doc = Root::deserialize("/hard/xar/projects/rust/format/format_struct/changes_parser/source/454_30_12_2021.json").unwrap();
        //let target_doc = Root::deserialize("/home/phobos/projects/rust/universal_format/format_constructor/changes_parser/source/454_30_12_2021.json").unwrap();
        let act = target_doc.apply_changes_map(groups.0);
        if let Ok(r) = act
        {
            write_json(r, "test_actualizer");
        }
        else 
        {
            write_json(act.err().unwrap(), "test_actualizer2_errors");    
        }
    }

    #[test]
    fn test_actualizer3()
    {
        logger::StructLogger::init(LevelFilter::Info);
        let test_data = vec![
            "6) часть 12 исключить;",
        ];
        let groups = super::ChangesMap::get_changes_map(test_data).unwrap();
        let target_doc = Root::deserialize("/hard/xar/projects/rust/format/format_struct/changes_parser/source/454_30_12_2021.json").unwrap();
        //let target_doc = Root::deserialize("/home/phobos/projects/rust/universal_format/format_constructor/changes_parser/source/454_30_12_2021.json").unwrap();
        let act = target_doc.apply_changes_map(groups.0);
      
    }
    
    #[test]
    fn check_changes_flag()
    {
        logger::StructLogger::init(LevelFilter::Info);
        let result = Parser::get_html::<IpsSource>("473", "04.08.2023", None).unwrap();
        let elements = result.get_main_element_from_html();
        //все ноды p
        //"p:not(.C):not(.T):not(.I)"
        //TODO доделать и другие элементы, например таблицу он не увидит
        let t_classes_selector = Selector::parse("p:not(.C):not(.T):not(.I)").unwrap();
        let mut changes: Vec<String> = vec![];
        //один из элемнтов селектора параграф в данном случае
        for selected_element in elements.select(&t_classes_selector)
        {
            let raw_text = &selected_element.text().collect::<Vec<_>>().join("");
            if !raw_text.trim().is_empty()
            {
                let is_change = crate::parsers::check_if_change(raw_text);
                if !is_change
                {
                    changes.push(raw_text.to_owned());
                }
            }
        }
        write_json(changes, "only_changes_text");
        //super::super::super::write_string_to_file(&result.join("\n"), "478-fz");
    }
}