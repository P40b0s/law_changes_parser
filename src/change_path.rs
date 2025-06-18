use std::{cmp::Ordering, hash::{DefaultHasher, Hash, Hasher}};

use nom::IResult;
use serde::{Deserialize, Serialize};

use crate::{error::ParserError, objects::{header_type::HeaderType, item_type::ItemType, number::Number}, outputs::{AsMarkdown, AsText}, parsers::space1};
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash)]
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

impl ChangePath
{
    ///получаем уровень пути, 
    pub fn get_lvl(&self) -> u8
    {
        match self
        {
            ChangePath::Header { number: _, header_type: ht } =>
            {
                match ht
                {
                    HeaderType::Chapter => 0,
                    HeaderType::Section => 1,
                    HeaderType::Article => 2
                }
            }
            ChangePath::Item { number: _, item_type: it } =>
            {
                match it
                {
                    ItemType::Part => 3,
                    ItemType::Item => 4,
                    ItemType::Subitem => 5
                }
            }
            ChangePath::Indent(_) => 6
        }
    }
    // pub fn as_markdown(&self) -> String
    // {
    //     let mut output = String::new();
    //     match self
    //     {
    //         ChangePath::Header { number: n, header_type: ht } =>
    //         {
    //             match ht
    //             {
    //                 HeaderType::Chapter => 
    //                 {
    //                     output.push_str("раздел ");
    //                     output.push_str(&n.get_number_as_markdown());
    //                 }
    //                 HeaderType::Section => 
    //                 {
    //                     output.push_str("глава ");
    //                     output.push_str(&n.get_number_as_markdown());
    //                 }
    //                 HeaderType::Article => 
    //                 {
    //                     output.push_str("статья ");
    //                     output.push_str(&n.get_number_as_markdown());
    //                 }
    //             }
    //         }
    //         ChangePath::Item { number: n, item_type: it } =>
    //         {
    //             match it
    //             {
    //                 ItemType::Part => 
    //                 {
    //                     output.push_str("часть ");
    //                     output.push_str(&n.get_number_as_markdown());
    //                 }
    //                 ItemType::Item => 
    //                 {
    //                     output.push_str("пункт ");
    //                     output.push_str(&n.get_number_as_markdown());
    //                 }
    //                 ItemType::Subitem => 
    //                 {
    //                     output.push_str("подпункт ");
    //                     output.push_str(&n.get_number_as_markdown());
    //                 }
    //             }
    //         }
    //         ChangePath::Indent(n) => 
    //         {
    //             output.push_str("абзац ");
    //             output.push_str(&n.to_string());
    //         }
    //     }
    //     output
    // }
    pub fn get_number(&self) -> Number
    {
        match self
        {
            ChangePath::Header { number: n, header_type: _ } =>
            {
                n.clone()
            }
            ChangePath::Item { number: n, item_type: _ } =>
            {
                n.clone()
            }
            ChangePath::Indent(n) => 
            {
                Number 
                {
                    number: n.to_string(),
                    va_number: None,
                    postfix: None
                }
            } 
        }
    }
}
// impl AsMarkdown for ChangePath
// {
//     fn as_markdown(&self) -> String
//     {
//         match self
//         {
//             ChangePath::Header { number: n, header_type: _ } =>
//             {
//                 n.as_markdown()
//             }
//             ChangePath::Item { number: n, item_type: _ } =>
//             {
//                 n.as_markdown()
//             }
//             ChangePath::Indent(n) => 
//             {
//                 Number 
//                 {
//                     number: n.to_string(),
//                     va_number: None,
//                     postfix: None
//                 }.as_markdown()
//             } 
//         }
//     }
// }

 
impl AsMarkdown for ChangePath
{
    fn as_markdown(&self) -> String
    {
        let mut output = String::new();
        match self
        {
            ChangePath::Header { number: n, header_type: ht } =>
            {
                match ht
                {
                    HeaderType::Chapter => 
                    {
                        output.push_str("раздел ");
                        output.push_str(&n.as_markdown());
                    }
                    HeaderType::Section => 
                    {
                        output.push_str("глава ");
                        output.push_str(&n.as_markdown());
                    }
                    HeaderType::Article => 
                    {
                        output.push_str("статья ");
                        output.push_str(&n.as_markdown());
                    }
                }
            }
            ChangePath::Item { number: n, item_type: it } =>
            {
                match it
                {
                    ItemType::Part => 
                    {
                        output.push_str("часть ");
                        output.push_str(&n.as_markdown());
                    }
                    ItemType::Item => 
                    {
                        output.push_str("пункт ");
                        output.push_str(&n.as_markdown());
                    }
                    ItemType::Subitem => 
                    {
                        output.push_str("подпункт ");
                        output.push_str(&n.as_markdown());
                    }
                }
            }
            ChangePath::Indent(n) => 
            {
                output.push_str("абзац ");
                output.push_str(&n.to_string());
            }
        }
        output
    }
}

