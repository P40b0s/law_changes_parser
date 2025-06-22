use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}, ops::Index, rc::Rc};

use nom::
{
    branch::alt, bytes::complete::{is_a, tag, tag_no_case}, combinator::map, sequence::pair, IResult, Parser
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::{change_action::ChangeAction, change_path::{ChangePath, TargetPath}, error::ParserError, objects::{number::{Number}, remain_tokens::RemainTokens}, parsers::{paths, space0}};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Action
{
    Words,
    Replace,
    Apply
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChangeType
{
    Text,
    Html,
    None
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Change
{
    pub target_path: TargetPath,
    pub changes: Option<Vec<ChangeAction>>,
    pub action_after_path: Option<TargetPath>,
    pub action: Action,
    pub change_type: ChangeType,
    pub text_changes: Option<Vec<String>>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Changes(pub (crate) Vec<Change>);
impl Changes
{
    fn new() -> Self
    {
        Self(Vec::new())
    }
    fn add_apply_directive(&mut self, original_str: &str, remains: &str, after: Option<TargetPath>, target: TargetPath)
    {
        if !remains.is_empty()
        {
            logger::warn!("Считаны не все символы при парсинге изменения `{}` ->`{}`", original_str, remains)
        }
        self.0.push(Change 
        { 
            target_path: target,
            changes: None,
            action_after_path: after,
            action: Action::Apply,
            text_changes: None,
            change_type: ChangeType::None
        });
    }
    fn add_words_directive(&mut self, original_str: &str, remains: &str, target: TargetPath, actions: Vec<ChangeAction>)
    {
        if !remains.is_empty()
        {
            logger::warn!("Считаны не все символы при парсинге изменения `{}` ->`{}`", original_str, remains)
        }
        self.0.push(Change 
        { 
            target_path: target,
            changes: Some(actions),
            action_after_path: None,
            action: Action::Words,
            text_changes: None,
            change_type: ChangeType::None
        });
    }
    fn add_replace_directive(&mut self, original_str: &str, remains: &str, target: TargetPath)
    {
        if !remains.is_empty()
        {
            logger::warn!("Считаны не все символы при парсинге изменения `{}` ->`{}`", original_str, remains)
        }
        self.0.push(Change 
        { 
            target_path: target,
            changes: None,
            action_after_path: None,
            action: Action::Replace,
            text_changes: None,
            change_type: ChangeType::None
        });
    }
    fn try_add_text(&mut self, txt: &str)
    {
        if let Some(last) = self.0.last_mut()
        {
            match last.action
            {
                Action::Apply => 
                {
                    if last.text_changes.is_none()
                    {
                        last.text_changes = Some(Vec::new());
                    }
                    if last.changes.is_none()
                    {
                        last.changes = Some(Vec::new());
                    }
                    let changes = last.text_changes.as_mut().unwrap();
                    let enum_changes = last.changes.as_mut().unwrap();
                    //TODO уточнить изменения в энумах (абзац, пункт, статья...)
                    //enum_changes.push(Cha);
                    changes.push(txt.to_owned());
                    last.change_type = ChangeType::Text;
                },
                Action::Replace => 
                {
                    if last.text_changes.is_none()
                    {
                        last.text_changes = Some(Vec::new());
                    }
                    let changes = last.text_changes.as_mut().unwrap();
                    changes.push(txt.to_owned());
                    last.change_type = ChangeType::Text;
                },
                Action::Words => ()
            }
        }
    }
    pub fn get_changes(text: &str) -> Self
    {
        let mut all_paths = Vec::new();
        let mut changes_list = Self::new();
        for ln in text.lines()
        {
            let result = Self::search_change(ln, &mut changes_list, &mut all_paths);
            if let Some(r) = result
            {
                //logger::debug!("input string `{}` remains tokens `{:?}`", ln, r);
            }
        }
        changes_list
    }
    fn search_change(s: &str, changes_list: &mut Self, all_paths: &mut Vec<ChangePath>) -> Option<RemainTokens>
    {
        //только путь, дальше идет уточнение
        //генерируем глобальный путь для всех изменений
        let only_path_definitions_directive: IResult<&str, TargetPath, ParserError> = super::only_path_definition(s);
        if let Ok((remains, path)) = only_path_definitions_directive
        {
            //добаляем глобальные пути если это выражение типа  
            //2) в статье 20:
            //а) в пункте 2:
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
            let mut tp = target;
            if !all_paths.is_empty()
            {
                tp.insert_paths(all_paths);
                logger::debug!("apply directive: {:?}", &tp)
            }
            changes_list.add_apply_directive(s, remains, after, tp);
            return Some(RemainTokens::new(s, remains));
        }
        //тут изменения в пределах абзаца дополнить словами заменить словами итд.
        let words_directive: IResult<&str, (TargetPath, Vec<ChangeAction>), ParserError> = super::words::words_operations(s);
        if let Ok((remains, (path, actions))) = words_directive
        {
            let mut tp = path;
            if !all_paths.is_empty()
            {
            
                tp.insert_paths(all_paths);
                logger::debug!("words directive: {:?}", &tp)
            }
            changes_list.add_words_directive(s, remains, tp, actions);
            return Some(RemainTokens::new(s, remains));
        }
        //замена чего либо (c нового абзаца и далее)
        let replace_directive: IResult<&str, TargetPath, ParserError> = super::replace_all(s);
        if let Ok((remains, path)) = replace_directive
        {
            let mut tp = path;
            if !all_paths.is_empty()
            {
                tp.insert_paths(all_paths);
                logger::debug!("replace directive: {:?}", &tp)
            }
            changes_list.add_replace_directive(s, remains, tp);
            return Some(RemainTokens::new(s, remains));
        }
        //хз не помню для чего это
        let item_name_directive: IResult<&str, &str, ParserError> = Self::item_name(s);
        if let Ok((remains, xz)) = item_name_directive
        {
            return Some(RemainTokens::new(s, remains));
        }
        changes_list.try_add_text(s);
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
}
///v1 - без рекурсии используя кольцевой буфер
// #[allow(dead_code)]
// fn mermaid_recursion(changes: Vec<Change>, dia: &mut String, changes_count: &mut u32)
// { 
//     let mut queue: VecDeque<(Vec<Change>, Option<(u64, ChangePath)>, usize)> = VecDeque::new();
//     queue.push_back((changes, None, 0));
//     while let Some((current_changes, parent, level)) = queue.pop_front() 
//     {
//         let mut groups = HashMap::new();
//         for change in current_changes.into_iter()
//         {
//             if let Some(path) = change.target_path.get_path_by_level(level)
//             {
//                 groups.entry(path)
//                 .or_insert_with(Vec::new)
//                 .push(change);
//             }
//         }
//         for ((id, cp), ch) in groups.into_iter()
//         {
//             if let Some((p_id, _)) = parent
//             {
//             //проверяем если путь является последним и если он совпадает с текущим то добавляем список изменений в диаграму
//                 if let Some(last_change) = ch.last()
//                 {
//                     if let Some(last_change_path) = last_change.target_path.get_paths().last()
//                     {
//                         logger::debug!("current change: `{:?}` last change `{:?}`", cp, last_change_path);
//                         if &cp == last_change_path
//                         {
//                             if let Some(change_actions) = last_change.changes.as_ref()
//                             {
//                                 *changes_count += change_actions.len() as u32;
//                             }
//                             else 
//                             {
//                                 *changes_count +=1;    
//                             }
//                             let constructor = MermaidDiagrammConstructor::new_with_parent_and_change(&id, &p_id, &cp, last_change);
//                             let rect = constructor.gen_line();
//                             logger::debug!("mermaid line: `{}`", &rect);
//                             dia.push_str(&rect);
//                         }
//                         else 
//                         {
//                             let constructor = MermaidDiagrammConstructor::new_with_parent(&id, &p_id, &cp);
//                             let rect = constructor.gen_line();
//                             dia.push_str(&rect);
//                         }
//                     }
//                 }
//                 //dia.push_str(&format!("  {} --> {}[\"`{} "{}`\"]\n", p_id, id, cp.as_markdown(), &ch_string));
//                 //dia.push_str(&format!("  {} --> {}[{:?}]\n", p_id, id, ch));
//             }
//             else 
//             {
//                 let constructor = MermaidDiagrammConstructor::new(&id, &cp);
//                 let rect = constructor.gen_line();
//                 dia.push_str(&rect);
//                 //dia.push_str(&format!("  {}[\"`{} {}`\"]\n", id, cp.as_markdown(), &ch_string));
//             }
//             if !ch.is_empty()
//             {
//                 queue.push_back((ch, Some((id, cp.clone())), level + 1));
//             }
//             //mermaid_recursion(ch, Some((id, cp)), lvl + 1, dia, changes_count);
//         }
//     }
// }

// impl AsMarkdown for Change
// {
//     fn as_markdown(&self) -> String
//     {
//         let mut md = String::new();
//         match self.action
//         {
//             Action::Apply =>
//             {
//                 if let Some(ap) = self.action_after_path.as_ref()
//                 {
//                     let md_path = ap.as_markdown();
//                     md.push_str("После: ");
//                     md.push_str(&md_path);
//                 }
//                 if let Some(txt) = self.text_changes.as_ref()
//                 {
//                     for t in txt
//                     {
//                         let escaped = escape_quotes(t);
//                         md.push_str(&escaped);
//                         md.push_str("  ");
//                     }
//                 }
//             }
//             Action::Replace => 
//             {
//                 if let Some(ap) = self.action_after_path.as_ref()
//                 {
//                     let md_path = ap.as_markdown();
//                     md.push_str("После: ");
//                     md.push_str(&md_path);
//                     md.push_str("  ");
//                 }
//                 if let Some(txt) = self.text_changes.as_ref()
//                 {
//                     for t in txt
//                     {
//                         let escaped = escape_quotes(t);
//                         md.push_str(&escaped);
//                         md.push_str("  ");
//                     }
//                 }
//             }
//             Action::Words =>
//             {
//                 if let Some(change_actions) = self.changes.as_ref()
//                 {
//                     md.push_str("Изменения слов:  ");
//                     for ca in change_actions
//                     {
//                         match ca
//                         {
//                             ChangeAction::AddWords { after, words } =>
//                             {
//                                 if let Some(af) = after
//                                 {
//                                     md.push_str("после '");
//                                     md.push_str(af);
//                                     md.push_str("' дополнить '");
//                                     md.push_str(words);
//                                     md.push_str("'");
//                                 }
//                             },
//                             _ => md.push_str("Еще не реализовано"),
//                         }
//                     }
//                 }
//             }
//         }
//         md
//     }
// }

#[cfg(test)]
mod tests
{
    use crate::{outputs::MermaidDiagram, parsers::changes_parser::{Changes}, ChangesGraph};

    #[test]
    fn test_changes_parser()
    {
        logger::StructLogger::new_default();
        let test_data = include_str!("..\\..\\test_data\\test_1.txt");
        let changes_list = Changes::get_changes(test_data);
        logger::debug!("{}", serde_json::to_string_pretty(&changes_list).unwrap())
    }
}