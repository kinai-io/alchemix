use zbra::prelude::*;

#[entity(index(name))]
pub struct User {
    name: String,
    value: usize,
    data: Vec<u8>,
}

#[tokio::main]
async fn main() {
    let mut users = vec![];
    for i in 0..10 {
        let user = User::new_with_id(
            &format!("User #{}", i),
            format!("User #{}", i),
            i,
            format!("#{}", i).as_bytes().into(),
        );
        // let user = User::new(format!("User #{}", i), i, format!("#{}", i).as_bytes().into());
        users.push(user);
    }

    let datastore = SQLiteEntityStore::new("./test-data/out/test.db")
        .open()
        .await;

    if let Ok(datastore) = datastore {
        let _ = datastore.update_entities(&users).await;

        let keys = vec!["User#User #2", "User#User #5"];
        datastore.remove_entities(&keys).await;
        datastore.close().await;

    }
}