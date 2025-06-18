use std::hash::{DefaultHasher, Hash, Hasher};
use crate::{change_action::ChangeAction, outputs::AsMarkdown, parsers::changes_parser::{Action, Change, ChangeType}, ChangesGraph};

pub struct MermaidDiagram
{
}
impl MermaidDiagram
{
    pub fn gen_diagram(changes: &ChangesGraph) -> String
    {
        let mut dia = String::from("```mermaid");
        dia.push_str("\n---");
        dia.push_str(&["\ntitle: ", "\"", "Количество изменений: ", &changes.total_changes.to_string(), "\""].concat());
        dia.push_str("\n---");
        dia.push_str("\n%%{init: { \"themeVariables\": {\"fontSize\": \"20px\"} } }%%");
        dia.push_str("\nflowchart TD\n");
        for node in &changes.nodes
        {
            if let Some(ch) = node.change.as_ref()
            {
                let node_md = format!("{}:::node@{{ shape: rect, label: \"{}\"}}\n", node.id, node.change_path.as_markdown());
                dia.push_str(&node_md);
                let change_md = Self::change_diagramm(ch, &node.id);
                dia.push_str(&change_md);
            }
            else 
            {   
                let node_md = format!("{}:::node@{{ shape: rect, label: \"{}\"}}\n", node.id, node.change_path.as_markdown());
                dia.push_str(&node_md);
            }
        }
        for e in &changes.edges
        {
            let d = format!("{}-->{}\n", e.from_id, e.to_id);
            dia.push_str(&d);
        }
        dia.push_str("\nclassDef replace fill:#061341,stroke:#333,stroke-width:4px,color:white,font-size:18px");
        dia.push_str("\nclassDef apply fill:#022400,stroke:#333,stroke-width:4px,color:white,font-size:18px");
        dia.push_str("\nclassDef words fill:#230024,stroke:#333,stroke-width:4px,color:white,font-size:18px");
        dia.push_str("\nclassDef node font-size:28px");
        dia.push_str("\n```");
        dia
    }
    ///диаграма для изменения
    fn change_diagramm<T: ToString>(change: &Change, parent_id: & T) -> String
    {
        let mut md = String::new();
        match change.action
        {
            Action::Apply =>
            {
                let action_after_target_path = if let Some(ap) = change.action_after_path.as_ref()
                {
                    ["После '", &ap.as_markdown(), "'"].concat()
                }
                else
                {
                    String::new()
                };
                let action_text = [action_after_target_path, " дополнить: ".to_owned()].concat();
                let mut changes_text = String::new();
                let id = if let Some(txt) = change.text_changes.as_ref()
                {
                    let mut hasher = DefaultHasher::new();
                    txt.hash(&mut hasher);
                    let hash = hasher.finish();
                    match change.change_type
                    {
                        ChangeType::Text =>
                        {
                            for t in txt
                            {
                                let escaped = Self::escape_quotes(t);
                                changes_text.push_str(&escaped);
                                changes_text.push_str("  ");
                            }
                        }
                        _ => ()
                    }
                    hash
                }
                else
                {
                    0000
                };
                md.push_str(&format!("{}-->|{}|{}:::apply@{{ shape: {}, label: \"{}\"}}\n", parent_id.to_string(), action_text, id, "rounded", changes_text));
            }
            Action::Replace => 
            {
                let action_after_target_path = if let Some(ap) = change.action_after_path.as_ref()
                {
                    ["После '", &ap.as_markdown(), "'"].concat()
                }
                else
                {
                    String::new()
                };
                let action_text = [action_after_target_path, " в редакции: ".to_owned()].concat();
                let mut changes_text = String::new();
                let id = if let Some(txt) = change.text_changes.as_ref()
                {
                    let mut hasher = DefaultHasher::new();
                    txt.hash(&mut hasher);
                    let hash = hasher.finish();
                    match change.change_type
                    {
                        ChangeType::Text =>
                        {
                            for t in txt
                            {
                                let escaped = Self::escape_quotes(t);
                                changes_text.push_str(&escaped);
                                changes_text.push_str("  ");
                            }
                        }
                        _ => ()
                    }
                    hash
                }
                else
                {
                    0000
                };
                md.push_str(&format!("{}-->|{}|{}:::replace@{{ shape: {}, label: \"{}\"}}\n", parent_id.to_string(), action_text, id, "rounded", changes_text));
            }
            Action::Words =>
            {
                if let Some(change_actions) = change.changes.as_ref()
                {
                    for ca in change_actions
                    {
                        let mut hasher = DefaultHasher::new();
                        ca.hash(&mut hasher);
                        let ca_id = hasher.finish();
                        match ca
                        {
                        
                            ChangeAction::AddWords { after, words } =>
                            {
                                let relation = if let Some(af) = after
                                {
                                    ["после '", &Self::escape_quotes(af), "' дополнить"].concat()
                                }
                                else
                                {
                                    String::from("дополнить")
                                };
                                md.push_str(&format!("{}-->|{}|{}:::words@{{ shape: {}, label: \"{}\"}}\n", parent_id.to_string(), relation, ca_id, "rounded",  Self::escape_quotes(words)));
                                // if let Some(af) = after
                                // {
                                    
                                //     md.push_str(&format!("{}-->|{}|{}@{{ shape: {}, label: \"после '{}' дополнить '{}'\"}}\n", parent_id.to_string(), ca_id, "rounded", escape_quotes(af), escape_quotes(words)));
                                // }
                                // else 
                                // {
                                //     md.push_str(&format!("{} --> {}@{{ shape: {}, label: \"дополнить '{}'\"}}\n", parent_id.to_string(), ca_id, "rounded", escape_quotes(words)));
                                // }
                            },
                            ChangeAction::ReplaceSentence { number, text } =>
                            {
                                let relation = ["предложение **№", &number.to_string(), "** изложить в редакции"].concat();
                                md.push_str(&format!("{}-->|{}|{}:::words@{{ shape: {}, label: \"{}\"}}\n", parent_id.to_string(), relation, ca_id, "rounded",  Self::escape_quotes(text)));
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
    use crate::{outputs::MermaidDiagram, parsers::changes_parser::Changes, ChangesGraph};

    
    #[test]
    fn test_diagramm()
    {
        logger::StructLogger::new_default();
        let test_data = include_str!("..\\..\\test_data\\test_1.txt");
        let changes_list = Changes::get_changes(test_data);
        let graph: ChangesGraph = changes_list.into();
        let diagram = MermaidDiagram::gen_diagram(&graph);
        std::fs::write("test_dia.md", diagram);
        //logger::debug!("{}", serde_json::to_string_pretty(&changes_list).unwrap())
    }
   

    #[test]
    fn test_diagramm2()
    {
        logger::StructLogger::new_default();
        let test_data = include_str!("..\\..\\test_data\\test_2.txt");
        let changes_list = Changes::get_changes(test_data);
        let graph: ChangesGraph = changes_list.into();
        let diagram = MermaidDiagram::gen_diagram(&graph);
        std::fs::write("test_dia2.md", diagram);
        //logger::debug!("{}", serde_json::to_string_pretty(&changes_list).unwrap())
    }
}