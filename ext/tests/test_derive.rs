use edgedb_derive::Queryable;
use edgedb_tokio_ext_derive::Project;
use uuid::Uuid;

#[derive(Project, Queryable)]
struct User {
    id: Uuid,
    name: String,
    #[project(alias = "age")]
    age_value: i64,
    #[project(alias = "org")]
    organization: Organization,
}

#[derive(Project, Queryable)]
struct Organization {
    users: Vec<User>,
    deez: Deez,
}

#[derive(Project, Queryable)]
struct Deez {
    id: Uuid,
}

#[tokio::test]
async fn test_derive_project() {
    println!("{:?}", User::project());
}
