use format_parser::{ItemParser, TextExtractor, StringExtractor};
use format_structure::{Item, Indent, Header};

pub trait ChangeOperations
{
    ///Замена слов на другие слова
    fn replace_words(&mut self, from: &str, to: &str) -> Option<&str>;
    fn add_words(&mut self, from: &Option<String>, to: &str) -> Option<&str>;
    ///если mark=true то значит не удалять а только промаркировать что они удалены, чтобы можно было их увидеть на фронте(html)
    fn exclude_words(&mut self, words: &str, mark: bool) -> Option<&str>;

}




impl ChangeOperations for Indent
{
    fn replace_words(&mut self, from: &str, to: &str) -> Option<&str>
    {
        let source = from.replace(" ", " ");
        let target = self.body.replace(" ", " ");
        //проверяем есть или нет вхождение, потому что по replace мы этого не поймем
        let finded = target.find(&source)?;
        //let len = source.len();
        //self.body.replace_range(finded..len - 1, &to);
        //TODO потом на досуге исправить, ненадо менять таргет текст!
        let new_sting = target.replace(&source, &select_changed_words(to));
        self.replace_body(new_sting);
        Some(self.body.as_str())      
    }

    fn add_words(&mut self, from: &Option<String>, to: &str) -> Option<&str>
    {
        let mut need_space: bool = true;
        let first_char = &to.chars().next()?;
        let to = select_changed_words(to);
        if first_char.is_ascii_punctuation()
        {
            need_space = false;
        }
        if let Some(from) = from
        {
            let source = from.replace(" ", " ");
            let target = self.body.replace(" ", " ");
            let finded = target.find(&source)?;
            let splice = target.split_at(finded + source.len());
            let new_string = match need_space 
            {
                true=> [splice.0, " ", &to, splice.1].concat(),
                false=> [splice.0, &to, splice.1].concat()    
            };
            self.body = new_string;
            return Some(self.body.as_str());
        }
        else 
        {
            let last = self.body.chars().last()?;
            if last.is_ascii_punctuation()
            {
                self.body.truncate(self.body.len() -1);
            }
            if need_space
            {
                self.body.push_str(" ");
            }
            self.body.push_str(&to);
            self.body.push(last);
            return Some(self.body.as_str());
        }
    }
    ///если mark=true то значит не удалять а только промаркировать что они удалены, чтобы можно было их увидеть на фронте(html)
    fn exclude_words(&mut self, ex: &str, mark: bool) -> Option<&str>
    {
        let source = ex.replace(" ", " ");
        let target = self.body.replace(" ", " ");
        let finded = target.find(&source)?;
        let first_part = &target[..finded];
        let last_part = &target[finded + source.len()..];
        let new_string = match mark
        {
            true =>
            {
                let middle = &target[finded..finded + source.len()];
                let middle = select_excluded_words(&middle);
                [first_part, &middle, last_part].concat().replace("  ", " ")
            },
            false => [first_part, last_part].concat().replace("  ", " ")
        };
        //let new_sting = [first_part, last_part].concat().replace("  ", " ");
        self.replace_body(new_string);
        Some(self.body.as_str())      
    }
}

impl ChangeOperations for Item
{
    fn replace_words(&mut self, from: &str, to: &str) -> Option<&str>
    {
        let indents = self.indents.as_mut()?;
        let first = indents.first_mut()?;
        first.replace_words(from, to)   
    }
    fn add_words(&mut self, from: &Option<String>, to: &str) -> Option<&str>
    {
        let indents = self.indents.as_mut()?;
        let first = indents.first_mut()?;
        first.add_words(from, to)   
    }
    fn exclude_words(&mut self, ex: &str, mark: bool) -> Option<&str>
    {
        let indents = self.indents.as_mut()?;
        let first = indents.first_mut()?;
        first.exclude_words(ex, mark)   
    }
}

