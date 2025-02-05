use std::error::Error;
use std::env;
pub(crate) type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V>;

mod consts;
mod loader;
mod init_objects;
mod metadata;
mod parser;
mod processing;
mod queries;
pub(crate) use metadata::Metadata;

const METADATA_FILE_NAME: &str = "metadata.json";
const SRC_QUERY: &str = r#"  -- /*comment/**/ [$Справочник.Номенклатура]
  /* /*[$Справочник.Номенклатура]*/ /*[$Справочник.Номенклатура]*/   */SELECT items.[$Ссылка] AS item_id
      ,items.[$Код] AS item_code
      ,items.[$Наименование] AS item_descr
FROM [$Справочник.Номенклатура] AS items
WHERE items.[$ПометкаУдаления] = 0
"#;

#[ntex::main]
async fn main() -> Result<()> {
    let password = env::var("DB_PSW").expect("Установите переменную среды DB_PSW");
    //let ms_connection_string = &format!("jdbc:sqlserver://localhost:1434;databaseName=ut;user=sa;password={password};");
    let db_url = &format!("server=127.0.0.1,1434;databaseName=ut;user=sa;password={password};TrustServerCertificate=true;");
    test(db_url, METADATA_FILE_NAME).await?;
    let db_url = &format!("postgres://postgres:{password}@127.0.0.1/ut"); //5432
    test(db_url, METADATA_FILE_NAME).await?;

    test_with_create_load_file(db_url, METADATA_FILE_NAME, SRC_QUERY).await?;
    Ok(())
}

async fn test(db_url: &str, file: &str) -> Result<()> {
    let mut creater = loader::Conf1SLoader::ini(db_url, file).await;

    let m = creater.load_from_db().await?;
    let qry = m.parse(SRC_QUERY)?;
    println!("Результат:\n{}", qry);
    Ok(())
}

async fn test_with_create_load_file(db_url: &str, file: &str, query: &str) -> Result<()> {
    let mut creater = loader::Conf1SLoader::ini(db_url, file).await;
    let m = creater.load_newer().await?;
    println!("Версия метаданных: {}", m.version);

    let qry = m.parse(query)?;
    println!("Результат:\n{}", qry);
    Ok(())
}
