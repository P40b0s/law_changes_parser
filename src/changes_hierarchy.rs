use format_parser::{TextExtractor, ItemsHierarchy};
use std::{path::Path, fs::File, io::BufReader, error::Error};
use serde::{Deserialize, Serialize};

use crate::{parsers::{TargetDocument, Deserializer}, json_path_creator::JsonPathCreator};

//TODO сделать тесты на проверку структуры
///Структура содержит иерархичный список изменений 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChangesHierarchyItem
{
    pub text: TextExtractor,
    pub changes: Vec<TextExtractor>,
    pub subitems: Vec<ChangesHierarchyItem>,
    pub indents: Vec<TextExtractor>,


}
impl ChangesHierarchyItem
{
    pub fn new(text: &TextExtractor, items: Vec<TextExtractor>) -> Self
    {
        ChangesHierarchyItem { text: text.clone(), changes: items, subitems: vec![], indents: vec![] } 
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChangesHierarchy
{
    target_document: Option<TargetDocument>,
    //путь к изменению, а что с ним надо сделать будем разбирать в парсере, но для этого надо отрезать все начало до конца скобки
    root_items: Option<ChangesHierarchyItem>,
    items: Vec<ChangesHierarchyItem>,
}
impl ChangesHierarchy
{
    ///находим элементы с флагом is_change == true и собираем все элементы после него у которых флаг is_change == false
    pub fn get_changes_hierarchy(v: &Vec<TextExtractor>)-> Result<Self, Vec<TextExtractor>>
    {
        let mut hierarchy = ItemsHierarchy::default();
        //находим определение -> Внести в статью 22 Закона Российской Федерации от 14 мая 1993 года № 4973-I "О зерне" следующие изменения:
        let target = v.first().and_then(|a| TargetDocument::check_target_annotation(a));
        let mut items: Vec<ChangesHierarchyItem> = vec![];
        let mut root_item: Option<ChangesHierarchyItem> = None;
        //если не найдено ни одного изменения то это скорее всего статья не с изменениями а с различными дополнениями, когда и что вступает в силу итд
        if v.iter().all(|a| !a.is_change) && target.is_none()
        {
            //let not_recognized = v[1..].iter().map(|m|m.clone()).collect::<Vec<TextExtractor>>();
            logger::error!("Невозможно получить иерархию изменений, изменения не найдены: {}", serde_json::to_string_pretty(v).unwrap());
            return Err(v.clone());
        }
        let mut last_item_hierarchy: Option<usize> = None;
        for (i, o) in v.iter().enumerate()
        {
            if target.is_some() && i == 0
            {
                //возможны короткие варианты изменений: 
                //Часть 4 статьи 5 Федерального закона от 29 декабря 2006 года № 264-ФЗ "О развитии сельского хозяйства" дополнить пунктом 8 следующего содержания:
                //здесь общая логика не подходит, надо взять путь и все изменения
                if target.as_ref().unwrap().root_path.is_some()
                {
                    let changes = v[i+1..].iter()
                    .take_while(|t| !t.is_change)
                    .map(|m| m.clone())
                    .collect::<Vec<TextExtractor>>();
                    root_item = Some(ChangesHierarchyItem::new(o, changes));
                    continue;
                }
                else
                {
                    continue;
                }
            }
            if o.is_change && i < v.len()
            {
                let changes = v[i+1..].iter()
                .take_while(|t| !t.is_change)
                .map(|m| m.clone())
                .collect::<Vec<TextExtractor>>();
                let item = ChangesHierarchyItem::new(o, changes);
                
                //группируем итемы по иерархии, чтобы было проще пробрасывать jsonpath
                //имеется ввиду иерархия изменений:
                //1) в статье 2:
                //а) в пункте 3: итд.
                //TODO проблемы в поле  13) наименование главы 5 изложить в следующей редакции: \n "Глава 5. Соглашение об осуществлении деятельности";
                if let Some(lvl) = hierarchy.get_lvl(&o.get_text())
                {
                    match lvl
                    {
                        1 => 
                        {
                            items.push(item);
                            last_item_hierarchy = Some(1);
                        }
                        2 => 
                        {
                            let last_item = items.last_mut().unwrap(); 
                            last_item.subitems.push(item);
                            last_item_hierarchy = Some(2);
                        }
                        3 => 
                        {
                            //FIXME пока сделал заглушку так, вся фишка в том что есть законы которые вносят изменение в изменяющие законы, это настоящая боль в заднице...
                            //19) части 2 и 3 статьи 32 изложить в следующей редакции:
                            //"2. Части 1, 3 - 8 статьи 19, части 1 - 5 и 9 статьи 20, статья 21 и части 5 и 6 статьи 22 настоящего Федерального закона вступают в силу с 1 сентября 2024 года.
                            //3. Часть 7 статьи 12, части 6 - 8, 10 и 11 статьи 20 настоящего Федерального закона вступают в силу с 1 сентября 2025 года.".
                            //по факту 3 это часть изменения но определяется как команда, и ничего тут не сделаешь, либо считать нумерацию команд, либо...
                            //во всяком случае можно пока на это забить я вообще не думаю что можно нормально распарсить такие документы.
                            if let Some(last_item) = 
                            items.last_mut().unwrap()
                            .subitems.last_mut()
                            {
                                last_item.subitems.push(item);
                                last_item_hierarchy = Some(3);
                            }
                            else 
                            {
                                items.last_mut().unwrap().changes.push(o.clone());
                            }
                            
                        }
                        4 => 
                        {
                            let last_item = 
                            items.last_mut().unwrap()
                            .subitems.last_mut().unwrap()
                            .subitems.last_mut().unwrap();
                            last_item.subitems.push(item);
                            last_item_hierarchy = Some(4);
                        }
                        _ => {println!("Иерархия `{}` не найдена!", o.get_text()); ()}
                    }
                }
                else
                {
                    if let Some(last) = last_item_hierarchy
                    {
                        ChangesHierarchy::not_numbered_items(&mut items, item, last);
                    }
                }
            }
        }
        return Ok(ChangesHierarchy {target_document: target, root_items: root_item, items});
    }

    fn not_numbered_items(items: &mut Vec<ChangesHierarchyItem>, item: ChangesHierarchyItem, lvl: usize)
    {
        match lvl
        {
            0 => 
            {
                items.push(item);
            }
            1 => 
            {
                let last_item = items.last_mut().unwrap(); 
                last_item.subitems.push(item);
            }
            2 => 
            {
                let last_item = 
                items.last_mut().unwrap()
                .subitems.last_mut().unwrap();
                last_item.subitems.push(item);
            }
            3 => 
            {
                let last_item = 
                items.last_mut().unwrap()
                .subitems.last_mut().unwrap()
                .subitems.last_mut().unwrap();
                last_item.subitems.push(item);
            }
            _ => ()
        }
    }
    pub fn get_items(&self) -> &Vec<ChangesHierarchyItem>
    {
        &self.items
    }
    pub fn get_target_document(&self) -> Option<&TargetDocument>
    {
        self.target_document.as_ref()
    }
    pub fn get_root_items(&self) -> Option<&ChangesHierarchyItem>
    {
        self.root_items.as_ref()
    }
}


impl Deserializer for ChangesHierarchy
{
    fn load<P: AsRef<Path>>(path: P) -> Result<Box<Self>, Box<dyn Error>>
    {
        let path = Path::new(path.as_ref());
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let des = serde_json::from_reader(reader)?;
        Ok(Box::new(des)) 
    }
}