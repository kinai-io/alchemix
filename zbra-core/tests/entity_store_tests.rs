use zbra::prelude::*;

#[entity(index(name), index(rank))]
pub struct User {
    name: String,
    rank: usize,
    data: Vec<u8>,
}


#[flow_context(User)]
pub struct AppContext{}

#[tokio::test]
pub async fn test_entity_store() {
    let mut users = vec![];
    for i in 0..10 {
        let user = User::new_with_id(
            &format!("User_{}", i),
            format!("User_{}", i),
            i,
            format!("#{}", i).as_bytes().into(),
        );
        users.push(user);
    }

    let mut datastore = SQLiteEntityStore::new("./test-data/out/test.db");
    let _ = datastore.open().await;

    datastore.clear().await;
    let _ = datastore.update_entities(&users).await;

    let users: Vec<User> = datastore.get_entities_of_kind("User", &vec![]).await;

    assert_eq!(users.len(), 10);

    let ids = vec!["User_2", "User_5"];
    let users: Vec<User> = datastore.get_entities_of_kind("User", &ids).await;
    assert_eq!(users.len(), 2);

    let users: Vec<User> = datastore
        .query_entities("User", "rank", "value >= 3 AND value < 6")
        .await;
    // println!("Users : {:?}", users);
    assert_eq!(users.len(), 3);

    let keys = vec!["User_2", "User_5"];
    datastore.remove_entities::<User>("User", &keys).await;
    datastore.close().await;
}
