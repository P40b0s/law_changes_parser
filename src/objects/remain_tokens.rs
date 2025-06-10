use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemainTokens
{
    source_string: String,
    remains_tokens: String
}
impl RemainTokens
{
    pub fn new(source_string: &str, remains: &str) -> Self
    {
        RemainTokens { source_string: source_string.to_owned(), remains_tokens: remains.to_owned()}
    }
}
impl Display for RemainTokens
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        if self.remains_tokens.len() > 0
        {
            let error = ["Есть не полностью обработанная строка ->", &self.source_string, "| остаток токенов ->", &self.remains_tokens, "|"].concat();
            f.write_str(&error)
        }
        else 
        {
            f.write_str("")
        }
    }
}