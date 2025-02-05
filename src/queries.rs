use crate::Result;
use rust_embed::RustEmbed;
use welds::{self, Client};

use welds_connections::mssql::MssqlClient;
use welds_connections::postgres::PostgresClient;
use welds_connections::{Param, Row};

#[derive(RustEmbed)]
#[folder = "queries/"]
struct SqlQueries;

lazy_static::lazy_static! {
    #[derive(Debug)]
    static ref QRY_GET_DB_RU_MS: String = String::from_utf8(SqlQueries::get("getDB_ru_ms.sql").unwrap().data.into_owned()).unwrap();
    static ref QRY_GET_DB_RU_PG: String = String::from_utf8(SqlQueries::get("getDB_ru_pg.sql").unwrap().data.into_owned()).unwrap();
}

const MS_QRY_GET_CV_NAMES: &'static str =
    "select BinaryData from dbo.Params where FileName = '1a621f0f-5568-4183-bd9f-f6ef670e7090.si'";
const PG_QRY_GET_CV_NAMES: &'static str =
    "select binarydata FROM params WHERE filename = '1a621f0f-5568-4183-bd9f-f6ef670e7090.si'";
const MS_QRY_GET_DB_NAMES: &'static str =
    "select BinaryData from dbo.Params where FileName = 'DBNames'";
const PG_QRY_GET_DB_NAMES: &'static str =
    "select binarydata FROM params WHERE filename = 'DBNames'";

const MS_QRY_GET_DB_VERSION: &'static str =
    "select convert(varchar, max(modify_date), 120) Version from sys.tables"; // 120 стиль : гггг-мм-дд чч:мм:сс
const PG_QRY_GET_DB_VERSION: &'static str =
    "select to_char(modified, 'YYYY-MM-DD HH24:MI:SS') FROM params WHERE filename = 'DBNamesVersion'";

pub enum SQLClient {
    MSSQL(MssqlClient),
    PGSQL(PostgresClient),
}
impl SQLClient {}
pub struct SQLConnection {
    pub(crate) cl: SQLClient,
    db_version: &'static str,
    db_data: &'static str,
    cv_names: &'static str,
    db_names: &'static str,
    pub(crate) is_pg_sql: bool,
    //buf: RefCell<BytesMut>, //buf: RefCell::new(BytesMut::with_capacity(10 * 1024 * 1024)),
}
impl SQLConnection {
    pub async fn connect(db_url: &str) -> SQLConnection {
        if !db_url.starts_with("postgres://") {
            SQLConnection {
                cl: SQLClient::MSSQL(welds::connections::mssql::connect(db_url).await.unwrap()),
                db_version: MS_QRY_GET_DB_VERSION,
                db_data: QRY_GET_DB_RU_MS.as_str(),
                cv_names: MS_QRY_GET_CV_NAMES,
                db_names: MS_QRY_GET_DB_NAMES,
                is_pg_sql: false,
            }
        } else {
            SQLConnection {
                cl: SQLClient::PGSQL(welds::connections::postgres::connect(db_url).await.unwrap()),
                db_version: PG_QRY_GET_DB_VERSION,
                db_data: QRY_GET_DB_RU_PG.as_str(),
                cv_names: PG_QRY_GET_CV_NAMES,
                db_names: PG_QRY_GET_DB_NAMES,
                is_pg_sql: true,
            }
        }
    }
}

impl SQLConnection {
    pub async fn db_version(&self) -> Result<String> {
        let row = self.fetch_rows(self.db_version, &[]).await.pop().unwrap();

        let version = row.get_by_position::<String>(0).unwrap();
        Ok(version)
    }

    pub async fn db_data(
        &mut self,
    ) -> Vec<(
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
    )> {
        let rows = self.fetch_rows(self.db_data, &[]).await;
        let rows = Vec::from_iter(rows.iter().map(|row| {
            (
                row.get_by_position::<String>(0).expect("db_data row[0]"),
                row.get_by_position::<String>(1).expect("db_data row[1]"),
                row.get_by_position::<String>(2).expect("db_data row[2]"),
                row.get_by_position::<String>(3).expect("db_data row[3]"),
                row.get_by_position::<String>(4).expect("db_data row[4]"),
                row.get_by_position::<String>(5).expect("db_data row[5]"),
                row.get_by_position::<String>(6).expect("db_data row[6]"),
                row.get_by_position::<String>(7).expect("db_data row[7]"),
                row.get_by_position::<String>(8).expect("db_data row[8]"),
                row.get_by_position::<String>(9).expect("db_data row[9]"),
                row.get_by_position::<String>(10).expect("db_data row[10]"),
                row.get_by_position::<String>(11).expect("db_data row[11]"),
            )
        }));
        rows
    }

    pub async fn fetch_rows(&self, sql: &str, params: &[&(dyn Param + Sync)]) -> Vec<Row> {
        let v = match &self.cl {
            SQLClient::MSSQL(mssql_client) => mssql_client.fetch_rows(sql, params).await,
            SQLClient::PGSQL(postgres_client) => postgres_client.fetch_rows(sql, params).await,
        };
        match v {
            Ok(v) => v,
            Err(e) => {
                println!("{sql}");
                println!("{e}");
                panic!("");
            }
        }
    }

    pub async fn db_names(&self) -> Vec<Row> {
        self.fetch_rows(self.db_names, &[]).await
    }

    pub async fn cv_names(&self) -> Vec<Row> {
        self.fetch_rows(self.cv_names, &[]).await
    }
}

/*#[derive(Default, Debug)]
pub struct DBConf {
    data_type: String,
    table_name: String,
    field_name: String,
    table_prefix: String,
    table_number: String,
    table_suffix: String,
    vt_prefix: String,
    vt_number: String,
    vt_suffix: String,
    field_prefix: String,
    field_number: String,
    field_suffix: String,
}*/
