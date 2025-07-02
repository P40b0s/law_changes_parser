use std::collections::{HashMap, HashSet, VecDeque};
use indexmap::IndexMap;
use serde::Serialize;
use crate::{change_path::ChangePath, outputs::AsMarkdown, parsers::changes_parser::{Change, Changes}};

#[derive(Debug, Serialize)]
pub struct ChangeNode 
{
    pub id: u64,
    pub change_path: ChangePath,
    pub change: Option<Change>,
    pub level: usize,
}

#[derive(Debug, Serialize)]
pub struct ChangeEdge 
{
    pub from_id: u64,
    pub to_id: u64
}

///Для построения графа изменений
#[derive(Debug, Default, Serialize)]
pub struct ChangesGraph
{
    pub nodes: Vec<ChangeNode>,
    pub edges: Vec<ChangeEdge>,
    //pub nodes: HashMap<u64, ChangeNode>,
    pub total_changes: u32,
}

impl ChangesGraph
{
    fn get_node(&self, node_id: &u64) -> &ChangeNode
    {
        self.nodes.iter().find(|f| &f.id == node_id).unwrap()
    }
    fn get_nodes_as_markdown(&self, node_ids: &[u64]) -> Vec<String>
    {
        self.nodes.iter().filter(|f| node_ids.contains(&f.id)).map(|m| m.change_path.as_markdown()).collect()
    }
    fn get_nodes(&self, node_ids: &[u64]) -> Vec<&ChangeNode>
    {
        self.nodes.iter().filter(|f| node_ids.contains(&f.id)).collect()
    }
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
    ///получение всех связвнных нод с каждолй нодой указанного уровня `пункт 2->пункт 3->абзац 2->абзац 3->статья 20`
    pub fn get_descendants_for_level_bfs(&self, level: usize) -> HashMap<u64, Vec<&ChangeNode>> 
    {
        let mut result = HashMap::new();
        let root_nodes = self.nodes.iter()
            .filter(|node| node.level == level)
            .collect::<Vec<_>>();
        
        for root_node in root_nodes 
        {
            let mut descendants = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(root_node.id);
            let mut visited = HashSet::new();
            visited.insert(root_node.id);
            
            while let Some(current_id) = queue.pop_front() 
            {
                for edge in self.edges.iter().filter(|e| e.from_id == current_id) 
                {
                    if !visited.contains(&edge.to_id) 
                    {
                        if let Some(node) = self.nodes.iter().find(|n| n.id == edge.to_id) 
                        {
                            if node.level != level 
                            {
                                descendants.push(node);
                            }
                            queue.push_back(node.id);
                            visited.insert(node.id);
                        }
                    }
                }
            }
            result.insert(root_node.id, descendants);
        }
        
        result
    }
   
    /// получаем рутовую ноду и список всех дочерних элементов последним из которых идет элемент с change.is_some()
    pub fn get_changed_descendants_with_paths(&self, level: usize) -> IndexMap<u64, Vec<ChangePathInfo>> 
    {
        // Создаем индекс связей для быстрого поиска детей
        let children_index: IndexMap<u64, Vec<u64>> = self.edges.iter()
            .fold(IndexMap::new(), |mut acc, edge| 
            {
                acc.entry(edge.from_id).or_default().push(edge.to_id);
                acc
            });

        // Создаем индекс нод для быстрого доступа
        let node_index: IndexMap<u64, &ChangeNode> = self.nodes.iter()
            .map(|node| (node.id, node))
            .collect();

        let mut result = IndexMap::new();

        // Обрабатываем каждую корневую ноду указанного уровня
        for root_node in self.nodes.iter().filter(|n| n.level == level) 
        {
            let mut changed_descendants = Vec::new();
            let mut stack = vec![(root_node.id, vec![root_node.id])]; // (node_id, current_path)
            let mut visited = HashSet::new();
            while let Some((current_id, current_path)) = stack.pop() 
            {
                if !visited.insert(current_id) 
                {
                    continue;
                }
                // Проверяем детей текущей ноды
                if let Some(children) = children_index.get(&current_id) 
                {
                    for &child_id in children 
                    {
                        if let Some(child_node) = node_index.get(&child_id) 
                        {
                            let mut new_path = current_path.clone();
                            new_path.push(child_id);
                            // Если у ноды есть изменения, добавляем в результат
                            if child_node.change.is_some() 
                            {
                                changed_descendants.push(ChangePathInfo 
                                {
                                    node: child_node,
                                    path: new_path.clone(),
                                });
                            }
                            // Продолжаем обход
                            stack.push((child_id, new_path));
                        }
                    }
                }
            }
            result.insert(root_node.id, changed_descendants);
        }
        result
    }

