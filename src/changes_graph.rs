use std::collections::{HashMap, VecDeque};

use indexmap::IndexMap;

use crate::{change_path::ChangePath, parsers::changes_parser::{Change, Changes}};



#[derive(Debug)]
pub struct ChangeNode 
{
    pub id: u64,
    pub change_path: ChangePath,
    pub change: Option<Change>,
    pub level: usize,
}

#[derive(Debug)]
pub struct ChangeEdge 
{
    pub from_id: u64,
    pub to_id: u64
}
///Для построения графа изменений
#[derive(Debug, Default)]
pub struct ChangesGraph
{
    pub nodes: Vec<ChangeNode>,
    pub edges: Vec<ChangeEdge>,
    pub total_changes: u32,
}
impl ChangesGraph
{
    pub fn get_parent_nodes<'a>(&'a self, node: &'a ChangeNode) -> Vec<&'a ChangeNode>
    {
        let mut nodes = Vec::new();
        let mut queue = VecDeque::new();
        if node.change.is_some()
        {
            queue.push_back(node.id);
            while let Some(id) = queue.pop_front()
            {
                if let Some(e) = self.edges.iter().find(|f| f.to_id == id)
                {
                    if let Some(n) = self.nodes.iter().find(|f| f.id == e.from_id)
                    {
                        nodes.push(n);
                        queue.push_back(n.id);
                    }
                    
                }
            }
        }
        nodes.reverse();
        nodes
    }
}

impl Into<ChangesGraph> for Changes
{
    ///v2 - с построением графа
    fn into(self) -> ChangesGraph
    { 
        let mut data = ChangesGraph::default();
        let mut queue: VecDeque<(Vec<Change>, Option<(u64, ChangePath)>, usize)> = VecDeque::new();
        queue.push_back((self.0, None, 0));
        while let Some((current_changes, parent, level)) = queue.pop_front() 
        {
            //нам нужно соблюдать порядок, поэтому возьму indexmap
            let mut groups = IndexMap::new();
            for change in current_changes.into_iter()
            {
                if let Some(path) = change.target_path.get_path_by_level(level)
                {
                    groups.entry(path)
                    .or_insert_with(Vec::new)
                    .push(change);
                }
            }
            for ((id, cp), ch) in groups.into_iter()
            {
                if let Some((p_id, _)) = parent
                {
                    if let Some(last_change) = ch.last()
                    {
                        if let Some(last_change_path) = last_change.target_path.get_paths().last()
                        {
                            if &cp == last_change_path
                            {
                                let node = ChangeNode 
                                {
                                    id,
                                    change_path: cp.clone(),
                                    change: Some(last_change.clone()),
                                    level: level,
                                };
                                data.nodes.push(node);
                                data.edges.push(ChangeEdge 
                                {
                                    from_id: p_id,
                                    to_id: id,
                                });
                            }
                            else 
                            {
                                let node = ChangeNode 
                                {
                                    id,
                                    change_path: cp.clone(),
                                    change: None,
                                    level: level,
                                };
                                data.nodes.push(node);
                                data.edges.push(ChangeEdge 
                                {
                                    from_id: p_id,
                                    to_id: id
                                });
                            }
                        }
                    }
                }
                else 
                {
                    let node = ChangeNode 
                    {
                        id,
                        change_path: cp.clone(),
                        change: None,
                        level: level,
                    };
                    data.nodes.push(node);
                }
                if !ch.is_empty()
                {
                    queue.push_back((ch, Some((id, cp.clone())), level + 1));
                }
            }
        }
        data.total_changes = data.nodes.iter().filter(|f| f.change.is_some()).count() as u32;
        data
    }
}

#[cfg(test)]
mod tests
{
    use crate::{outputs::AsMarkdown, parsers::changes_parser::Changes, ChangesGraph};

    #[test]
    fn test_graph()
    {
        logger::StructLogger::new_default();
        let test_data = include_str!("..\\test_data\\test_1.txt");
        let changes_list = Changes::get_changes(test_data);
        let gr: ChangesGraph = changes_list.into();
        logger::debug!("Total nodes : {}", gr.nodes.len());
        logger::debug!("Total edges: {}", gr.edges.len());
        logger::debug!("Total changes: {}", gr.total_changes);
        for n in &gr.nodes
        {
            let path = gr.get_parent_nodes(n);
            if !path.is_empty()
            {
                let fullpath: Vec<String> = path.iter().map(|m| m.change_path.as_markdown()).collect();
                let fullpath = fullpath.join("->");
                let fullpath = [&fullpath, "->", &n.change_path.as_markdown()].concat();
                logger::debug!("fullpath: {}", fullpath);
            }
        }
        assert_eq!(gr.nodes.len(), 13);
        assert_eq!(gr.edges.len(), 9);
        assert_eq!(gr.total_changes, 7);
    }

     #[test]
    fn test_graph2()
    {
        logger::StructLogger::new_default();
        let test_data = include_str!("..\\test_data\\test_2.txt");
        let changes_list = Changes::get_changes(test_data);
        let graph: ChangesGraph = changes_list.into();
        for n in graph.nodes
        {
            logger::debug!("{}", n.change_path.as_markdown());
        }
        //logger::debug!("{}", serde_json::to_string_pretty(&changes_list).unwrap())
    }
}