use super::error::Result;
use super::future::BoxFuture;
use super::keys::wrap::WrapKeyMethod;
use super::options::IntoOptions;
use super::store::{Backend, ManageBackend, QueryBackend, Scan, Session, Store};
use super::types::{Entry, EntryKind, EntryOperation, EntryTag, TagFilter};

#[cfg(feature = "postgres")]
use super::postgres::PostgresStore;

#[cfg(feature = "sqlite")]
use super::sqlite::SqliteStore;

pub type AnyStore = Store<AnyBackend>;

pub type AnySession = Session<AnyQueryBackend>;

#[derive(Debug)]
pub enum AnyBackend {
    #[cfg(feature = "postgres")]
    Postgres(PostgresStore),

    #[cfg(feature = "sqlite")]
    Sqlite(SqliteStore),

    #[allow(unused)]
    Other,
}

impl Backend for AnyBackend {
    type Session = AnyQueryBackend;
    type Transaction = AnyQueryBackend;

    fn create_profile(&self, name: Option<&str>) -> BoxFuture<Result<String>> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(store) => store.create_profile(name),

            #[cfg(feature = "sqlite")]
            Self::Sqlite(store) => store.create_profile(name),

            _ => unreachable!(),
        }
    }

    fn remove_profile(&self, name: String) -> BoxFuture<Result<bool>> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(store) => store.remove_profile(name),

            #[cfg(feature = "sqlite")]
            Self::Sqlite(store) => store.remove_profile(name),

            _ => unreachable!(),
        }
    }

    fn scan(
        &self,
        profile: Option<String>,
        kind: EntryKind,
        category: String,
        tag_filter: Option<TagFilter>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> BoxFuture<Result<Scan<'static, Entry>>> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(store) => store.scan(profile, kind, category, tag_filter, offset, limit),

            #[cfg(feature = "sqlite")]
            Self::Sqlite(store) => store.scan(profile, kind, category, tag_filter, offset, limit),

            _ => unreachable!(),
        }
    }

    fn session(&self, profile: Option<String>) -> BoxFuture<Result<Self::Session>> {
        Box::pin(async move {
            match self {
                #[cfg(feature = "postgres")]
                Self::Postgres(store) => {
                    let session = store.session(profile).await?;
                    Ok(AnyQueryBackend::PostgresSession(session))
                }

                #[cfg(feature = "sqlite")]
                Self::Sqlite(store) => {
                    // FIXME - avoid double boxed futures by exposing public method
                    let session = store.session(profile).await?;
                    Ok(AnyQueryBackend::SqliteSession(session))
                }

                _ => unreachable!(),
            }
        })
    }

    fn transaction(&self, profile: Option<String>) -> BoxFuture<Result<Self::Transaction>> {
        Box::pin(async move {
            match self {
                #[cfg(feature = "postgres")]
                Self::Postgres(store) => {
                    // FIXME - avoid double boxed futures by exposing public method
                    let session = store.transaction(profile).await?;
                    Ok(AnyQueryBackend::PostgresTxn(session))
                }

                #[cfg(feature = "sqlite")]
                Self::Sqlite(store) => {
                    // FIXME - avoid double boxed futures by exposing public method
                    let session = store.transaction(profile).await?;
                    Ok(AnyQueryBackend::SqliteTxn(session))
                }

                _ => unreachable!(),
            }
        })
    }

    fn close(&self) -> BoxFuture<Result<()>> {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(store) => store.close(),

            #[cfg(feature = "sqlite")]
            Self::Sqlite(store) => store.close(),

            _ => unreachable!(),
        }
    }
}

pub enum AnyQueryBackend {
    #[cfg(feature = "postgres")]
    PostgresSession(<PostgresStore as Backend>::Session),
    #[cfg(feature = "postgres")]
    PostgresTxn(<PostgresStore as Backend>::Transaction),