//TODO проблема возможна с нименованием или первым параграфом!
//если есть только один параграф то это к нему, если есть несколько параграфов или сабитемы то это уже запрос к внутренней структуре
impl ChangeOperations for Header
{
    ///тут надо проследить как это правильно делать
    /// по идее тут не должно быть ничего кроме одного абзаца,
    fn replace_words(&mut self, from: &str, to: &str) -> Option<&str>
    {
        //let source = from.replace(" ", " ");
        //let target = self.name.replace(" ", " ");
        //проверяем есть или нет вхождение, потому что по replace мы этого не поймем
        //let finded = target.find(&source)?;
        //let new_sting = target.replace(&source, &select_changed_words(to));
        //self.name = new_sting;
        let mut new_string : Option<String> = None;
        {
            let indents = self.indents.as_mut()?;
            let first = indents.first_mut()?;
            new_string = Some(first.replace_words(from, to)?.to_owned());
        }
        let mut new_string = new_string.unwrap();
        if new_string.starts_with("˹")
        {
            new_string = new_string[2..].to_owned();
        }
        let text_extractor = TextExtractor::extract(&new_string, false);
        let mut item_parser = ItemParser::new();
        //не парсит изменения "˹1. Сорта и гибриды˺
        if let Ok(_) = item_parser.parse(&text_extractor)
        {
            if let Some(item) = item_parser.get_items().as_mut().and_then(|a| a.first_mut())
            {
                if let Some(indents) = item.indents.as_mut()
                {
                    //не вставилось! проверить!
                   if let Some(indent) = indents.first_mut().and_then(|a| Some(a))
                   {
                        indent.body.insert_str(0, "˹");
                   }
                }
                let _remove = self.indents.as_mut()?.remove(0);
                self.items = Some(item_parser.get_items().unwrap());  
            }
        }
        Some("")
    }

    fn add_words(&mut self, from: &Option<String>, to: &str) -> Option<&str>
    {
        if let Some(first_indent) = self.indents.as_mut().and_then(|a|a.first_mut())
        {
            return first_indent.add_words(from, to);
            // if let Some(from) = from
            // {
            //     let source = from.replace(" ", " ");
            //     let target = first_indent.body.replace(" ", " ");
            //     let finded = target.find(&source)?;
            //     let splice = target.split_at(finded + source.len());
            //     let new_string = [splice.0, &to, splice.1].concat();
            //     first_indent.replace_body(new_string);
            //     return Some(&first_indent.body.as_str());
            // }
            // else 
            // {
            //     let last = first_indent.body.chars().last()?;
            //     self.name.truncate(self.name.len() -1);
            //     self.name.push_str(&to);
            //     self.name.push(last);
            //     return Some(&first_indent.body.as_str());
            // } 
        }
        None
    }
    fn exclude_words(&mut self, ex: &str, mark: bool) -> Option<&str>
    {
        //let source = ex.replace(" ", " ");
        //let target = self.name.replace(" ", " ");
        //let finded = target.find(&source)?;
        //let first_part = utf8_slice(&target, 0, finded)?;
        //let last_part = utf8_slice_full(&target, source.chars().count() -1)?;
        //let middle_part =  utf8_slice(&target, finded, finded + source.chars().count())?;
        //let new_sting = [first_part, last_part].concat();
        //self.name = new_sting;
        let indents = self.indents.as_mut()?;
        let first = indents.first_mut()?;
        first.exclude_words(ex, mark)   
        //Some(self.name.as_str())      
    }
}

fn search_words(source: &str, target: &str) -> Option<String>
{
    let source = source.replace(" ", " ");
    let target = target.replace(" ", " ");
    let finded = target.find(&source)?;
    Some(target)
}

fn select_changed_words(s: &str) -> String
{
    ["˹", s, "˺"].concat()
}
fn select_excluded_words(s: &str) -> String
{
    ["~", s, "~"].concat()
}

pub fn utf8_slice(s: &str, start: usize, end: usize) -> Option<&str> 
{
    assert!(end >= start);
    let start_index = s.char_indices().nth(start)?;
    let end_index = s.char_indices().nth(end)?;
    Some(&s[start_index.0..end_index.0])
}
pub fn utf8_slice_full(s: &str, start: usize) -> Option<&str> 
{
    let start_index = s.char_indices().nth(start)?;
    Some(&s[start_index.0..])
}