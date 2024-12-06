use std::process::exit;

use crate::HashMap;
use crate::Result;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "queries/"]
struct SqlQueries;

lazy_static::lazy_static! {
    #[derive(Debug)]
    pub static ref QRY_GET_DB_VERSION: String = String::from_utf8(SqlQueries::get("getDBVersion.sql").unwrap().data.into_owned()).unwrap();
    static ref QRY_GET_DB_EN: String = String::from_utf8(SqlQueries::get("getDB_en.sql").unwrap().data.into_owned()).unwrap();
    #[derive(Debug)]
    static ref QRY_GET_DB_RU: String = String::from_utf8(SqlQueries::get("getDB_ru.sql").unwrap().data.into_owned()).unwrap();
    #[derive(Debug)]
    pub static ref QRY_GET_CV_NAMES: String = String::from_utf8(SqlQueries::get("getCVNames.sql").unwrap().data.into_owned()).unwrap();
    #[derive(Debug)]
    pub static ref QRY_GET_DB_NAMES: String = String::from_utf8(SqlQueries::get("getDBNames.sql").unwrap().data.into_owned()).unwrap();

    #[derive(Debug)]
    pub static ref QRY_GET_DB: HashMap<&'static str, String> = {
        let mut map = HashMap::new();
        map.insert("en", QRY_GET_DB_EN.clone());
        map.insert("ru", QRY_GET_DB_RU.clone());
        map
    };
}

//use tokio_postgres::{connect, Client, Statement};

use tiberius::Row;
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;
use tokio_util::compat::TokioAsyncWriteCompatExt;

/// Postgres interface
pub struct PgConnection {
    pub(crate) cl: Client<Compat<TcpStream>>,
    db_version: String,
    db_data: String,
    cv_names: String,
    db_names: String,
    //buf: RefCell<BytesMut>,
}
impl PgConnection {
    pub async fn ms_conn(db_url: &str) -> anyhow::Result<Client<Compat<TcpStream>>> {
        let config = Config::from_jdbc_string(db_url).unwrap();

        let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
        tcp.set_nodelay(true)?;

        let client: Client<tokio_util::compat::Compat<TcpStream>> =
            match Client::connect(config, tcp.compat_write()).await {
                Ok(c) => c,
                Err(e) => {
                    println!("{e}");
                    exit(1)
                }
            };

        Ok(client)
    }
    pub async fn connect(db_url: &str) -> PgConnection {
        /*let (cl, conn) = connect(db_url)
            .await
            .expect("can not connect to postgresql");


        ntex::rt::spawn(async move {
            let _x = conn.await.unwrap();
        });*/

        let cl = Self::ms_conn(db_url).await.unwrap();
        let db_data = QRY_GET_DB_RU.to_string();
        let cv_names = QRY_GET_CV_NAMES.clone();
        let db_names = QRY_GET_DB_NAMES.clone();
        let db_version = QRY_GET_DB_VERSION.clone();

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
    pub async fn db_version(&mut self) -> Result<String> {
        let rows = self.cl.query(&self.db_version, &[]).await?;

        let version = rows
            .into_row()
            .await
            .unwrap()
            .unwrap()
            .get::<&str, _>(0)
            .unwrap()
            .to_string();
        Ok(version)
    }

    pub async fn db_data(
        &mut self,
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
        let rows = self
            .cl
            .query(&self.db_data, &[])
            .await
            .unwrap()
            .into_first_result()
            .await
            .unwrap();
        let rows = Vec::from_iter(rows.iter().map(|row| {
            Ok((
                row.get::<&str, _>(0).unwrap().to_string(),
                row.get::<&str, _>(1).unwrap().to_string(),
                row.get::<&str, _>(2).unwrap().to_string(),
                row.get::<&str, _>(3).unwrap().to_string(),
                row.get::<&str, _>(4).unwrap().to_string(),
                row.get::<&str, _>(5).unwrap().to_string(),
                row.get::<&str, _>(6).unwrap().to_string(),
                row.get::<&str, _>(7).unwrap().to_string(),
                row.get::<&str, _>(8).unwrap().to_string(),
                row.get::<&str, _>(9).unwrap().to_string(),
                row.get::<&str, _>(10).unwrap().to_string(),
                row.get::<&str, _>(11).unwrap().to_string(),
            ))
        }));
        rows
    }

    pub async fn db_names(&mut self) -> Row {
        self.cl
            .query(&self.db_names, &[])
            .await
            .unwrap()
            .into_row()
            .await
            .unwrap()
            .unwrap()
    }

    pub async fn cv_names(&mut self) -> Row {
        self.cl
            .query(&self.cv_names, &[])
            .await
            .unwrap()
            .into_row()
            .await
            .unwrap()
            .unwrap()
    }
}
