use deadpool::managed::{self, Pool};
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

#[derive(Debug)]
pub struct Manager {
    path: String,
}

impl Manager {
    /// Create a new `Manager` that handles creating and recyling connections from a
    /// pool to a `SurrealDB` instance.
    ///
    /// # Panics
    ///
    /// Panics if the runtime cannot be initialized.
    #[must_use]
    pub fn new(path: &str, size: usize) -> managed::Pool<Manager> {
        Pool::builder(Manager {
            path: path.to_string(),
        })
        .max_size(size)
        .build()
        .expect("No runtime (tokio/async-std) specified")
    }
}

impl managed::Manager for Manager {
    type Error = surrealdb::Error;
    type Type = Surreal<Db>;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let db = Surreal::new::<RocksDb>(self.path.as_str()).await?;

        db.use_ns("adam").use_db("adam").await?;

        Ok(db)
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        conn.invalidate().await.map_err(Self::Error::from)?;
        conn.use_ns("adam")
            .use_db("adam")
            .await
            .map_err(Self::Error::from)?;

        Ok(())
    }
}
