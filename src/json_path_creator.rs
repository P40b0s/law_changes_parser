use std::{iter::Sum, fmt::Display};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
pub enum FormatPath
{
    Header(String),
    Item,
    Indent
}
impl Display for FormatPath
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self 
        {
            FormatPath::Header(t) => write!(f, "{}", t),
            FormatPath::Item => f.write_str("пункт, подпункт итд."),
            FormatPath::Indent => f.write_str("абзац"),
        }
    }
}
#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct FormatPathItem
{
    item: FormatPath,
    number: Option<String>
}


impl FormatPathItem
{
    pub fn get_item_type(&self) -> &FormatPath
    {
        &self.item
    }
    pub fn get_item_number(&self) -> Option<&String>
    {
        self.number.as_ref()
    }
    pub fn get_path(&self) -> String
    {
        match &self.item 
        {
            FormatPath::Header(t) => 
            {
                if let Some(num) = self.number.as_ref()
                {
                    FormatPathItem::get_header(num, t)
                }
                else 
                {
                    [".headers", OPEN_SEARCH_TAG, "*", CLOSE_SEARCH_TAG].concat()
                }
            },
            FormatPath::Item => 
            {
                if let Some(num) = self.number.as_ref()
                {
                    FormatPathItem::get_item(num)
                }
                else 
                {
                    [".items", OPEN_SEARCH_TAG, "*", CLOSE_SEARCH_TAG].concat()
                }
            },
            FormatPath::Indent => 
            {
                if let Some(num) = self.number.as_ref()
                {
                    FormatPathItem::get_indent(num.parse().unwrap())
                }
                else 
                {
                    [".indents[", "*","]"].concat()
                }
            }
        }
    }
    pub fn get_enum(&self) -> (FormatPath, String)
    {
        let num = self.number.as_ref();
        if num.is_some()
        {
            (self.item.clone(), num.unwrap().clone())
        }
        else
        {
            (self.item.clone(), String::new())
        }
    }

    fn get_indent(number: u32) -> String
    {
        [".indents[", &(number-1).to_string(),"]"].concat()
    }
    fn get_item(number: &str) -> String
    {
        //let json_path_str = number_search(number);
        [".items", OPEN_SEARCH_TAG, &number_search(number), CLOSE_SEARCH_TAG].concat()
    }
    fn get_header(number: &str, header_type: &str) -> String
    {
        
        //let json_path_str = number_search(number);
        [".headers", OPEN_SEARCH_TAG, &header_search(number, header_type), CLOSE_SEARCH_TAG].concat()
    }
}
const OPEN_SEARCH_TAG: &str = "[?(";
const CLOSE_SEARCH_TAG: &str = ")]";
#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct JsonPathCreator
{
    ///Полный путь к конечному итему: в пункте 2 статьи 3 ....
    items: Vec<FormatPathItem>,
    hierarchy_lvl: usize,
}

impl Sum for JsonPathCreator
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self 
    {
        iter.fold(JsonPathCreator { items: vec![], hierarchy_lvl: 0 }, |a, b|
        {
            a + b
        })
    }
}
impl Display for JsonPathCreator
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.get_body_jsonpath())
    }
}

impl std::ops::Add for JsonPathCreator
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output 
    {
        let mut self_vec = self.items.clone();
        self_vec.extend(rhs.items);
        Self
        {
            hierarchy_lvl: 0,
            items: self_vec
        }
    }
}
impl std::ops::Add<FormatPathItem> for JsonPathCreator
{
    type Output = Self;
    fn add(self, rhs: FormatPathItem) -> Self::Output 
    {
        let mut self_vec = self.items.clone();
        self_vec.push(rhs);
        Self
        {
            hierarchy_lvl: 0,
            items: self_vec
        }
    }
}


impl Default for JsonPathCreator
{
    fn default() -> Self 
    {
        JsonPathCreator { items: vec![], hierarchy_lvl: 0}
    }
}

impl JsonPathCreator
{
    pub fn new_with_indent(number: Option<u32>) -> Self
    {
        let mut js = JsonPathCreator::default();
        js.add_indent(number);
        js
    }
    pub fn new_with_header(h_type: &str, number: Option<&str>) -> Self
    {
        let mut js = JsonPathCreator::default();
        js.add_header(h_type, number);
        js
    }
    pub fn new_with_item(number: Option<&str>) -> Self
    {
        let mut js = JsonPathCreator::default();
        js.add_item(number);
        js
    }
    pub fn add_header(&mut self, header_type: &str, number: Option<&str>) -> &mut Self
    {
        let mut num : Option<String> = None;
        if let Some(n) = number
        {
            num = Some(n.to_owned());
        }
        let fi = FormatPathItem {item: FormatPath::Header(reverse_header_name(header_type)), number: num};
        self.items.push(fi);
        self
    }
    pub fn add(&mut self, items: &[FormatPathItem]) -> &mut Self
    {
        for i in items
        {
            self.items.push(i.clone())
        }
        self
    }
   
