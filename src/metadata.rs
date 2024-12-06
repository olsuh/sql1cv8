use crate::HashMap;
//use std::hash::Hash;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
//use serde_json::{Serializer, Deserializer};

// Объект метаданных
#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct Object {
    pub(crate) uuid: String,                      // Идентификатор
    pub(crate) r#type: String,                    // Тип объекта
    pub(crate) number: String,                    // Номер объекта DBNames
    pub(crate) db_name: String,                   // Имя в базе данных
    pub(crate) cv_name: String,                   // Имя в конфигурации
    pub(crate) synonyms: HashMap<String, String>, // Синонимы объекта
    pub(crate) params: HashMap<String, Object>,   // Параметры объекта
}

impl Object {
    // Возвращает ВидСсылки типа INT.
    pub fn _rt_ref_int(&self) -> Result<String, std::num::ParseIntError> {
        self.number.parse::<u32>()?;
        Ok(self.number.clone())
    }

    // Возвращает ВидСсылки типа BINARY(4).
    pub fn rt_ref_bin(&self) -> Result<String, std::num::ParseIntError> {
        let u = self.number.parse::<u32>()?;
        Ok(format!("0x{:08X}", u))
    }
}

// Метаданные
#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Metadata {
    pub(crate) version: String,                  // Версия метаданных
    pub(crate) language: String,                 // Язык конфигурации
    pub(crate) objects: HashMap<String, Object>, // Объекты метаданных первого уровня. Это либо таблицы, либо какие-то констаты вроде типов полей для составных типов, значения перечислений и виды ссылок
}

impl Metadata {
    // Сохраняет метаданные в файл.
    // В качестве параметров принимает строковую переменную:
    // s - имя файла, в котором хранится кэш метаданных в формате json.
    pub fn save_to_file(&self, s: &str) -> std::io::Result<()> {
        let b = serde_json::to_vec_pretty(self)?;
        let mut f = File::create(s)?;
        f.write_all(&b)?;
        Ok(())
    }
}
