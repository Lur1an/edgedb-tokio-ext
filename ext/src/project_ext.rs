pub trait QueryProjection {
    fn project() -> &'static str;
}

#[derive(edgedb_derive::Queryable)]
struct Organization {
    name: String,
}

#[derive(edgedb_derive::Queryable)]
struct User {
    id: String,
    org: (String, Organization),
}
