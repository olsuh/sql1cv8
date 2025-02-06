use sql1cv8::*;
use std::env;

const SRC_QUERY: &str = r#"  -- /*comment/**/ [$Справочник.Номенклатура]
  /* /*[$Справочник.Номенклатура]*/ /*[$Справочник.Номенклатура]*/   */SELECT items.[$Ссылка] AS item_id
      ,items.[$Код] AS item_code
      ,items.[$Наименование] AS item_descr
FROM [$Справочник.Номенклатура] AS items
WHERE items.[$ПометкаУдаления] = 0
"#;

#[ntex::main]
async fn main() -> Result<()> {
    let file_name1 = "metadata_1.json";
    let file_name2 = "metadata_2.json";

    let _x = std::fs::remove_file(file_name1);
    let _x = std::fs::remove_file(file_name2);

    let password = env::var("DB_PSW").expect("Переменная среды DB_PSW");
    //let db_url = &format!("jdbc:sqlserver://localhost:1434;databaseName=ut;user=sa;password={password};");
    let db_url1 = &format!("server=127.0.0.1,1434;databaseName=ut;user=sa;password={password};TrustServerCertificate=true;");
    let db_url2 = &format!("postgres://postgres:{password}@127.0.0.1/ut"); //5432
    let mut qry1 = test_with_create_load_file(db_url1, file_name1, SRC_QUERY).await?;
    let mut qry2 = test_with_create_load_file(db_url2, file_name2, SRC_QUERY).await?;

    qry1 = qry1.to_lowercase();
    qry2 = qry2.to_lowercase();
    assert_eq!(qry1, qry2);

    conpare_two_files(file_name1, file_name2).await;
    conpare_two_files(file_name2, file_name1).await;

    Ok(())
}

async fn _test_only_from_db(db_url: &str, file: &str, query: &str) -> Result<String> {
    let mut loader = loader::MetaDataLoader::ini(db_url, file).await;

    let metadata = loader.load_from_db().await?;
    let qry = metadata.parse(query)?;
    println!("Результат:\n{}", qry);
    Ok(qry)
}

async fn test_with_create_load_file(db_url: &str, file: &str, query: &str) -> Result<String> {
    let mut loader = loader::MetaDataLoader::ini(db_url, file).await;
    let metadata = loader.load_newer().await?;
    println!("Версия метаданных: {}", metadata.version);

    let query_for_db = metadata.parse(query)?;
    println!("Результат:\n{}", query_for_db);
    Ok(query_for_db)
}

async fn conpare_two_files(file1: &str, file2: &str) {
    let l1 = loader::MetaDataLoader::ini("", file1).await;
    let l2 = loader::MetaDataLoader::ini("", file2).await;
    let m1 = l1.load_from_file().unwrap();
    let m2 = l2.load_from_file().unwrap();

    for (k, obj1) in m1.objects.iter() {
        let obj2 = m2.objects.get(k).unwrap();
        compare_two_obj(obj1, obj2, k);
    }
}

fn compare_two_obj(obj1: &Object, obj2: &Object, k: &str) {
    let key = k.to_owned();

    for (k, v1) in obj1.synonyms.iter() {
        match obj2.synonyms.get(k) {
            Some(v2) => {
                if v1 != v2 {
                    println!("{} - syn {} != {}", obj1.cv_name, v1, v2)
                }
            }
            None => {
                println!("{} {} - syn {} - не нашли ", key, obj1.cv_name, v1)
            }
        };
    }

    for (k, param1) in obj1.params.iter() {
        let param2 = obj2.params.get(k).unwrap();
        compare_two_obj(param1, param2, &(key.clone() + "\\" + &k));
    }
}