    pub fn get_changed_descendants_with_paths2(&self, level: usize) -> IndexMap<u64, Vec<ChangePathInfo2>> 
    {
        // Создаем индекс связей для быстрого поиска детей
        let children_index: IndexMap<u64, Vec<u64>> = self.edges.iter()
            .fold(IndexMap::new(), |mut acc, edge| 
            {
                acc.entry(edge.from_id).or_default().push(edge.to_id);
                acc
            });

        // Создаем индекс нод для быстрого доступа
        let node_index: IndexMap<u64, &ChangeNode> = self.nodes.iter()
            .map(|node| (node.id, node))
            .collect();

        let mut result = IndexMap::new();

        // Обрабатываем каждую корневую ноду указанного уровня
        for root_node in self.nodes.iter().filter(|n| n.level == level) 
        {
            let mut changed_descendants = Vec::new();
            let mut stack = vec![(root_node.id, vec![root_node.id])]; // (node_id, current_path)
            let mut visited = HashSet::new();
            while let Some((current_id, current_path)) = stack.pop() 
            {
                if !visited.insert(current_id) 
                {
                    continue;
                }
                // Проверяем детей текущей ноды
                if let Some(children) = children_index.get(&current_id) 
                {
                    for &child_id in children 
                    {
                        if let Some(child_node) = node_index.get(&child_id) 
                        {
                            let mut new_path = current_path.clone();
                            new_path.push(child_id);
                            // Если у ноды есть изменения, добавляем в результат
                            if child_node.change.is_some() 
                            {
                                changed_descendants.push(ChangePathInfo2 
                                {
                                    node: child_node,
                                    path: new_path.iter().map(|n| &node_index.get(n).unwrap().change_path).collect(),
                                });
                            }
                            // Продолжаем обход
                            stack.push((child_id, new_path));
                        }
                    }
                }
            }
            result.insert(root_node.id, changed_descendants);
        }
        result
    }

}

pub struct ChangePathInfo<'a> 
{
    pub node: &'a ChangeNode,
    pub path: Vec<u64>,
}
pub struct ChangePathInfo2<'a> 
{
    pub node: &'a ChangeNode,
    pub path: Vec<&'a ChangePath>,
}
// impl<'a> Into<ChangePathInfo2<'a>> for ChangePathInfo<'a>
// {
//     fn into(self) -> ChangePathInfo2<'a> 
//     {
//         let nodes = graph.get_nodes_as_markdown(&m.path);
//     }
// }

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
    use crate::{outputs::AsMarkdown, parsers::changes_parser::Changes, ChangeNode, ChangesGraph};
    #[test]
    fn test_graph()
    {
        logger::StructLogger::new_default();
        let test_data = include_str!("..\\test_data\\test_1.txt");
        let changes_list = Changes::get_changes(test_data);
        logger::debug!("{}", serde_json::to_string_pretty(&changes_list).unwrap());
        let gr: ChangesGraph = changes_list.into();
        logger::debug!("Total nodes : {}", gr.nodes.len());
        logger::debug!("Total edges: {}", gr.edges.len());
        logger::debug!("Total changes: {}", gr.total_changes);
        for n in &gr.nodes
        {
            let path = gr.get_parent_nodes(n);
            logger::debug!("{}", n.change_path.as_markdown());
            if !path.is_empty()
            {
                let fullpath: Vec<String> = path.iter().map(|m| m.change_path.as_markdown()).collect();
                let fullpath = fullpath.join("->");
                let fullpath = [&fullpath, "->", &n.change_path.as_markdown()].concat();
                //logger::debug!("fullpath: {}", fullpath);
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
      #[test]
    fn test_graph3()
    {
        logger::StructLogger::new_default();
        let test_data = include_str!("..\\test_data\\test_1.txt");
        let changes_list = Changes::get_changes(test_data);
        let graph: ChangesGraph = changes_list.into();
        let changes_nodes = graph.get_descendants_for_level_bfs(0);
        for ch in &changes_nodes
        {
            let fullpath: Vec<String> = ch.1.iter().map(|m| m.change_path.as_markdown()).collect();
            let fullpath = fullpath.join("->");
            let root_node =  graph.nodes.iter().find(|f| &f.id == ch.0).unwrap();
            let fullpath = [&fullpath, "->", &root_node.change_path.as_markdown()].concat();
            logger::debug!("fullpath: {}", fullpath);
        }
        
        //logger::debug!("{}", serde_json::to_string_pretty(&changes_list).unwrap())
    }
    #[test]
    fn test_graph4()
    {
        logger::StructLogger::new_default();
        let test_data = include_str!("..\\test_data\\test_1.txt");
        let changes_list = Changes::get_changes(test_data);
        let graph: ChangesGraph = changes_list.into();
        let changes_nodes =  graph.get_changed_descendants_with_paths(0);
        for ch in &changes_nodes
        {
            let fullpath: Vec<Vec<String>> = ch.1.iter().map(|m| 
            {
                let nodes = graph.get_nodes_as_markdown(&m.path);
                nodes
            }).collect();

            for v in fullpath
            {
                let fp = v.join("->");
                //let root_node =  graph.nodes.iter().find(|f| &f.id == ch.0).unwrap();
                //let fp = [&fp, "->", &root_node.change_path.as_markdown()].concat();
                logger::debug!("fullpath: {}", fp);
            }
            logger::debug!("root: {}", ch.0);
        }
        //logger::debug!("{}", serde_json::to_string_pretty(&changes_list).unwrap())
    }

     #[test]
    fn test_graph5()
    {
        logger::StructLogger::new_default();
        let test_data = include_str!("..\\test_data\\test_1.txt");
        let changes_list = Changes::get_changes(test_data);
        let graph: ChangesGraph = changes_list.into();
        let changes_nodes =  graph.get_changed_descendants_with_paths2(0);
        for ch in &changes_nodes
        {
            let fullpath: Vec<Vec<String>> = ch.1.iter().map(|m| 
            {
                let nodes = m.path.iter().map(|p| p.as_markdown()).collect();
                nodes
            }).collect();

            for v in fullpath
            {
                let fp = v.join("->");
                //let root_node =  graph.nodes.iter().find(|f| &f.id == ch.0).unwrap();
                //let fp = [&fp, "->", &root_node.change_path.as_markdown()].concat();
                logger::debug!("fullpath: {}", fp);
            }
            logger::debug!("root: {}", ch.0);
        }
        //logger::debug!("{}", serde_json::to_string_pretty(&changes_list).unwrap())
    }
}