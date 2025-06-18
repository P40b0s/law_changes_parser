use crate::{change_action::ChangeAction, parsers::changes_parser::{Action, Change, ChangeType}, ChangesGraph};

pub trait AsText
{
    fn as_text(&self) -> String;
}
pub struct TextOutput{}
impl TextOutput
{
    pub fn gen_text(changes: &ChangesGraph) -> String
    {
        let mut dia = String::from("");
        dia.push_str("\n---\n");
        dia.push_str(&["Количество изменений: ", &changes.total_changes.to_string()].concat());
        dia.push_str("\n---\n");
        for node in &changes.nodes
        {
            if let Some(ch) = node.change.as_ref()
            {
                let path = changes.get_parent_nodes(node);
                if !path.is_empty()
                {
                    let fullpath: Vec<String> = path.iter().map(|m| m.change_path.as_text()).collect();
                    let fullpath = fullpath.join("->");
                    let fullpath = [&fullpath, "->", &node.change_path.as_text()].concat();
                    dia.push_str(&fullpath);
                    dia.push_str(" ");
                }
                
                
                let change_md = Self::change_text(ch, &node.id);
                dia.push_str(&change_md);
            }
            // else 
            // {   
            //     dia.push_str("нода есть но не обработана!");
            // }
        }
        dia
    }
    ///диаграма для изменения
    fn change_text<T: ToString>(change: &Change, parent_id: & T) -> String
    {
        let mut md = String::new();
        match change.action
        {
            Action::Apply =>
            {
                let action_after_target_path = if let Some(ap) = change.action_after_path.as_ref()
                {
                    ["После '", &ap.as_text(), "' "].concat()
                }
                else
                {
                    String::new()
                };
                let action_text = [action_after_target_path, "дополнить: ".to_owned()].concat();
                let mut changes_text = String::new();
                if let Some(txt) = change.text_changes.as_ref()
                {
                    match change.change_type
                    {
                        ChangeType::Text =>
                        {
                            for t in txt
                            {
                                changes_text.push_str(t);
                                changes_text.push_str("\n");
                            }
                        }
                        _ => ()
                    }
                }
                md.push_str(&[&action_text, "\n", &changes_text].concat());
            }
            Action::Replace => 
            {
                let action_after_target_path = if let Some(ap) = change.action_after_path.as_ref()
                {
                    ["После '", &ap.as_text(), "' "].concat()
                }
                else
                {
                    String::new()
                };
                let action_text = [action_after_target_path, "в редакции:".to_owned()].concat();
                let mut changes_text = String::new();
                if let Some(txt) = change.text_changes.as_ref()
                {
                    match change.change_type
                    {
                        ChangeType::Text =>
                        {
                            for t in txt
                            {
                                changes_text.push_str(t);
                                changes_text.push_str("\n");
                            }
                        }
                        _ => ()
                    }
                }
                md.push_str(&[&action_text, "\n", &changes_text].concat());
            }
            Action::Words =>
            {
                if let Some(change_actions) = change.changes.as_ref()
                {
                    for ca in change_actions
                    {
                        match ca
                        {
                        
                            ChangeAction::AddWords { after, words } =>
                            {
                                let relation = if let Some(af) = after
                                {
                                    ["после '", &Self::escape_quotes(af), "' дополнить:"].concat()
                                }
                                else
                                {
                                    String::from("дополнить:")
                                };
                                let res = [&relation, "\n", &words, "\n"].concat();
                                md.push_str(&res);
                            },
                            ChangeAction::ReplaceSentence { number, text } =>
                            {
                                let relation = ["предложение №", &number.to_string(), " изложить в редакции:"].concat();
                                let res = [&relation, "\n", text, "\n"].concat();
                                md.push_str(&res);
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
        md
    }

    fn escape_quotes(input: &str) -> String 
    {
        input.replace('"', r#"\""#)
    }

}

#[cfg(test)]
mod tests
{
    use crate::{outputs::{MermaidDiagram, TextOutput}, parsers::changes_parser::Changes, ChangesGraph};

    
    #[test]
    fn test_text()
    {
        logger::StructLogger::new_default();
        let test_data = include_str!("..\\..\\test_data\\test_1.txt");
        let changes_list = Changes::get_changes(test_data);
        let graph: ChangesGraph = changes_list.into();
        let text = TextOutput::gen_text(&graph);
        std::fs::write("test_txt.txt", text);
        //logger::debug!("{}", serde_json::to_string_pretty(&changes_list).unwrap())
    }
}