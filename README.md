# edgedb-tokio-ext
A library of common helper traits, macros & functions when working with [edgedb-rust](https://github.com/edgedb/edgedb-rust)

## Serde Extension
This library implements `SerdeClientExt` and `SerdeTrxExt` on `edgedb_tokio::Client` and `edgedb_tokio::Transaction` respectively. There are two traits as on `Transaction` a mutable ref is required to run client functions and also the futures are not `Send`. To use this trait and seamlessly deserialize edgedb query return values into your `serde` structs import the trait.
```rust
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
```
Tip: this works really well when you have a polymorphic query that can return different subtypes of an abstract type, just use an `enum` and annotate with `#[serde(untagged)]`.

# Shaped queries
To avoid the hassle of having to always edit your queries when you edit the struct you want them to deserialize into I created two macros: `Shape` and `shaped_query`.
```rust
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
```
The macro takes care of creating a `shape` function that returns a `&'static str` with the body of the field selection. The macro expansion works recursively as well, and the projection for `Organization` will be applied to the `organization` field in `User`.

To interpolate the field selection into an EdgeDB query use the `shaped_query!` macro:
```rust
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
```
The macro attributes are quite simple:
- `nested` use this attribute if your field needs to expand the subtype.
- `exp` this attribute allows you to pass in the right hand side of the assignment, its exclusive with `alias`. You could build a *named tuple* inside of the expression that then is picked up by the `Queryable` struct or just follow a link and a property, destructure a named tuple field and much more... It's up to you how to use this
- `alias` Use this attribute if your struct field and the edgedb type field have different names. It is a shorthand for `exp = ".fieldname"`

## Tx Variant
There is an issue with code that should accept both a `&edgedb_tokio::Client` OR `&edgedb_tokio::Transaction` since there is no trait that unifies them. `Transaction` creates futures that are not `Send` and also requires a mutable reference to be used, this would make functions using such a trait quite limited for the cases they are used with a `Client` and not a `Transaction`. I still think there would be value in such a trait, however to fit my current needs I created the `tx_variant` macro that creates a function with the exact same name followed by `_tx` that accepts `&mut Transaction` rather than `&Client`.
This macro is still a thought experiment as for example the inner function body wouldn't replace calls to other DB functions with their `tx` variant *(yet!)*, however it can still be useful to cut down on duplicated code.
```rust
#[tx_variant]
async fn sample_db_function(id: &Uuid, client: &edgedb_tokio::Client) {
    client.query_required_single_json("", &(id,)).await.unwrap();
    todo!()
}
```
Generates:
```rust
async fn sample_db_function_tx(id: &Uuid, client: &mut edgedb_tokio::Transaction) {
    client.query_required_single_json("", &(id,)).await.unwrap();
    todo!()
}
```