    #[cfg(feature = "sqlite")]
    SqliteSession(<SqliteStore as Backend>::Session),
    #[cfg(feature = "sqlite")]
    SqliteTxn(<SqliteStore as Backend>::Transaction),

    #[allow(unused)]
    Other,
}

impl QueryBackend for AnyQueryBackend {
    fn count<'q>(
        &'q mut self,
        kind: EntryKind,
        category: &'q str,
        tag_filter: Option<TagFilter>,
    ) -> BoxFuture<'q, Result<i64>> {
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresSession(session) => session.count(kind, category, tag_filter),
            #[cfg(feature = "postgres")]
            Self::PostgresTxn(txn) => txn.count(kind, category, tag_filter),

            #[cfg(feature = "sqlite")]
            Self::SqliteSession(session) => session.count(kind, category, tag_filter),
            #[cfg(feature = "sqlite")]
            Self::SqliteTxn(txn) => txn.count(kind, category, tag_filter),

            _ => unreachable!(),
        }
    }

    fn fetch<'q>(
        &'q mut self,
        kind: EntryKind,
        category: &'q str,
        name: &'q str,
        for_update: bool,
    ) -> BoxFuture<'q, Result<Option<Entry>>> {
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresSession(session) => session.fetch(kind, category, name, for_update),
            #[cfg(feature = "postgres")]
            Self::PostgresTxn(txn) => txn.fetch(kind, category, name, for_update),

            #[cfg(feature = "sqlite")]
            Self::SqliteSession(session) => session.fetch(kind, category, name, for_update),
            #[cfg(feature = "sqlite")]
            Self::SqliteTxn(txn) => txn.fetch(kind, category, name, for_update),

            _ => unreachable!(),
        }
    }

    fn fetch_all<'q>(
        &'q mut self,
        kind: EntryKind,
        category: &'q str,
        tag_filter: Option<TagFilter>,
        limit: Option<i64>,
        for_update: bool,
    ) -> BoxFuture<'q, Result<Vec<Entry>>> {
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresSession(session) => {
                session.fetch_all(kind, category, tag_filter, limit, for_update)
            }
            #[cfg(feature = "postgres")]
            Self::PostgresTxn(txn) => txn.fetch_all(kind, category, tag_filter, limit, for_update),

            #[cfg(feature = "sqlite")]
            Self::SqliteSession(session) => {
                session.fetch_all(kind, category, tag_filter, limit, for_update)
            }
            #[cfg(feature = "sqlite")]
            Self::SqliteTxn(txn) => txn.fetch_all(kind, category, tag_filter, limit, for_update),

            _ => unreachable!(),
        }
    }

    fn remove_all<'q>(
        &'q mut self,
        kind: EntryKind,
        category: &'q str,
        tag_filter: Option<TagFilter>,
    ) -> BoxFuture<'q, Result<i64>> {
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresSession(session) => session.remove_all(kind, category, tag_filter),
            #[cfg(feature = "postgres")]
            Self::PostgresTxn(txn) => txn.remove_all(kind, category, tag_filter),

            #[cfg(feature = "sqlite")]
            Self::SqliteSession(session) => session.remove_all(kind, category, tag_filter),
            #[cfg(feature = "sqlite")]
            Self::SqliteTxn(txn) => txn.remove_all(kind, category, tag_filter),

            _ => unreachable!(),
        }
    }

    fn update<'q>(
        &'q mut self,
        kind: EntryKind,
        operation: EntryOperation,
        category: &'q str,
        name: &'q str,
        value: Option<&'q [u8]>,
        tags: Option<&'q [EntryTag]>,
        expiry_ms: Option<i64>,
    ) -> BoxFuture<'q, Result<()>> {
        match self {
            #[cfg(feature = "postgres")]
            Self::PostgresSession(session) => {
                session.update(kind, operation, category, name, value, tags, expiry_ms)
            }
            #[cfg(feature = "postgres")]
            Self::PostgresTxn(txn) => {
                txn.update(kind, operation, category, name, value, tags, expiry_ms)
            }

            #[cfg(feature = "sqlite")]
            Self::SqliteSession(session) => {
                session.update(kind, operation, category, name, value, tags, expiry_ms)
            }
            #[cfg(feature = "sqlite")]
            Self::SqliteTxn(txn) => {
                txn.update(kind, operation, category, name, value, tags, expiry_ms)
            }

            _ => unreachable!(),
        }
    }

    fn close(self, commit: bool) -> BoxFuture<'static, Result<()>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::SqliteSession(session) => session.close(commit),
            #[cfg(feature = "sqlite")]
            Self::SqliteTxn(txn) => txn.close(commit),

            #[cfg(feature = "postgres")]
            Self::PostgresSession(session) => session.close(commit),
            #[cfg(feature = "postgres")]
            Self::PostgresTxn(txn) => txn.close(commit),

            _ => unreachable!(),
        }
    }
}

