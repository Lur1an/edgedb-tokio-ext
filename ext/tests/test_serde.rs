#![allow(dead_code)]

use edgedb_tokio_ext::SerdeClientExt;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
struct User {
    id: Uuid,
    name: Name,
    age: i64,
    org: Option<Organization>,
}

#[derive(Deserialize, Debug)]
struct Name {
    first: String,
    last: String,
}

#[derive(Deserialize, Debug)]
struct Organization {
    id: Uuid,
    name: String,
}

#[tokio::test]
async fn test_derive_project() {
    let edgedb_client = edgedb_tokio::create_client().await.unwrap();
    let query = "
        with
            org := (insert Organization {
                name := 'Edgedb'
            }),
            user := (insert User {
                name := (first := 'Lurian', last := 'Warlock'),
                age := 26,
                org := org
            })
        select user { ** }
    ";

    let user = edgedb_client
        .query_required_single_serde::<User>(query, &())
        .await
        .unwrap();
    assert_eq!(user.name.first, "Lurian");
}
