#![allow(dead_code)]

use edgedb_derive::Queryable;
use edgedb_tokio_ext_derive::{query_project, Project};
use uuid::Uuid;

#[derive(Project, Queryable)]
struct User {
    id: Uuid,
    #[project(exp = ".name.first")]
    first_name: String,
    #[project(exp = ".name.last")]
    last_name: String,
    #[project(alias = "age")]
    age_value: i64,
    #[project(alias = "org", nested)]
    organization: Option<Organization>,
}

#[derive(Project, Queryable)]
struct Organization {
    id: Uuid,
    name: String,
}

#[tokio::test]
async fn test_derive_project() {
    let edgedb_client = edgedb_tokio::create_client().await.unwrap();
    let query = query_project!(
        "
        with
            user := (insert User {
                name := { first := 'Lurian', last := 'Warlock'},
                age := 26
            })
        select user { project::User }
    "
    );
    println!("{}", query);
    let user = edgedb_client
        .query_required_single::<User, _>(query, &())
        .await
        .unwrap();
    assert_eq!(user.first_name, "Lurian");
}
