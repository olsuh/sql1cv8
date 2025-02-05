use std::fs::File;
use std::io::Read;

use crate::init_objects::InitedObjects;
use crate::queries::SQLConnection;
use crate::{Metadata, Result};

pub(crate) struct Conf1SLoader {
    pub(crate) conn: SQLConnection,
    file: String,
}

impl Conf1SLoader {
    pub async fn ini(db_url: &str, file: &str) -> Self {
        let conn = SQLConnection::connect(db_url).await;
        Self {
            conn,
            file: file.into(),
        }
    }

    // Assuming Metadata, Object, and other related structs are defined elsewhere

    // LoadNewer возвращает метаданные из базы данных, либо из файла, если объекты в базе не менялись.
    // В качестве параметров принимает две строковые переменные:
    // cs - строка подключения, описание которой можно посмотреть по ссылке https://github.com/denisenkom/go-mssqldb#connection-parameters-and-dsn;
    // s - имя файла, в котором хранится кэш метаданных в формате json.
    // Возвращает объект Metadata.
    pub async fn load_newer(&mut self) -> Result<Metadata> {
        let mut m = self
            .load_from_file()
            .unwrap_or_else(|_| Metadata::default());

        let version = self.conn.db_version().await.unwrap();

        if m.version != version {
            m = self.load_from_db().await?;
            m.save_to_file(&self.file)?;
        }

        Ok(m)
    }

    // LoadFromFile возвращает метаданные из файла.
    // В качестве параметров принимает строковую переменную:
    // s - имя файла, в котором хранится кэш метаданных в формате json.
    // Возвращает объект Metadata.
    pub fn load_from_file(&self) -> Result<Metadata> {
        let mut file = File::open(&self.file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let m: Metadata = serde_json::from_str(&contents).unwrap();
        Ok(m)
    }

    pub async fn load_from_db(&mut self) -> Result<Metadata> {
        let mut obj_main = InitedObjects::init_objects(&mut self.conn).await?;
        obj_main.types_insert();

        let rows = self.conn.db_data().await;

        let mut to = String::new();
        let mut vo = String::new();
        let mut tt_cv_name = String::new();

        for row in rows {
            let (
                data_type,
                table_name,
                field_name,
                table_prefix,
                table_number,
                table_suffix,
                vt_prefix,
                vt_number,
                vt_suffix,
                field_prefix,
                field_number,
                field_suffix,
            ) = row;

            let tn = format!("{}{}{}", table_prefix, table_number, table_suffix);
            if to != tn {
                to = tn;

                let Some(table_object) = obj_main.obj(
                    &data_type,
                    &table_number,
                    &table_name,
                    &table_prefix,
                    &table_suffix,
                ) else {
                    continue;
                };

                tt_cv_name = table_object.cv_name.clone();

                match data_type.as_str() {
                    "Enum" => obj_main.agregs_insert(&table_object, "Enum"),
                    "BPrPoints" => obj_main.agregs_insert(&table_object, "RoutePoint"),
                    _ => {}
                }
                obj_main.rtref_insert(&table_object);

                obj_main
                    .metadata
                    .objects
                    .insert(tt_cv_name.clone(), table_object);
            }

            let vn = format!("{}{}{}", vt_prefix, vt_number, vt_suffix);
            if vo != vn && !vn.is_empty() {
                vo = vn;
                tt_cv_name = tt_cv_name.split(&vt_prefix).collect::<Vec<&str>>()[0].to_string();

                let Some(table_object) = obj_main.obj(
                    "VT",
                    &vt_number,
                    &table_name,
                    &format!("{}{}", tt_cv_name, vt_prefix),
                    &vt_suffix,
                ) else {
                    continue;
                };

                tt_cv_name = table_object.cv_name.clone();
                obj_main
                    .metadata
                    .objects
                    .insert(tt_cv_name.clone(), table_object);
            }

            let Some(field_object) = obj_main.obj(
                "Fld",
                &field_number,
                &field_name,
                &field_prefix,
                &field_suffix,
            ) else {
                continue;
            };

            let fl_cv_name = field_object.cv_name.clone();

            obj_main
                .metadata
                .objects
                .get_mut(&tt_cv_name)
                .unwrap()
                .params
                .insert(fl_cv_name, field_object);
        }

        Ok(obj_main.metadata)
    }
}
