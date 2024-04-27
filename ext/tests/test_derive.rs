use edgedb_tokio_ext_derive::{query_project, Project};

#[derive(Project)]
struct Parent {
    id: String,
}

#[tokio::test]
async fn test_derive_project() {
    let query = query_project! {"
        select User {
            project::Parent
        }
    "
    };
    println!("{:?}", query);
}
