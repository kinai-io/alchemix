use std::marker::PhantomData;

pub struct EntitySchema<T> {
    pub name: &'static str,
    pub _marker: PhantomData<T>,
}