impl AsText for ChangePath
{
    fn as_text(&self) -> String 
    {
        self.as_markdown()
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub enum Changes<O>
// {
//     ChangeWords
//     {
//         from: String,
//         to: String
//     },
//     DeleteWords(String),
//     ///Замена структурной единицы, например статьи целиком, нужно брать объект в том формате в котором будем его парсить
//     ChangeObject(O)
// }
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    ///добавляет глобальные пути в начало вектора если в этом векторе уже есть путь с таким же уровнем удаляем его из глобальных путей
    pub fn insert_paths(&mut self, paths: &mut Vec<ChangePath>)
    {
        paths.retain(|r| 
        {
            !self.0.iter().any(|a| a.get_lvl() == r.get_lvl())
        });
        if !paths.is_empty()
        {
            self.0 = [paths.clone(), self.0.clone()].concat();
        }
    }
    ///возвращает пути с их текущим хешем
    pub fn get_paths(&self) -> &Vec<ChangePath>
    {
      &self.0
    }
    pub fn get_paths_with_id(&self) -> Vec<(u64, ChangePath)>
    {
        let mut hasher = DefaultHasher::new();
        self.0.iter().map(|cp|
        {
            cp.hash(&mut hasher);
            let id = hasher.finish();
            (id, cp.clone())
        }).collect()
    }
    pub fn get_path_by_level(&self, level: usize) -> Option<(u64, ChangePath)>
    {
        let mut paths = self.get_paths_with_id();
        if paths.get(level).is_some()
        {
            Some(paths.swap_remove(level))
        }
        else
        {
            None
        }
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
    ///Отдает новый вектор с клонироваными номерами
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

impl AsMarkdown for TargetPath
{
    fn as_markdown(&self) -> String
    {
        let mut paths = String::new();
        for p in &self.0
        {
            paths.push_str(&p.as_markdown());
            paths.push(' ');
        }
        paths
    }
}
impl AsText for TargetPath
{
    fn as_text(&self) -> String 
    {
        let mut paths = String::new();
        for p in &self.0
        {
            paths.push_str(&p.as_text());
            paths.push(' ');
        }
        paths
    }
}



///определяем Header как наименьший а Indent как наибольший для правильной сортировки
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
                    _ => Ordering::Less
                }
            },
            ChangePath::Item { number, item_type } =>
            {
                match other
                {
                    ChangePath::Header { number: _, header_type: _ } =>
                    {
                        Ordering::Greater
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
                    _ => Ordering::Less
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
                    _ => Ordering::Greater
                }
            }
        }
    }
}
impl PartialOrd for ChangePath
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> 
    {
        Some(self.cmp(other))
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
    use crate::{change_path::ChangePath, objects::{header_type::HeaderType, number::Number}};

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

    #[test]
    fn test_ord()
    { 
        let indent = ChangePath::Indent(3);
        let header = ChangePath::Header { number: Number::parse("20").unwrap().1, header_type: "Статья".parse().unwrap() };
        let item = ChangePath::Item { number: Number::parse("2)").unwrap().1, item_type: "пункт".parse().unwrap()  };
        let mut items = vec![item.clone(), indent.clone(), header.clone()];
        items.sort();
        assert_eq!(items[0], header);
        assert_eq!(items[1], item);
        assert_eq!(items[2], indent);
    }
}