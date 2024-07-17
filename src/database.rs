use crate::plugin::GeyserPluginPostgresError;
use crate::schema::chain_transactions::dsl::chain_transactions;
use diesel::r2d2::ConnectionManager;
use diesel::{
    r2d2, Insertable, PgConnection, QueryDsl, Queryable, RunQueryDsl, Selectable, SelectableHelper,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Database {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

#[derive(Queryable, Insertable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::chain_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionDTO {
    pub signature: String,
    pub fee: i64,
    pub slot: i32,
}
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
impl Database {
    pub fn new(url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Pool creation failure");
        pool.get()
            .expect("cannot get pool")
            .run_pending_migrations(MIGRATIONS)
            .unwrap();
        Self { pool }
    }

    pub fn add_transaction(&mut self, tx: TransactionDTO) -> Result<(), GeyserPluginPostgresError> {
        diesel::insert_into(chain_transactions)
            .values(&tx)
            .execute(&mut self.pool.get().unwrap())?;
        Ok(())
    }

    pub fn query_all(&self) -> Result<Vec<TransactionDTO>, GeyserPluginPostgresError> {
        let res = chain_transactions
            .limit(100)
            .select(TransactionDTO::as_select())
            .load(&mut self.pool.get().unwrap());
        match res {
            Ok(v) => Ok(v),
            Err(e) => Err(GeyserPluginPostgresError::DatabaseError(e)),
        }
    }
}