impl<'a> ManageBackend<'a> for &'a str {
    type Store = AnyStore;

    fn open_backend(
        self,
        method: Option<WrapKeyMethod>,
        pass_key: Option<&'a str>,
    ) -> BoxFuture<'a, Result<Self::Store>> {
        Box::pin(async move {
            let opts = self.into_options()?;
            debug!("Open store with options: {:?}", &opts);

            match opts.schema.as_ref() {
                #[cfg(feature = "postgres")]
                "postgres" => {
                    let opts = super::postgres::PostgresStoreOptions::new(opts)?;
                    let mgr = opts.open(method, pass_key).await?;
                    Ok(Store::new(AnyBackend::Postgres(mgr.into_inner())))
                }

                #[cfg(feature = "sqlite")]
                "sqlite" => {
                    let opts = super::sqlite::SqliteStoreOptions::new(opts)?;
                    let mgr = opts.open(method, pass_key).await?;
                    Ok(Store::new(AnyBackend::Sqlite(mgr.into_inner())))
                }

                _ => Err(err_msg!(Unsupported, "Invalid backend: {}", &opts.schema)),
            }
        })
    }

    fn provision_backend(
        self,
        method: WrapKeyMethod,
        pass_key: Option<&'a str>,
        recreate: bool,
    ) -> BoxFuture<'a, Result<Self::Store>> {
        Box::pin(async move {
            let opts = self.into_options()?;
            debug!("Provision store with options: {:?}", &opts);

            match opts.schema.as_ref() {
                #[cfg(feature = "postgres")]
                "postgres" => {
                    let opts = super::postgres::PostgresStoreOptions::new(opts)?;
                    let mgr = opts.provision(method, pass_key, recreate).await?;
                    Ok(Store::new(AnyBackend::Postgres(mgr.into_inner())))
                }

                #[cfg(feature = "sqlite")]
                "sqlite" => {
                    let opts = super::sqlite::SqliteStoreOptions::new(opts)?;
                    let mgr = opts.provision(method, pass_key, recreate).await?;
                    Ok(Store::new(AnyBackend::Sqlite(mgr.into_inner())))
                }

                _ => Err(err_msg!(Unsupported, "Invalid backend: {}", &opts.schema)),
            }
        })
    }

    fn remove_backend(self) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            let opts = self.into_options()?;
            debug!("Remove store with options: {:?}", &opts);

            match opts.schema.as_ref() {
                #[cfg(feature = "postgres")]
                "postgres" => {
                    let opts = super::postgres::PostgresStoreOptions::new(opts)?;
                    Ok(opts.remove().await?)
                }

                #[cfg(feature = "sqlite")]
                "sqlite" => {
                    let opts = super::sqlite::SqliteStoreOptions::new(opts)?;
                    Ok(opts.remove().await?)
                }

                _ => Err(err_msg!(Unsupported, "Invalid backend: {}", &opts.schema)),
            }
        })
    }
}
