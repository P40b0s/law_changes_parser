use std::{cmp::Ordering, str::FromStr};
use nom::{branch::alt, bytes::tag_no_case, Parser};
use serde::{Deserialize, Serialize};

use crate::error::ParserError;



#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemType
{
    ///иногда частями назвают например 1) это часть первая в ФЗ
    Part,
    ///пункт где то являеется главной единицей нумерации где то идет после части
    Item,
    ///подпункт
    Subitem
}

impl FromStr for ItemType
{
    type Err = ParserError;
    fn from_str(s: &str) -> Result<Self, Self::Err> 
    {
        match s.to_lowercase()
        {
            h if h.starts_with("част") => Ok(ItemType::Part),
            h if h.starts_with("пункт") => Ok(ItemType::Item),
            h if h.starts_with("подпункт") => Ok(ItemType::Subitem),
            _ => Err(ParserError::OperationError(["строка `", s, "` не является валидным списочным элементом"].concat()))
        }  
    }
}

impl Ord for ItemType
{
    fn cmp(&self, other: &Self) -> Ordering 
    {
        match self
        {
            ItemType::Part =>
            {
                match other
                {
                    ItemType::Part => Ordering::Equal,
                    _ => Ordering::Less
                }
            }
            ItemType::Item =>
            {
                match other 
                {
                    ItemType::Part => Ordering::Greater,
                    ItemType::Item => Ordering::Equal,
                    ItemType::Subitem => Ordering::Less
                }
            }
            ItemType::Subitem =>
            {
                match other
                {
                    ItemType::Subitem => Ordering::Equal,
                    _ => Ordering::Greater
                }
            }
        }
    }
}
impl PartialOrd for ItemType
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> 
    {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests
{
    use crate::objects::item_type::ItemType;

    #[test]
    fn test_parse()
    {
        logger::StructLogger::new_default();
        let p_name = "пункт ";
        let p_num = "1^1.";
        let it: ItemType = p_name.parse().unwrap();
        assert_eq!(ItemType::Item, it);
    }
    #[test]
    fn test_ordering()
    {
        let mut items = vec![
            ItemType::Subitem,
            ItemType::Part,
            ItemType::Item
        ];
        logger::StructLogger::new_default();
        logger::debug!("before sorting: {:?}", &items);
        items.sort();
        logger::debug!("after sorting: {:?}", &items);
        assert_eq!(items[0], ItemType::Part);
        assert_eq!(items[1], ItemType::Item);
        assert_eq!(items[2], ItemType::Subitem);
    }
}