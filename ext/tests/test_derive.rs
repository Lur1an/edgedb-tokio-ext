#![allow(dead_code)]

use edgedb_derive::Queryable;
use edgedb_tokio_ext::{shaped_query, Shape};
use edgedb_tokio_ext_derive::tx_variant;
use uuid::Uuid;

#[derive(Shape, Queryable, Debug)]
struct User {
    id: Uuid,
    #[shape(exp = ".name.first")]
    first_name: String,
    #[shape(exp = ".name.last")]
    last_name: String,
    #[shape(alias = "age")]
    age_value: i64,
    #[shape(alias = "org", nested)]
    organization: Option<Organization>,
}

#[derive(Shape, Queryable, Debug)]
struct Organization {
    id: Uuid,
    name: String,
}

#[tx_variant]
async fn sample_db_function(id: &Uuid, client: &edgedb_tokio::Client) {
    client.query_required_single_json("", &(id,)).await.unwrap();
    todo!()
}

#[tokio::test]
async fn test_derive_project() {
    let edgedb_client = edgedb_tokio::create_client().await.unwrap();
    let query = shaped_query!(
        "
        with
            org := (insert Organization {
                name := 'Edgedb'
            }),
            user := (insert User {
                name := (first := 'Lurian', last := 'Warlock'),
                age := 26,
                org := org
            })
        select user { shape::User }
    "
    );
    let user = edgedb_client
        .query_required_single::<User, _>(query, &())
        .await
        .unwrap();
    assert_eq!(user.first_name, "Lurian");
}