    pub fn reverse(&mut self)
    {
        self.items.reverse();
    }
    pub fn add_item(&mut self, number: Option<&str>) -> &mut Self
    {
        let mut num : Option<String> = None;
        if let Some(n) = number
        {
            num = Some(n.to_owned());
            self.hierarchy_lvl = self.hierarchy_lvl + 1;
        }
        let fi = FormatPathItem {item: FormatPath::Item, number: num};
        self.items.push(fi);
        self
    }
    pub fn add_indent(&mut self, number: Option<u32>) -> &mut Self
    {
        let fi = FormatPathItem {item: FormatPath::Indent, number: number.and_then(|a| Some(a.to_string()))};
        self.items.push(fi);
        self
    }
    pub fn get_jsonpath(&self) -> String
    {
        let path = self.items.iter().map(|m| m.get_path()).collect::<String>();
        path
    }
    pub fn get_sequence(&self) -> Option<(FormatPath, Vec<String>)>
    {
        let path = self.items.iter().map(|m| m).collect::<Vec<&FormatPathItem>>();
        let first = path.first()?.item.clone();
        let mut numbers = path.iter().map(|m| m.number.as_ref().cloned().unwrap_or(String::new())).collect::<Vec<String>>();
        numbers.reverse();
        Some((first, numbers))
    }
    pub fn get_numbers(&self) -> Vec<String>
    {
        let path = self.items.iter()
        .filter(|f| f.number.is_some())
        .map(|m| m.number.as_ref().unwrap().to_owned())
        .collect::<Vec<String>>();
        path
    }
    pub fn get_path_items(&self) -> Vec<&FormatPathItem>
    {
        let path = self.items.iter().map(|m| m).collect::<Vec<&FormatPathItem>>();
        path
    }
    pub fn get_unique_path()
    {
        
    }
    pub fn get_body_jsonpath(&self) -> String
    {
        ["$.body", &self.get_jsonpath()].concat()
    }
    
    ///Возвращаем тип итема\номер итема
    pub fn last_path(&self) -> Option<&FormatPathItem>
    {
        self.items.last()
    }
    pub fn is_empty(&self) -> bool
    {
        self.items.len() == 0
    }
    pub fn get_hierarchy_item_query(&self) -> usize
    {
        self.hierarchy_lvl
    }
    pub fn get_items(&self) -> &Vec<FormatPathItem>
    {
        &self.items
    }

   
}

fn reverse_header_name(header_type: &str) -> String
{
    if header_type.starts_with("стат") || header_type.starts_with("Стат")
    {
        return "статья".to_owned();
    }
    if header_type.starts_with("глав") || header_type.starts_with("Глав")
    {
        return "глава".to_owned();
    }
    if header_type.starts_with("разд") || header_type.starts_with("Разд")
    {
        return "раздел".to_owned();
    }
    return "НЕОПРЕДЕЛЕН".to_owned();  
}

fn split_superscript_number(number: &str) -> Option<(&str, &str)>
{
    number.split_once("^")
}   
fn split_subscript_number(number: &str) -> Option<(&str, &str)>
{
    number.split_once("_")
}   
fn number_search(number: &str) -> String
{
    format!("@.number.val == '{}'", number)
}
fn header_search(number: &str, h_type: &str) -> String
{
    format!("{} && @.type == '{}'", number_search(number), h_type)
}

#[cfg(test)]
mod tests
{
    use super::JsonPathCreator;

    #[test]
    fn test_headers()
    {
        let mut jp =  JsonPathCreator::new_with_header("глава", Some("211"));
        assert_eq!(jp.get_jsonpath(), ".headers[?(@.number.val == '211' && @.type == 'глава')]");

        let mut jp =  JsonPathCreator::new_with_header("статью", Some("2^21"));
        assert_eq!(jp.get_jsonpath(), ".headers[?(@.number.val == '2^21' && @.type == 'статья')]");

        let mut jp =  JsonPathCreator::new_with_header("раздела", Some("21_б"));
        assert_eq!(jp.get_jsonpath(), ".headers[?(@.number.val == '21_б' && @.type == 'раздел')]");
    }

    #[test]
    fn test_hierarchy()
    {
        let mut path = JsonPathCreator::new_with_header("статья", Some("5"));

        path.add_item(Some("6"))
        .add_item(Some("2"))
        .add_item(Some("б"));
        let hierarchy = path.get_hierarchy_item_query();
        assert_eq!(3, hierarchy);
    }
}