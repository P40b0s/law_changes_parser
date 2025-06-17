pub mod numbers;
pub mod words;
mod apply;
pub mod diagramm;
pub use apply::apply_all;
pub mod consts;
//mod tags;
mod replace;
pub use replace::replace_all;
use std::{fs::File, path::Path, error::Error, io::BufReader};
mod tags;
pub use tags::{apply, in_new_edition, next_is_content, lost_power};
pub use consts::{ITEM_NUMBER, HEADER_NUMBER, ALPHA_NUMERIC, INDENT_NUMBERS};
//mod change_action;
pub mod changes_checker;
//mod changes_map;
//pub use changes_map::{ChangesMap, RemainTokens};
//mod target_document;
//pub use target_document::{DocumentType, TargetDocument};
mod path_plus_number;
pub use path_plus_number::{paths, only_path_definition};

//pub use changes_checker::check_if_change;
//pub use change_action::ChangeAction;
pub mod changes_parser;
mod chars;
pub use chars::{space0, space1};
pub mod paths_list;



// pub trait Deserializer
// {
//     fn load<P: AsRef<Path>>(path: P) -> Result<Box<Self>, Box<dyn Error>>;
// }

// impl Deserializer for (Option<TargetDocument>, Vec<ChangesMap>)
// {
//     fn load<P: AsRef<Path>>(path: P) -> Result<Box<Self>, Box<dyn Error>>
//     {
//         let path = Path::new(path.as_ref());
//         let file = File::open(path)?;
//         let reader = BufReader::new(file);
//         let des = serde_json::from_reader(reader)?;
//         Ok(Box::new(des)) 
//     }

    
// }


