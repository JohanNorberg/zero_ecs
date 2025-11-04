use proc_macro2::TokenStream;
use quote::quote;
pub fn get_default_queries() -> TokenStream {
    quote! {
        #[derive(Default, Debug)]
        pub struct Query<T> {
            phantom: PhantomData<T>,
        }

        impl<T> Copy for Query<T> {}
        impl<T> Clone for Query<T> {
            fn clone(&self) -> Self { Query { phantom: PhantomData, } }
        }

        #[allow(dead_code)]
        impl<T> Query<T> {
            pub fn new() -> Query<T> {
                Query {
                    phantom: PhantomData,
                }
            }
        }
        pub trait LenFrom<'a, T>
        where
            T: 'a + Send,
        {
            fn len(&'a self) -> usize;
            fn is_empty(&'a self) -> bool {
                self.len() == 0
            }
        }

        pub trait QueryFrom<'a, T>
        where
            T: 'a + Send,
        {
            fn query_from(&'a self) -> impl Iterator<Item = T>;
            fn par_query_from(&'a self) -> impl ParallelIterator<Item = T>;
            fn get_from(&'a self, entity: Entity) -> Option<T>;
            fn at(&'a self, index: usize) -> Option<T>;
        }
        pub trait QueryMutFrom<'a, T>
        where
            T: 'a + Send,
        {
            fn query_mut_from(&'a mut self) -> impl Iterator<Item = T>;
            fn par_query_mut_from(&'a mut self) -> impl ParallelIterator<Item = T>;
            fn get_mut_from(&'a mut self, entity: Entity) -> Option<T>;
            fn at_mut(&'a mut self, index: usize) -> Option<T>;
        }

        impl<'a, T: 'a + Send> Query<T>
        {
            pub fn iter(&self, world: &'a World) -> impl Iterator<Item = T> + 'a
            where
                World: QueryFrom<'a, T>,
            {
                world.query_from()
            }
        }
        impl<'a, T: 'a + Send> Query<T>
        {
            pub fn par_iter(&self, world: &'a World) -> impl ParallelIterator<Item = T> + 'a
            where
                World: QueryFrom<'a, T>,
            {
                world.par_query_from()
            }
        }
        impl<'a, T: 'a + Send> Query<T> {
            pub fn iter_mut(&self, world: &'a mut World) -> impl Iterator<Item = T> + 'a
            where
                World: QueryMutFrom<'a, T>,
            {
                world.query_mut_from()
            }
        }
        impl<'a, T: 'a + Send> Query<T>
        {
            pub fn par_iter_mut(&self, world: &'a mut World) -> impl ParallelIterator<Item = T> + 'a
            where
                World: QueryMutFrom<'a, T>,
            {
                world.par_query_mut_from()
            }
        }
        impl<'a, T: 'a + Send> Query<T> {
            pub fn get(&self, world: &'a World, entity: Entity) -> Option<T>
            where
                World: QueryFrom<'a, T>,
            {
                world.get_from(entity)
            }
        }
        impl<'a, T: 'a + Send> Query<T> {
            pub fn get_mut(&self, world: &'a mut World, entity: Entity) -> Option<T>
            where
                World: QueryMutFrom<'a, T>,
            {
                world.get_mut_from(entity)
            }
        }

        // implement len
        impl<'a, T: 'a + Send> Query<T> {
            pub fn len(&self, world: &'a World) -> usize
            where
                World: LenFrom<'a, T>,
            {
                world.len()
            }
        }

        // impl at_mut
        impl<'a, T: 'a + Send> Query<T> {
            pub fn at_mut(&self, world: &'a mut World, index: usize) -> Option<T>
            where
                World: QueryMutFrom<'a, T>,
            {
                world.at_mut(index)
            }
        }

        // impl at
        impl<'a, T: 'a + Send> Query<T> {
            pub fn at(&self, world: &'a World, index: usize) -> Option<T>
            where
                World: QueryFrom<'a, T>,
            {
                world.at(index)
            }
        }


        pub struct WithQueryMut<'a, T> {
            query: Query<T>,
            world: &'a mut World,
        }
        pub struct WithQuery<'a, T> {
            query: Query<T>,
            world: &'a World,
        }

        #[allow(dead_code)]
        impl<'a, T> WithQueryMut<'a, T>
            where World: QueryMutFrom<'a, T>,
                World: LenFrom<'a, T>,
                T: 'a + Send,
        {
            pub fn iter_mut<U>(&'a mut self) -> impl Iterator<Item = U> + 'a
                where T: Into<U>
            {
                self.query.iter_mut(self.world).map(|e|e.into())
            }
            pub fn par_iter_mut<U>(&'a mut self) -> impl ParallelIterator<Item = U> + 'a
                where T: Into<U>, U: Send
            {
                self.query.par_iter_mut(self.world).map(|e|e.into())
            }
            pub fn get_mut<U>(&'a mut self, entity: Entity) -> Option<U>
                where T: Into<U>,
            {
                self.query.get_mut(self.world, entity).map(|e| e.into())
            }

            pub fn len(&'a mut self) -> usize {
                self.query.len(self.world)
            }

            pub fn at_mut<U>(&'a mut self, index: usize) -> Option<U>
                where T: Into<U>,
            {
                self.query.at_mut(self.world, index).map(|e| e.into())
            }

            pub fn is_empty(&'a mut self) -> bool {
                self.query.len(self.world) == 0
            }
        }

        #[allow(dead_code)]
        impl<'a, T> WithQuery<'a, T>
            where World: QueryFrom<'a, T>,
                World: LenFrom<'a, T>,
                T: 'a + Send,
        {
            pub fn iter<U>(&'a self) -> impl Iterator<Item = U> + 'a
                where T: Into<U>, U: Send
            {
                self.query.iter(self.world).map(|e|e.into())
            }
            pub fn par_iter<U>(&'a self) -> impl ParallelIterator<Item = U> + 'a
                where T: Into<U>, U: Send
            {
                self.query.par_iter(self.world).map(|e|e.into())
            }
            pub fn get<U>(&'a self, entity: Entity) -> Option<U>
                where T: Into<U>, U: Send
            {
                self.query.get(self.world, entity).map(|e|e.into())
            }
            pub fn len(&'a self) -> usize {
                self.query.len(self.world)
            }
            pub fn at<U>(&'a self, index: usize) -> Option<U>
                where T: Into<U>, U: Send
            {
                self.query.at(self.world, index).map(|e|e.into())
            }
            pub fn is_empty(&'a self) -> bool {
                self.query.len(self.world) == 0
            }
        }

        #[allow(dead_code)]
        impl World {
            pub fn with_query_mut<'a, T: 'a + Send>(&'a mut self, query: Query<T>) -> WithQueryMut<'a, T>
            where
                World: QueryMutFrom<'a, T>,
            {
                WithQueryMut {
                    query,
                    world: self,
                }
            }
        }
        #[allow(dead_code)]
        impl World {
            pub fn with_query<'a, T: 'a + Send>(&'a self, query: Query<T>) -> WithQuery<'a, T>
            where
                World: QueryFrom<'a, T>,
            {
                WithQuery {
                    query,
                    world: self,
                }
            }
        }


    }
}
