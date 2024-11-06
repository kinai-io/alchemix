use zbra::prelude::*;

#[entity(index(name), index(rank))]
pub struct User {
    name: String,
    rank: usize,
    data: Vec<u8>,
}

#[test]
pub fn test_entity_flow() {

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

    // println!("{:?}", users);
    
}