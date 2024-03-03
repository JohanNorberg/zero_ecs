pub use itertools::chain;
pub use itertools::izip;
pub use itertools::Itertools;
pub use rayon::prelude::*;
pub use zero_ecs_macros::component;
pub use zero_ecs_macros::entity;
pub use zero_ecs_macros::system;
#[macro_export]
macro_rules! izip_par {
    // @closure creates a tuple-flattening closure for .map() call. usage:
    // @closure partial_pattern => partial_tuple , rest , of , iterators
    // eg. izip!( @closure ((a, b), c) => (a, b, c) , dd , ee )
    ( @closure $p:pat => $tup:expr ) => {
        |$p| $tup
    };

    // The "b" identifier is a different identifier on each recursion level thanks to hygiene.
    ( @closure $p:pat => ( $($tup:tt)* ) , $_iter:expr $( , $tail:expr )* ) => {
        $crate::izip_par!(@closure ($p, b) => ( $($tup)*, b ) $( , $tail )*)
    };

    // unary
    ($first:expr $(,)*) => {
        $crate::IntoParallelIterator::into_par_iter($first)
    };

    // binary
    ($first:expr, $second:expr $(,)*) => {
        $crate::izip_par!($first)
            .zip($second)
    };

    // n-ary where n > 2
    ( $first:expr $( , $rest:expr )* $(,)* ) => {
        $crate::izip_par!($first)
            $(
                .zip($rest)
            )*
            .map(
                $crate::izip!(@closure a => (a) $( , $rest )*)
            )
    };
}
#[macro_export]
macro_rules! chain_par {
    () => {
        rayon::iter::empty()
    };
    ($first:expr $(, $rest:expr )* $(,)?) => {
        {
            let iter = $crate::IntoParallelIterator::into_par_iter($first);
            $(
                let iter =
                    ParallelIterator::chain(
                        iter,
                        $crate::IntoParallelIterator::into_par_iter($rest));
            )*
            iter
        }
    };
}
