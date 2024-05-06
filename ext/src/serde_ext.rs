use async_trait::async_trait;
use edgedb_errors::ErrorKind;
use edgedb_protocol::query_arg::QueryArgs;

#[async_trait]
pub trait SerdeClientExt {
    async fn query_required_single_serde<T>(
        &self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<T, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned;

    async fn query_serde<T>(
        &self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<T, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned;

    async fn query_single_serde<T>(
        &self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<Option<T>, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned;
}

#[async_trait]
impl SerdeClientExt for edgedb_tokio::Client {
    async fn query_required_single_serde<T>(
        &self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<T, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let json = self.query_required_single_json(query, args).await?;
        serde_json::from_str(&json).map_err(edgedb_errors::UserError::with_source)
    }

    async fn query_serde<T>(
        &self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<T, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let json = self.query_json(query, args).await?;
        serde_json::from_str(&json).map_err(edgedb_errors::UserError::with_source)
    }

    async fn query_single_serde<T>(
        &self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<Option<T>, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let json_opt = self.query_single_json(query, args).await?;
        json_opt
            .map(|json| serde_json::from_str(&json).map_err(edgedb_errors::UserError::with_source))
            .transpose()
    }
}

#[async_trait(?Send)]
pub trait SerdeTrxExt {
    async fn query_required_single_serde<T>(
        &mut self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<T, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned;

    async fn query_serde<T>(
        &mut self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<T, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned;

    async fn query_single_serde<T>(
        &mut self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<Option<T>, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned;
}

#[async_trait(?Send)]
impl SerdeTrxExt for edgedb_tokio::Transaction {
    async fn query_required_single_serde<T>(
        &mut self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<T, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let json = self.query_required_single_json(query, args).await?;
        serde_json::from_str(&json).map_err(edgedb_errors::UserError::with_source)
    }

    async fn query_serde<T>(
        &mut self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<T, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let json = self.query_json(query, args).await?;
        serde_json::from_str(&json).map_err(edgedb_errors::UserError::with_source)
    }

    async fn query_single_serde<T>(
        &mut self,
        query: &str,
        args: &impl QueryArgs,
    ) -> Result<Option<T>, edgedb_tokio::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let json_opt = self.query_single_json(query, args).await?;
        json_opt
            .map(|json| serde_json::from_str(&json).map_err(edgedb_errors::UserError::with_source))
            .transpose()
    }
}
