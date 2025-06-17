use crate::objects::number::Number;

#[derive(serde::Serialize, Clone, PartialEq, serde::Deserialize, Debug, Hash)]
pub enum ChangeAction
{
    //слова "в области связи" заменить словами ", осуществляющий функции по контролю и надзору в сфере средств массовой информации, массовых коммуникаций, информационных технологий и связи"
    ReplaceWords 
    { 
        old_words: String,
        new_words: String
    },
    ///слова ... исключить
    ExcludeWords(String),
    ///<после слов "321"> дополнить словами "123" после слов опциональна потому что может быть без них, просто дополнить словами...
    AddWords 
    {
        after: Option<String>,
        words: String
    },
    ///например дополнить статьями....
    ApplyHeaders(Vec<String>),
    ///напирмер дополнить пунктами или подпунктами....
    ApplyItems(Vec<String>),
    //2) пункт 2 статьи 12 дополнить абзацем следующего содержания:
    //"устанавливает требования к эксплуатации сетей связи и управлению сетями связи в части использования операторами связи услуг сторонних организаций.";
    ApplyIndents(Vec<String>),
    ///дополнить предложением, создаем абзац и плюсуем его с предыдущим
    ApplySentence(String),
    ///в части 1 первое предложение изложить в следующей редакции:
    ReplaceSentence 
    { 
        number: u32,
        text: String 
    },
    ///напимер сатью ... изложить в новой редакции
    HeaderInNewEdition(String),
    ///напимер наименование статьи... изложить в новой редакции
    HeaderNameInNewEdition(String),
    HeaderNameOperations(Vec<ChangeAction>),
    ///Правки вносимые в наименование заголовка. Использовал ChangeAction потому что с названием может быть кроме изложения в новой редакции 3 вещи
    /// слова заменить на слова, дополнить словами или слова исключить
    /// у нас уже есть готовый механизм в виде 
    /// ReplaceWords\ExcludeWords\AddWords поэтому будем проверять только эти 3 ?? пока с этим тормозну, по ходу дела подумаю
    //HeaderNameOperations(Box<Vec<ChangeAction>>),
    ItemsInNewEdition(Vec<String>),
    IndentsInNewEdition(Vec<String>),
    ///пункт 1 признать утратившим силу
    LostPower(Number),
    ///часть 12 исключить;
    Exclude(Number),
    ///никаких комманд выполнять ненадо
    None
}