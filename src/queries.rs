use crate::HashMap;
use crate::Result;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "queries/"]
struct SqlQueries;

lazy_static::lazy_static! {
    pub static ref QRY_GET_DB_VERSION: String = String::from_utf8(SqlQueries::get("getDBVersion.sql").unwrap().data.into_owned()).unwrap();
    static ref QRY_GET_DB_EN: String = String::from_utf8(SqlQueries::get("getDB_en.sql").unwrap().data.into_owned()).unwrap();
    static ref QRY_GET_DB_RU: String = String::from_utf8(SqlQueries::get("getDB_ru.sql").unwrap().data.into_owned()).unwrap();
    pub static ref QRY_GET_CV_NAMES: String = String::from_utf8(SqlQueries::get("getCVNames.sql").unwrap().data.into_owned()).unwrap();
    pub static ref QRY_GET_DB_NAMES: String = String::from_utf8(SqlQueries::get("getDBNames.sql").unwrap().data.into_owned()).unwrap();

    pub static ref QRY_GET_DB: HashMap<&'static str, String> = {
        let mut map = HashMap::new();
        map.insert("en", QRY_GET_DB_EN.clone());
        map.insert("ru", QRY_GET_DB_RU.clone());
        map
    };
}

use tokio_postgres::{connect, Client, Statement};

/// Postgres interface
pub struct PgConnection {
    pub(crate) cl: Client,
    db_version: Statement,
    db_data: Statement,
    cv_names: Statement,
    db_names: Statement,
    //buf: RefCell<BytesMut>,
}

impl PgConnection {
    pub async fn connect(db_url: &str) -> PgConnection {
        let (cl, conn) = connect(db_url)
            .await
            .expect("can not connect to postgresql");

        ntex::rt::spawn(async move {
            let _ = conn.await;
        });

        let db_version = cl.prepare(&QRY_GET_DB_VERSION).await.unwrap();
        let db_data = cl.prepare(&QRY_GET_DB_RU).await.unwrap();
        let cv_names = cl.prepare(&QRY_GET_CV_NAMES).await.unwrap();
        let db_names = cl.prepare(&QRY_GET_DB_NAMES).await.unwrap();

        PgConnection {
            cl,
            db_version,
            db_data,
            cv_names,
            db_names,
            //buf: RefCell::new(BytesMut::with_capacity(10 * 1024 * 1024)),
        }
    }
}

impl PgConnection {
    pub async fn db_version(&self) -> Result<String> {
        let rows = self.cl.query_raw(&self.db_version, &[]).await?;

        let version = rows.first().unwrap().get(0);
        Ok(version)
    }

    pub async fn db_data(
        &self,
    ) -> Vec<
        Result<(
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        )>,
    > {
        let rows = self.cl.query_raw(&self.db_data, &[]).await.unwrap();
        let rows = Vec::from_iter(rows.iter().map(|row| {
            Ok((
                row.get::<_, String>(0),
                row.get::<_, String>(1),
                row.get::<_, String>(2),
                row.get::<_, String>(3),
                row.get::<_, String>(4),
                row.get::<_, String>(5),
                row.get::<_, String>(6),
                row.get::<_, String>(7),
                row.get::<_, String>(8),
                row.get::<_, String>(9),
                row.get::<_, String>(10),
                row.get::<_, String>(11),
            ))
        }));
        rows
    }

    pub async fn db_names(&self) -> Result<Vec<u8>> {
        let rows = self.cl.query_raw(&self.db_names, &[]).await?;

        let db_names: &[u8] = rows.first().unwrap().get(0);
        Ok(db_names.into())
    }

    pub async fn cv_names(&self) -> Result<Vec<u8>> {
        let rows = self.cl.query_raw(&self.cv_names, &[]).await?;

        let cv_names: &[u8] = rows.first().unwrap().get(0);
        Ok(cv_names.into())
    }
}
