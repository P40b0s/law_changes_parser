use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{change_action::ChangeAction, parsers::changes_parser::{Action, Change, ChangeType}};

pub trait MermaidDiagramm
{
    fn get_diagramm(&self) -> String;
}
pub trait AsMarkdown
{
    fn as_markdown(&self) -> String;
}

pub struct MermaidDiagrammConstructor<'a, M: AsMarkdown, T: ToString>
{
    shape: String,
    pub id: &'a T,
    pub parent_id: Option<&'a T>,
    pub label: &'a M,
    pub change: Option<&'a Change>
}

impl<'a, M: AsMarkdown, T: ToString> MermaidDiagrammConstructor<'a, M, T>
{
    pub fn new(id: &'a T, label: &'a M) -> Self
    {
        Self
        {
            shape: "rect".to_owned(),
            id,
            parent_id: None,
            label,
            change: None
        }
    }

    pub fn new_with_parent(id: &'a T, parent_id: &'a T, label: &'a M) -> Self
    {
        Self
        {
            shape: "rect".to_owned(),
            id,
            parent_id: Some(parent_id),
            label,
            change: None
        }
    }
    pub fn new_with_parent_and_change(id: &'a T, parent_id: &'a T, label: &'a M, change: &'a Change) -> Self
    {
        Self
        {
            shape: "rect".to_owned(),
            id,
            parent_id: Some(parent_id),
            label,
            change: Some(change)
        }
    }
    // fn chanage_to_md(change: &Change) -> String
    // {
    //     let mut md = String::new();
    //     match change.action
    //     {
    //         Action::Apply =>
    //         {
    //             if let Some(ap) = change.action_after_path.as_ref()
    //             {
    //                 let md_path = ap.as_markdown();
    //                 md.push_str("После: ");
    //                 md.push_str(&md_path);
    //             }
    //             if let Some(txt) = change.text_changes.as_ref()
    //             {
    //                 for t in txt
    //                 {
    //                     let escaped = escape_quotes(t);
    //                     md.push_str(&escaped);
    //                     md.push_str("  ");
    //                 }
    //             }
    //         }
    //         Action::Replace => 
    //         {
    //             if let Some(ap) = change.action_after_path.as_ref()
    //             {
    //                 let md_path = ap.as_markdown();
    //                 md.push_str("После: ");
    //                 md.push_str(&md_path);
    //                 md.push_str("  ");
    //             }
    //             if let Some(txt) = change.text_changes.as_ref()
    //             {
    //                 for t in txt
    //                 {
    //                     let escaped = escape_quotes(t);
    //                     md.push_str(&escaped);
    //                     md.push_str("  ");
    //                 }
    //             }
    //         }
    //         Action::Words =>
    //         {
    //             if let Some(change_actions) = change.changes.as_ref()
    //             {
    //                 md.push_str("Изменения слов:  ");
    //                 for ca in change_actions
    //                 {
    //                     match ca
    //                     {
    //                         ChangeAction::AddWords { after, words } =>
    //                         {
    //                             if let Some(af) = after
    //                             {
    //                                 md.push_str("после '");
    //                                 md.push_str(af);
    //                                 md.push_str("' дополнить '");
    //                                 md.push_str(words);
    //                                 md.push_str("'");
    //                             }
    //                         },
    //                         _ => md.push_str("Еще не реализовано"),
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     md

    // }

    pub fn gen_line(&self) -> String
    {
        if let Some(p_id) = self.parent_id.as_ref()
        {
            let struct_line = format!("{}:::path-->{}:::path@{{ shape: {}, label: \"{}\"}}\n", p_id.to_string(), self.id.to_string(), &self.shape, &self.label.as_markdown());
            if let Some(ch) = self.change
            {
                let concated_line = [struct_line, "\n".to_owned(), change_diagramm(ch, self.id)].concat();
                concated_line
            }
            else 
            {
                struct_line
            }
        }
        else 
        {
            format!("{}:::path@{{ shape: {}, label: \"{}\"}}\n", self.id.to_string(), &self.shape, &self.label.as_markdown())
        }
       
    }

}

fn change_diagramm<'a, T: ToString>(change: &Change, parent_id: &'a T) -> String
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
                            let escaped = escape_quotes(t);
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
                            let escaped = escape_quotes(t);
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
                                ["после '", &escape_quotes(af), "' дополнить"].concat()
                            }
                            else
                            {
                                String::from("дополнить")
                            };
                            md.push_str(&format!("{}-->|{}|{}:::words@{{ shape: {}, label: \"{}\"}}\n", parent_id.to_string(), relation, ca_id, "rounded",  escape_quotes(words)));
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
                            md.push_str(&format!("{}-->|{}|{}:::words@{{ shape: {}, label: \"{}\"}}\n", parent_id.to_string(), relation, ca_id, "rounded",  escape_quotes(text)));
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
