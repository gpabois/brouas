use std::ops::DerefMut;

#[derive(Default)]
pub struct Counter<Id>(std::cell::RefCell<Id>);

impl<Id: std::ops::AddAssign<u8>> Counter<Id> {
    pub fn inc(&self) -> u64 {
        *self.0.borrow_mut().deref_mut() += 1;
        *self.0.borrow_mut()
    }
}

pub mod cell {
    pub trait TryCell {
            type Error;
            type Ref; 
            type RefMut;

        fn try_borrow(&self) -> std::result::Result<Self::Ref, Self::Error>;
        fn try_borrow_mut(&mut self) -> std::result::Result<Self::RefMut, Self::Error>;
    }
}

pub mod slice {
    use std::ops::{Index, IndexMut, Deref, DerefMut};
    use crate::{utils::ops::GenRange};

    use super::borrow::{TryBorrow, TryBorrowMut};

    pub trait BorrowSection<'a, S> {
        type Cursor;

        fn borrow_section(&'a self, cursor: Self::Cursor) -> S; 
    }

    pub trait BorrowMutSection<'a, S> {
        type Cursor;

        fn borrow_mut_section(&'a mut self, cursor: Self::Cursor) -> S; 
    }

    pub trait CloneSection<S> {
        type Cursor;

        fn clone_section(&self, cursor: Self::Cursor) -> S; 
    }

    pub trait IntoSection<S> {
        type Cursor;

        fn into_section(self, cursor: Self::Cursor) -> S; 
    }

    pub trait SubSlice {
        type Domain;

        fn sub_slice<Sub: std::ops::RangeBounds<Self::Domain>>(&self, sub: Sub) -> Self;
    }

    pub struct Section<'a, Q, I, T>(Q, GenRange<I>, std::marker::PhantomData<&'a T>) where [T]: Index<GenRange<I>, Output=[T]>;

    impl<'a, Q, I, T> Section<'a, Q, I, T> where [T]: Index<GenRange<I>, Output=[T]>, I: Clone {
        pub fn new<I2: std::ops::RangeBounds<I>>(src: Q, range: I2) -> Self {
            Self(src, (range.start_bound().cloned(), range.end_bound().cloned()), Default::default())
        }
    }

    impl<'a, Q, I, T>  TryBorrow<'a, [T]> for Section<'a, Q, I, T> 
    where Q: TryBorrow<'a, [T]>, [T]: Index<GenRange<I>, Output=[T]>, I: Clone
    {
        type Ref = Section<'a, Q::Ref, I, T>;
        type Error = Q::Error;

        fn try_borrow(&self) -> std::result::Result<Self::Ref, Self::Error> {
            Ok(Section::new(self.0.try_borrow()?, self.1.clone()))
        }
    }

    impl<'a, Q, I, T>  TryBorrowMut<'a, [T]> for Section<'a, Q, I, T> 
    where Q: TryBorrowMut<'a, [T]>, [T]: IndexMut<GenRange<I>, Output=[T]>, I: Clone
    {
        type RefMut = Section<'a, Q::RefMut, I, T>;

        fn try_borrow_mut(&mut self) -> std::result::Result<Self::RefMut, Self::Error> {
            Ok(Section::new(self.0.try_borrow_mut()?, self.1.clone()))
        }
    }

    impl<'a, Q, I, T> Clone for Section<'a, Q, I, T> 
    where Q: Clone, [T]: Index<GenRange<I>, Output=[T]>, I: Clone {
        fn clone(&self) -> Self {
            Section(self.0.clone(), self.1.clone(), Default::default())
        }
    }

    impl<'a, S, I, T> AsRef<[T]> for Section<'a, S, I, T> 
    where S: AsRef<[T]>, [T]: Index<GenRange<I>, Output=[T]>, I: Clone {

        fn as_ref(&self) -> &[T] {
            &self.0.as_ref()[self.1.clone()]
        }
    }

    impl<'a, S, I, T> Deref for Section<'a, S, I, T> 
    where S: AsRef<[T]>, [T]: IndexMut<GenRange<I>, Output=[T]>, I: Clone {
        type Target = [T];

        fn deref(&self) -> &Self::Target {
            &self.0.as_ref()[self.1.clone()]
        }
    }

    impl<'a, S, I, T> DerefMut for Section<'a, S, I, T> 
    where S: AsRef<[T]> + AsMut<[T]>, [T]: IndexMut<GenRange<I>, Output=[T]>, I: Clone {

        fn deref_mut(&mut self) -> &mut Self::Target {
            self.as_mut()
        }
    }

    impl<'a, S, I, T> AsMut<[T]> for Section<'a, S, I, T> 
    where S: AsMut<[T]>, [T]: IndexMut<GenRange<I>, Output=[T]>, I: Clone
    {
        fn as_mut(&mut self) -> &mut [T] 
        {
            &mut self.0.as_mut()[self.1.clone()]
        }
    }

}

pub mod borrow {
    use std::{ops::{DerefMut, Deref}, borrow::{Borrow, BorrowMut}};
    pub trait TryBorrow<'a, T> where T: ?Sized {
        type Ref: AsRef<T>;
        type Error;

        fn try_borrow(&self) -> std::result::Result<Self::Ref, Self::Error>;
    }

    pub trait TryBorrowMut<'a, T>: TryBorrow<'a, T> where T: ?Sized {
        type RefMut: AsMut<T> + AsRef<T>;

        fn try_borrow_mut(&mut self) -> std::result::Result<Self::RefMut, Self::Error>;
    }

    pub trait RefBorrow<'a, T> where T: ?Sized
    {   
        type Ref: Deref<Target=T>;

        fn borrow_ref(&'a self) -> Self::Ref;
    }
       
    pub trait RefBorrowMut<'a, T>: RefBorrow<'a, T> where T: ?Sized
    {
        type RefMut: DerefMut<Target=T>;
    
        fn borrow_mut_ref(&'a mut self) -> Self::RefMut;
    }

    impl<'a, T: 'a, Q> RefBorrow<'a, T> for Q where Q: Borrow<T> {
        type Ref = &'a T;

        fn borrow_ref(&'a self) -> Self::Ref {
            self.borrow()
        }
    }

    impl<'a, T: 'a, Q> RefBorrowMut<'a, T> for Q where Q: BorrowMut<T> {
        type RefMut = &'a mut T;

        fn borrow_mut_ref(&'a mut self) -> Self::RefMut {
            self.borrow_mut()
        }
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