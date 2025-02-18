use crate::{processing, queries::SQLConnection, HashMap};
use std::ops::Deref;
use crate::{
    metadata::Object,
    processing::{CVNames, DBNames, Enums, Points},
    Metadata, Result,
};
use tracing;

pub(crate) struct InitedObjects {
    pub(crate) metadata: Metadata,
    fields: Box<HashMap<&'static str, &'static str>>,
    types: Box<HashMap<&'static str, &'static str>>,
    dbnames: DBNames,
    cvnames: CVNames,
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
        c_type: &str,
    ) -> Option<Object> {
        if d_type == "Fld" {
            // стандатрные поля
            if let Some(name) = self.fields.get(d_number) {
                return Some(Object {
                    r#type: c_type.to_string(), //d_number[1..]
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

        let r#type = if d_type == "Fld" {c_type} else {&d.typ};
        Some(Object {
            uuid: d.ids.clone(),
            r#type: r#type.to_string(),
            number: d.num.clone(),
            db_name: d_name.to_string(),
            cv_name: format!("{}{}{}", d_prefix, c.val, d_suffix),
            synonyms: c.syn.clone(),
            ..Default::default()
        })
    }

    pub(crate) fn agregs_insert(&mut self, o: &Object, agreg: &str, is_pg_sql: bool) {
        let mut qc = String::new();
        let mut qd = String::new();

        let agregs = match agreg {
            "Enum" => &self.enums,
            "RoutePoint" => &self.points,
            _ => unreachable!(),
        };

        let vec = match agregs.get(&o.uuid) {
            Some(v) => v,
            None => {
                tracing::error!("{agreg} {} - not found", &o.uuid);
                return;
            },
        };

        for e in vec {
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
            let cast_top = if is_pg_sql { "" } else { "top 1 " };
            self.metadata.objects.insert(
                dollar_name.clone(),
                Object {
                    r#type: agreg.to_string() + "RRef",
                    db_name: format!(
                        "(select {cast_top}_IDRRef from {} where _{agreg}Order = {})",
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
    
    pub async fn init_objects(conn: &mut SQLConnection) -> Result<InitedObjects> {
        let metadata = Metadata {
            language: "ru".to_string(),
            objects: HashMap::with_capacity(65536),
            version: conn.db_version().await?,
        };

        let row = conn.db_names().await.pop().unwrap();
        let bin = row.get_by_position::<Vec<u8>>(0).unwrap();
        let bin = deflater(bin.as_ref());
        let dbnames = processing::processing_db_names(&bin, conn.is_pg_sql);

        let row = conn.cv_names().await.pop().unwrap();
        let bin = row.get_by_position::<Vec<u8>>(0).unwrap();
        let bin = deflater(bin.as_ref());
        let cvnames = processing::processing_cv_names(&bin);

        let mut enums = HashMap::with_capacity(dbnames.cnt_enums);

        let rows = conn.fetch_rows(&dbnames.qry_enums, &[]).await;
        for row in rows {
            /*if conn.is_pg_sql {
                let pg_row = row.as_postgres_row().unwrap();
                let x = pg_row.try_get_raw(0).unwrap();
                let y1 = x.as_bytes().unwrap();
                let s = String::from_utf8(y1.to_vec()).unwrap();
                dbg!(&s);
                let s2 = s.replace("\0", "");
                dbg!(&s2);
                let y2 = from_utf16le(y1).unwrap();
                dbg!(&y2);
                assert_eq!(s2,y2);

                //dbg!(&pg_row);
                continue;
            } else {
                
            }*/
            
            let Ok(k) = row.get_by_position::<String>(0) else {
                tracing::error!("row[0] qry_enums - error");
                continue;
            };
            let Ok(v) = row.get_by_position::<Vec<u8>>(1) else {
                tracing::error!("row[1] qry_enums - error");
                continue;
            };
            //let k = k.as_ref();
            //let k = String::from_utf8_lossy(k).into_owned();
            let v = deflater(v.as_ref());
            let v = processing::processing_enums(&v);
            //dbg!(&k,&v);
            enums.insert(k, v);
        }

        let mut points = HashMap::with_capacity(dbnames.cnt_points);

        let rows = conn.fetch_rows(&dbnames.qry_points, &[]).await;
        for row in rows {
            let Ok(k) = row.get_by_position::<String>(0) else {
                tracing::error!("row[0] qry_points - error");
                continue;
            };
            let Ok(v) = row.get_by_position::<Vec<u8>>(1) else {
                tracing::error!("row[1] qry_points - error");
                continue;
            };
            let v = deflater(v.as_ref());
            let v = processing::processing_points(&v);
            let k = k[..36].to_string();
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
    let x = match crate::consts::FIELD_SYNONYMS.get(field) {
        Some(x) => x,
        None => &HashMap::<&str, &str>::new(),
    };
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

pub fn deflater(bin: &[u8]) -> Vec<u8> {
    use flate2::read::DeflateDecoder;
    use std::io::Read;
    let mut deflater = DeflateDecoder::new(bin);
    let mut decompressed = Vec::new();
    deflater.read_to_end(&mut decompressed).unwrap();

    decompressed
}

//#![feature(array_chunks)]

/*pub fn from_utf16le(v: &[u8]) -> Result<String> {
    if v.len() % 2 != 0 {
        let e = "FromUtf16Error(())".into();
        return Err(e);
    }

    let v2 = unsafe{ std::slice::from_raw_parts(v.as_ptr() as *const u16, v.len()/2) };
    Ok(String::from_utf16(v2)?)
}*/

