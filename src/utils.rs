use std::ops::DerefMut;

#[derive(Default)]
pub struct Counter(std::cell::RefCell<u64>);

impl Counter {
    pub fn inc(&self) -> u64 {
        *self.0.borrow_mut().deref_mut() += 1;
        *self.0.borrow_mut()
    }
}

pub mod slice {
    use std::ops::{Deref, Index, DerefMut};
    use crate::{utils::ops::GenRange};

    use super::borrow::{Borrow, BorrowMut};

    pub trait SubSlice {
        type Domain;

        fn sub_slice<Sub: std::ops::RangeBounds<Self::Domain>>(&self, sub: Sub) -> Self;
    }

    pub struct Section<'a, Q, I, T>(Q, GenRange<I>, std::marker::PhantomData<&'a T>)
    where Q: Borrow<[T]>, [T]: Index<GenRange<I>, Output=[T]>;

    impl<'a, Q, I, T>  Borrow<[T]> for Section<'a, Q, I, T> 
    where Q: Borrow<[T]>, [T]: Index<GenRange<I>, Output=[T]>, I: Clone
    {
        type Ref = RefSection<'a, Q::Ref, I, T>;

        fn borrow(&self) -> Self::Ref {
            RefSection::new(self.0.borrow(), self.1.clone())
        }
    }

    impl<'a, Q, I, T> ToOwned for Section<'a, Q, I, T> where Q: ToOwned {
        type Owned = Section<'a, Q::Owned, I, T>;

        fn to_owned(&self) -> Self::Owned {
            Section::new(self.0.to_owned(), self.1.clone(), Default::default())
        }
    }

    impl<'a, Q, I, T>  BorrowMut<[T]> for Section<'a, Q, I, T> 
    where Q: BorrowMut<[T]>, [T]: Index<GenRange<I>, Output=[T]>, I: Clone
    {
        type RefMut = RefSection<'a, Q::RefMut, I, T>;

        fn borrow_mut(&mut self) -> Self::RefMut {
            RefSection::new(self.0.borrow_mut(), self.1.clone())
        }
    }

    pub struct RefSection<'a, R, I, T>(R, GenRange<I>, std::marker::PhantomData<&'a T>) 
    where R: Deref<Target=[T]>, [T]: Index<GenRange<I>, Output=[T]>;

    impl<'a, S, I, T> RefSection<'a, S, I, T> 
    where S: Deref<Target=[T]>, [T]: Index<GenRange<I>, Output=[T]>, I: Clone {
        pub fn new<I2: std::ops::RangeBounds<I>>(src: S, range: I2) -> Self {
            Self(src, (range.start_bound().cloned(), range.end_bound().cloned()), Default::default())
        }
    }

    impl<'a, S, I, T> Deref for RefSection<'a, S, I, T> 
    where S: Deref<Target=[T]>, [T]: Index<GenRange<I>, Output=[T]> {
        type Target = [T];

        fn deref(&self) -> &Self::Target {
            &self.0.deref()[self.1]
        }
    }

    impl<'a, S, I, T> DerefMut for RefSection<'a, S, I, T> 
    where S: DerefMut<Target=[T]>, [T]: Index<GenRange<I>, Output=[T]> {

        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0.deref_mut()[self.1]
        }
    }
}

pub mod borrow {
    use std::ops::{DerefMut, Deref};

    pub struct Ref<'a, T: ?Sized>(&'a T);

    impl<'a, T: ?Sized> From<&'a T> for Ref<'a, T> {
        fn from(a: &'a T) -> Self {
            Self(a)
        }
    }

    impl<'a, T: ?Sized> Deref for Ref<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

    pub struct RefMut<'a, T: ?Sized>(&'a mut T);
    
    impl<'a, T: ?Sized> From<&'a mut T> for RefMut<'a, T> {
        fn from(a: &'a mut T) -> Self {
            Self(a)
        }
    }

    impl<'a, T: ?Sized> Deref for RefMut<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

    impl<'a, T: ?Sized> DerefMut for RefMut<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.0
        }
    }

    pub trait Borrow<T> where T: ?Sized
    {   
        type Ref: Deref<Target=T>;

        fn borrow(&self) -> Self::Ref;
    }
    
    pub trait BorrowMut<T>: Borrow<T> where T: ?Sized
    {
        type RefMut: DerefMut<Target=T>;
    
        fn borrow_mut(&mut self) -> Self::RefMut;
    }    
}

