// Импорт необходимых библиотек
use crate::{processing, queries::PgConnection, HashMap};
use std::ops::Deref;

use crate::{
    metadata::Object,
    processing::{CVNames, DBNames, Enums, Points},
    Metadata, Result,
};

// Структура initedObjects
pub(crate) struct InitedObjects {
    pub(crate) metadata: Metadata,
    fields: Box<HashMap<&'static str, &'static str>>,
    types: Box<HashMap<&'static str, &'static str>>,
    dbnames: DBNames,
    pub(crate) cvnames: CVNames,
    enums: HashMap<String, Enums>,
    points: HashMap<String, Points>,
}

impl InitedObjects {
    pub fn obj(
        &self,
        d_type: &str,
        d_number: &str,
        d_name: &str,
        d_prefix: &str,
        d_suffix: &str,
    ) -> Option<Object> {
        if d_type == "Fld" {
            if let Some(name) = self.fields.get(d_number) {
                return Some(Object {
                    r#type: d_number[1..].to_string(),
                    number: d_number.to_string(),
                    db_name: d_name.to_string(),
                    cv_name: format!("{}{}{}", d_prefix, name, d_suffix),
                    synonyms: field_synonyms(d_number),
                    ..Default::default()
                });
            }
        }

        let d = self.dbnames.m.get(&format!("{}{}", d_type, d_number))?;
        let c = self.cvnames.m.get(&d.ids)?;

        Some(Object {
            uuid: d.ids.clone(),
            r#type: d.typ.clone(),
            number: d.num.clone(),
            db_name: d_name.to_string(),
            cv_name: format!("{}{}{}", d_prefix, c.val, d_suffix),
            synonyms: c.syn.clone(),
            ..Default::default()
        })
    }

    pub(crate) fn agregs_insert(&mut self, o: &Object, agreg: &str) {
        let mut qc = String::new();
        let mut qd = String::new();

        let agregs = match agreg {
            "Enum" => &self.enums,
            "RoutePoint" => &self.points,
            _ => unreachable!(),
        };

        for e in &agregs[&o.uuid] {
            let name = format!("{}.{}", o.cv_name, e.val);
            self.metadata.objects.insert(
                name.clone(),
                Object {
                    r#type: agreg.to_string() + "Order",
                    db_name: e.num.clone(),
                    cv_name: name.clone(),
                    synonyms: e.syn.clone(),
                    ..Default::default()
                },
            );

            qc.push_str(&format!(" when {} then '{}'", e.num, e.val));
            qd.push_str(&format!(
                " when {} then '{}'",
                e.num, e.syn[&self.metadata.language]
            ));

            let dollar_name = format!("${}", name);
            self.metadata.objects.insert(
                dollar_name.clone(),
                Object {
                    r#type: agreg.to_string() + "RRef",
                    db_name: format!(
                        "(select top 1 _IDRRef from {} where _{agreg}Order = {})",
                        o.db_name, e.num
                    ),
                    cv_name: dollar_name,
                    ..Default::default()
                },
            );
        }

        let name = format!("${}", o.cv_name);
        let qry = format!("(select _IDRRef, case _{agreg}Order{} end _Code, case _{agreg}Order{} end _Description from {})", qc, qd, o.db_name);

        self.metadata.objects.insert(
            name.clone(),
            Object {
                uuid: o.uuid.clone(),
                r#type: agreg.to_string() + "Virtual",
                number: o.number.clone(),
                db_name: qry,
                cv_name: name,
                synonyms: o.synonyms.clone(),
                params: {
                    let mut params = HashMap::new();
                    self.params_insert(&mut params, "_IDRRef");
                    self.params_insert(&mut params, "_Code");
                    self.params_insert(&mut params, "_Description");
                    // Добавьте аналогичные вставки для _Code и _Description
                    params
                },
            },
        );
    }
    #[inline]
    fn param_object(&self, param: &str) -> Object {
        Object {
            r#type: param[1..].to_string(),
            number: param.to_string(),
            db_name: param.to_string(),
            cv_name: self.fields[param].to_string(),
            synonyms: field_synonyms(param),
            ..Default::default()
        }
    }
    #[inline]
    fn params_insert(&self, params: &mut HashMap<String, Object>, param: &str) {
        params.insert(self.fields[param].to_string(), self.param_object(param));
    }

    pub fn types_insert(&mut self) {
        for (value, name) in self.types.deref() {
            self.metadata.objects.insert(
                name.to_string(),
                Object {
                    r#type: "Type".to_string(),
                    db_name: value.to_string(),
                    cv_name: name.to_string(),
                    ..Default::default()
                },
            );
        }
    }

    pub(crate) fn rtref_insert(&mut self, table_object: &Object) {
        let rt_ref_bin = match table_object.rt_ref_bin() {
            Ok(s) => s,
            Err(_e) => return,
        };

        let name = format!("{}.{}", table_object.cv_name, self.fields["_IDTRef"]);

        self.metadata.objects.insert(
            name.clone(),
            Object {
                r#type: "TRef".to_string(),
                db_name: rt_ref_bin,
                cv_name: name,
                synonyms: field_synonyms("_IDTRef"),
                ..Default::default()
            },
        );
    }

    pub async fn init_objects(conn: &PgConnection) -> Result<InitedObjects> {
        let metadata = Metadata {
            language: "ru".to_string(),
            objects: HashMap::with_capacity(65536),
            version: conn.db_version().await?,
        };

        let bin = conn.db_names().await?;
        let dbnames = processing::processing_db_names(bin.as_slice());

        let bin = conn.cv_names().await?;
        let cvnames = processing::processing_cv_names(bin.as_slice());

        let mut enums = HashMap::with_capacity(dbnames.cnt_enums);
        let stmt = conn.cl.prepare(&dbnames.qry_enums).await.unwrap();
        let rows = conn.cl.query(&stmt, &[]).await.unwrap();
        for row in rows {
            let k = row.get::<_, String>(0);
            let v = row.get::<_, &[u8]>(1);
            let v = processing::processing_enums(v);
            enums.insert(k, v);
        }

        let mut points = HashMap::with_capacity(dbnames.cnt_points);
        let stmt = conn.cl.prepare(&dbnames.qry_points).await.unwrap();
        let rows = conn.cl.query(&stmt, &[]).await.unwrap();
        for row in rows {
            let k = row.get::<_, String>(0);
            let v = row.get::<_, &[u8]>(1);
            let v = processing::processing_points(v);
            points.insert(k, v);
        }

        let obj = InitedObjects {
            types: types(&metadata.language),
            fields: fields(&metadata.language),
            metadata,
            dbnames,
            cvnames,
            enums,
            points,
        };

        Ok(obj)
    }
}

fn field_synonyms(field: &str) -> HashMap<String, String> {
    let x = crate::consts::FIELD_SYNONYMS.get(field).unwrap();
    let z = x.iter().map(|(k, v)| (k.to_string(), v.to_string()));

    let h = HashMap::from_iter(z);
    h
}

fn types(language: &str) -> Box<HashMap<&'static str, &'static str>> {
    let x = crate::consts::TYPES.get(language).unwrap();
    let y = x.to_owned();
    Box::new(y)
}

fn fields(language: &str) -> Box<HashMap<&'static str, &'static str>> {
    let x = crate::consts::FIELDS.get(language).unwrap();
    let y = x.to_owned();
    Box::new(y)
}
