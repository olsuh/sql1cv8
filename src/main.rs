use std::error::Error;
pub(crate) type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V>;

mod consts;
mod creater;
mod init_objects;
mod metadata;
mod parser;
mod processing;
mod queries;
pub(crate) use metadata::Metadata;

const CONNECTION_STRING: &str =
    "postgres://benchmarkdbuser:benchmarkdbpass@tfb-database/hello_world";
const METADATA_FILE_NAME: &str = "metadata.json";

static SRC_QUERY: &str = r#"
SELECT items.[$Ссылка] AS item_id
      ,items.[$Код] AS item_code
      ,items.[$Наименование] AS item_descr
FROM [$Справочник.Номенклатура] AS items
WHERE items.[$ПометкаУдаления] = 0
"#;

#[ntex::main]
async fn main() -> Result<()> {
    let creater = creater::AppCreater::create(CONNECTION_STRING, METADATA_FILE_NAME).await;
    let m = creater.load_newer().await?;
    println!("Версия метаданных: {}", m.version);

    let qry = m.parse(SRC_QUERY)?;
    println!("Результат:\n{}", qry);

    Ok(())
}