pub mod ops {
    use std::ops::Bound;

    pub type GenRange<T> = (Bound<T>, Bound<T>);

    pub fn subrange<T, R1, R2>(src: R1, dest: R2) -> GenRange<T>
    where R1: std::ops::RangeBounds<T>, 
            R2: std::ops::RangeBounds<T>, 
            T: std::ops::Add<T, Output=T> + std::ops::Sub<T, Output=T> + Clone + Copy
    {
        let start = match (src.start_bound(), dest.start_bound()) {
            (std::ops::Bound::Included(&s1), std::ops::Bound::Included(&s2)) => {
                std::ops::Bound::Included(s1 + s2)
            },
            (std::ops::Bound::Included(&s1), std::ops::Bound::Excluded(&s2)) => {
                std::ops::Bound::Excluded(s1 + s2)
            },
            (std::ops::Bound::Included(&s1), std::ops::Bound::Unbounded) => {
                std::ops::Bound::Included(s1)
            },
            (std::ops::Bound::Excluded(_), std::ops::Bound::Included(&s2)) => {
                std::ops::Bound::Included(s2)
            },
            (std::ops::Bound::Excluded(&s1), std::ops::Bound::Excluded(&s2)) => {
                std::ops::Bound::Excluded(s1 + s2)
            },
            (std::ops::Bound::Excluded(&s1), std::ops::Bound::Unbounded) => {
                std::ops::Bound::Excluded(s1)
            },
            (std::ops::Bound::Unbounded, std::ops::Bound::Included(&s2)) => {
                std::ops::Bound::Included(s2)
            },
            (std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&s2)) => {
                std::ops::Bound::Excluded(s2)
            },
            (std::ops::Bound::Unbounded, std::ops::Bound::Unbounded) => {
                std::ops::Bound::Unbounded
            },
        };
    
        let end = match (src.end_bound(), dest.end_bound()) {
            (Bound::Included(&e1), Bound::Included(&e2)) => {
                Bound::Included(e1 - e2)
            },
            (Bound::Included(&e1), Bound::Excluded(&e2)) => {
                Bound::Excluded(e1 - e2)
            },
            (Bound::Included(&e1), Bound::Unbounded) => {
                Bound::Included(e1)
            },
            (Bound::Excluded(&e1), Bound::Included(&e2)) => {
                Bound::Included(e1 - e2)
            },
            (Bound::Excluded(&e1), Bound::Excluded(&e2)) => {
                Bound::Excluded(e1 - e2)
            },
            (Bound::Excluded(&e1), Bound::Unbounded) => {
                Bound::Excluded(e1)
            },
            (Bound::Unbounded, Bound::Included(&e2)) => {
                match src.start_bound() {
                    Bound::Included(&s1) => {
                        Bound::Included(s1 + e2)
                    },
                    Bound::Excluded(&s1) => {
                        Bound::Excluded(s1 + e2)
                    },
                    Bound::Unbounded => {
                        Bound::Included(e2)
                    },
                }
                
            },
            (Bound::Unbounded, Bound::Excluded(&e2)) => {
                match src.start_bound() {
                    Bound::Included(&s1) => {
                        Bound::Excluded(s1 + e2)
                    },
                    Bound::Excluded(&s1) => {
                        Bound::Excluded(s1 + e2)
                    },
                    Bound::Unbounded => {
                        Bound::Included(e2)
                    },
                }
            },
            (Bound::Unbounded, Bound::Unbounded) => {
                Bound::Unbounded
            },
        };
    
        return (start, end)
    }
}


pub mod traits {
    pub trait ResetableIterator: Iterator {
        fn reset(&mut self);
    }

    pub trait CursorIterator: Iterator {
        fn current(&self) -> Self::Item;
    }
}