// ---------------------------------------------------------------- imports

use crate::config::MAX_ENTITIES;
use crate::ecs::{Components, Entity, Pos, Size};
use std::marker::PhantomData;

// ---------------------------------------------------------------- trait

pub trait QuerySpec {
    type Item<'a>;
    fn fetch<'a>(components: &'a Components, entity: Entity) -> Option<Self::Item<'a>>;
}

impl QuerySpec for (Entity, &Pos) {
    type Item<'a> = (Entity, &'a Pos);
    fn fetch<'a>(components: &'a Components, entity: Entity) -> Option<Self::Item<'a>> {
        let position = components.positions.get(entity)?;
        Some((entity, position))
    }
}

impl QuerySpec for (Entity, &Pos, &Size) {
    type Item<'a> = (Entity, &'a Pos, &'a Size);
    fn fetch<'a>(components: &'a Components, entity: Entity) -> Option<Self::Item<'a>> {
        let position = components.positions.get(entity)?;
        let size = components.sizes.get(entity)?;
        Some((entity, position, size))
    }
}

// ---------------------------------------------------------------- struct
/// example
/// ```
/// let query = Query::<(&Pos,)> {
///     components: &world.data.components,
///     marker: PhantomData,
/// };
/// let result = query.get(Entity::new(0));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Query<'w, Q> {
    components: &'w Components,
    marker: PhantomData<Q>,
}

impl<'w, Q> Query<'w, Q> {
    pub fn new(components: &'w Components) -> Self {
        Self {
            components,
            marker: PhantomData,
        }
    }
    /// example
    /// ```
    /// let query = world.query::<(&Pos,)>();
    /// for (position,) in query.into_iter() {
    ///    ...
    /// }
    /// ```
    pub fn iter<'q>(&'q self) -> QueryIter<'w, 'q, Q> {
        QueryIter {
            query: &self,
            next_entity: 0,
        }
    }
}

impl<'w, Q: QuerySpec> Query<'w, Q> {
    pub fn get(&self, entity: Entity) -> Option<Q::Item<'w>> {
        Q::fetch(self.components, entity)
    }
}

// ---------------------------------------------------------------- iter struct

#[derive(Debug, Clone, Copy)]
pub struct QueryIter<'q, 'w, Q> {
    query: &'q Query<'w, Q>,
    next_entity: usize,
}

impl<'w, 'q, Q> Iterator for QueryIter<'w, 'q, Q>
where
    Q: QuerySpec,
{
    type Item = Q::Item<'w>;
    fn next(&mut self) -> Option<Q::Item<'w>> {
        while self.next_entity < MAX_ENTITIES {
            let entity = Entity::new(self.next_entity);
            self.next_entity += 1;
            if let Some(item) = self.query.get(entity) {
                return Some(item);
            }
        }
        None
    }
}
