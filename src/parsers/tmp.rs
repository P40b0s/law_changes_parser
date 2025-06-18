// #[derive(Debug, Default)]
// pub struct MermaidDiagramData {
//     pub nodes: Vec<MermaidNode>,
//     pub edges: Vec<MermaidEdge>,
//     pub total_changes: u32,
// }

// #[derive(Debug)]
// pub struct MermaidNode {
//     pub id: u64,
//     pub label: String,
//     pub change_path: ChangePath,
//     pub level: usize,
// }

// #[derive(Debug)]
// pub struct MermaidEdge {
//     pub from_id: u64,
//     pub to_id: u64,
//     pub label: Option<String>,
// }

// pub fn generate_mermaid_data(
//     changes: &[Change],
//     parent: Option<(&u64, &ChangePath)>,
//     start_level: usize,
// ) -> MermaidDiagramData {
//     let mut data = MermaidDiagramData::default();
//     let mut stack = Vec::new();
    
//     stack.push((changes.to_vec(), parent, start_level));

//     while let Some((current_changes, current_parent, current_level)) = stack.pop() {
//         // Группировка изменений
//         let mut groups = HashMap::new();
//         for c in &current_changes {
//             let paths = c.target_path.get_paths_with_id();
//             if let Some(cp) = paths.get(current_level) {
//                 groups.entry(cp.clone())
//                     .or_insert_with(Vec::new)
//                     .push(c.clone());
//             }
//         }

//         // Обработка групп
//         for ((id, cp), ch) in groups.into_iter().rev() {
//             // Создаем узел
//             let node = MermaidNode {
//                 id: *id,
//                 label: cp.as_markdown(),
//                 change_path: cp.clone(),
//                 level: current_level,
//             };
//             data.nodes.push(node);

//             // Обработка связей и изменений
//             if let Some((p_id, _)) = current_parent {
//                 let edge = if let Some(last_change) = ch.last() {
//                     if let Some(last_change_path) = last_change.target_path.get_paths().last() {
//                         if cp == *last_change_path {
//                             // Учет изменений
//                             if let Some(change_actions) = last_change.changes.as_ref() {
//                                 data.total_changes += change_actions.len() as u32;
//                             } else {
//                                 data.total_changes += 1;
//                             }
                            
//                             // Создание ребра с меткой изменений
//                             MermaidEdge {
//                                 from_id: *p_id,
//                                 to_id: *id,
//                                 label: Some(format!("{:?}", last_change.changes)),
//                             }
//                         } else {
//                             // Обычное ребро без изменений
//                             MermaidEdge {
//                                 from_id: *p_id,
//                                 to_id: *id,
//                                 label: None,
//                             }
//                         }
//                     } else {
//                         MermaidEdge {
//                             from_id: *p_id,
//                             to_id: *id,
//                             label: None,
//                         }
//                     }
//                 } else {
//                     MermaidEdge {
//                         from_id: *p_id,
//                         to_id: *id,
//                         label: None,
//                     }
//                 };
//                 data.edges.push(edge);
//             }

//             // Добавляем в стек для дальнейшей обработки
//             stack.push((ch, Some((id, cp)), current_level + 1));
//         }
//     }

//     data
// }

// fn convert_to_mermaid(diagram_data: &MermaidDiagramData) -> String {
//     let mut dia = String::new();
    
//     // Добавляем узлы
//     for node in &diagram_data.nodes {
//         dia.push_str(&format!("  {}[\"{} (level {})\"]\n", 
//             node.id, 
//             node.label, 
//             node.level
//         ));
//     }
    
//     // Добавляем связи
//     for edge in &diagram_data.edges {
//         if let Some(label) = &edge.label {
//             dia.push_str(&format!("  {} --> |{}| {}\n", 
//                 edge.from_id, 
//                 label, 
//                 edge.to_id
//             ));
//         } else {
//             dia.push_str(&format!("  {} --> {}\n", 
//                 edge.from_id, 
//                 edge.to_id
//             ));
//         }
//     }
    
//     dia
// }

// let mermaid_string = convert_to_mermaid(&diagram_data);


// // Получение статистики
// println!("Total nodes: {}", diagram_data.nodes.len());
// println!("Total edges: {}", diagram_data.edges.len());
// println!("Total changes: {}", diagram_data.total_changes);

// // Фильтрация по уровню
// let top_level_nodes: Vec<_> = diagram_data.nodes
//     .iter()
//     .filter(|n| n.level == 0)
//     .collect();