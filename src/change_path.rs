use std::cmp::Ordering;

use nom::IResult;
use serde::{Deserialize, Serialize};

use crate::{error::ParserError, objects::{header_type::HeaderType, item_type::ItemType, number::Number}, parsers::space1};
#[derive(Debug, Serialize, Deserialize, PartialOrd, Eq)]
pub enum ChangePath
{
    ///индекс заголовока в который нужно внести изменение
    Header
    {
        ///номер заголовка в документе
        number: Number,
        ///тип загловка
        header_type: HeaderType
    },
    Item
    {
        ///номер заголовка в документе
        number: Number,
        ///тип нумерации
        item_type: ItemType
    },
    ///номер параграфа (отсчет от 1) параграфа для внесения изменения
    Indent(u32)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Changes<O>
{
    ChangeWords
    {
        from: String,
        to: String
    },
    DeleteWords(String),
    ///Замена структурной единицы, например статьи целиком, нужно брать объект в том формате в котором будем его парсить
    ChangeObject(O)
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TargetPath(Vec<ChangePath>);
impl TargetPath
{
    pub fn new() -> Self
    {
        Self(Vec::new())
    }
    pub fn sort(&mut self)
    {
        self.0.sort();
    }
    ///у нас уже есть отдельно типы и номер
    pub fn add_header<'a, 'b>(&'a mut self, header_type: &'b str, number: &'b str) -> Result<(), ParserError>
    {
        let h_type: HeaderType = header_type.parse()?;
        let h_number: Number = number.parse()?;
        self.0.push(ChangePath::Header 
        {
            number: h_number,
            header_type: h_type
        });
        Ok(())
    }
    pub fn add_item(&mut self, number: &str, item_type: &str) -> Result<(), ParserError>
    {
        let i_number: Number = number.parse()?;
        let i_type: ItemType = item_type.parse()?;
        self.0.push(ChangePath::Item
        {
            number: i_number,
            item_type: i_type
        });
        Ok(())
    }
    pub fn add_indent(&mut self, indent_number: u32)
    {
        self.0.push(ChangePath::Indent(indent_number));
    }
    ///concat Vec<TargetPath> to one object and sorting items
    pub fn flatten(vec: Vec<Self>) -> Self
    {
        let mut slf = Self::new();
        for v in vec
        {
            slf.0.extend(v.0);
        }
        slf.sort();
        slf
    }
    pub fn get_numbers(&self) -> Vec<Number>
    {
        self.0.iter().filter_map(|m|
        {
            match m 
            {
                ChangePath::Header { number, header_type: _ }    => Some(number.clone()),
                ChangePath::Item { number, item_type: _ } => Some(number.clone()),
                _ => None
            }
        }).collect()
    }
}



impl Ord for ChangePath
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering 
    {
        match self
        {
            ChangePath::Header { number, header_type } =>
            {
                match other
                {
                    ChangePath::Header { number: other_number, header_type: other_header_type } =>
                    {
                        let ht = header_type.cmp(other_header_type);
                        if ht != Ordering::Equal
                        {
                            number.cmp(other_number)
                        }
                        else 
                        {
                            ht
                        }
                    }
                    _ => Ordering::Greater
                }
            },
            ChangePath::Item { number, item_type } =>
            {
                match other
                {
                    ChangePath::Header { number: _, header_type: _ } =>
                    {
                        Ordering::Less
                    }
                    ChangePath::Item { number: other_number, item_type: other_item_type } =>
                    {
                        let hierarchy_ord = item_type.cmp(other_item_type);
                        if hierarchy_ord == Ordering::Equal
                        {
                            number.cmp(other_number)
                        }
                        else 
                        {
                            hierarchy_ord    
                        }
                        
                    }
                    _ => Ordering::Greater
                }
            }
            ChangePath::Indent(i) =>
            {
                match other
                {
                    ChangePath::Indent(other_i) =>
                    {
                        i.cmp(other_i)
                    }
                    _ => Ordering::Less
                }
            }
        }
    }
}

impl PartialEq for ChangePath
{
    fn eq(&self, other: &Self) -> bool 
    {
        match self
        {
            ChangePath::Header { number, header_type } =>
            {
                match other
                {
                    ChangePath::Header { number: other_number, header_type: other_header_type } =>
                    {
                        number == other_number && header_type == other_header_type
                    }
                    _ => false
                }
            },
            ChangePath::Item { number , item_type} =>
            {
                match other
                {
                    ChangePath::Item { number: other_number, item_type: other_item_type } =>
                    {
                        number == other_number && item_type == other_item_type
                    }
                    _ => false
                }
            }
            ChangePath::Indent(i) =>
            {
                match other
                {
                    ChangePath::Indent(other_i) =>
                    {
                        i == other_i
                    }
                    _ => false
                }
            }
        }
    }
}


#[cfg(test)]
mod tests
{
    use crate::change_path::ChangePath;

    #[test]
    fn test_path_1()
    {
        logger::StructLogger::new_default();
        let (type1, number1) = ("раздел", "1");
        let (type2, number2) = ("глава", "3");
        let (type3, number3) = ("статья", "22^2");
        let parag1 = "1_1)";
        let hie1 = "часть";
        let parag2 = "1_2)";
        let hie2 = "пункт";
        let subparag3 = "а^1.";
        let hie3 = "подпункт";
        let mut tp = super::TargetPath::new();
        let h1 = tp.add_header(type3, number3);
        let h2 = tp.add_header(type1, number1);
        let h3 = tp.add_header(type2, number2);
        let p1 = tp.add_item(subparag3, hie3);
                logger::debug!("{:?}", p1);
        let p2 = tp.add_item(parag2, hie2);
                logger::debug!("{:?}", p2);
        let p3 = tp.add_item(parag1, hie1);
                logger::debug!("{:?}", p3);
        tp.sort();
        if let ChangePath::Header { number, header_type } = &tp.0[2]
        {
            logger::debug!("{:?}", tp);
            assert_eq!(number.number, "22");
        }
    }
}