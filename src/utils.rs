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

    use super::borrow::{Borrow, BorrowMut, ToOwned};

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

    impl<'a, Q, I, T>  Borrow<[T]> for Section<'a, Q, I, T> 
    where Q: Borrow<[T]>, [T]: Index<GenRange<I>, Output=[T]>, I: Clone
    {
        type Ref = Section<'a, Q::Ref, I, T>;

        fn borrow(&self) -> Self::Ref {
            Section::new(self.0.borrow(), self.1.clone())
        }
    }

    impl<'a, Q, I, T> ToOwned for Section<'a, Q, I, T> 
    where Q: ToOwned, [T]: Index<GenRange<I>, Output=[T]>, I: Clone {
        type Owned = Section<'a, Q::Owned, I, T>;

        fn to_owned(&self) -> Self::Owned {
            Section(self.0.to_owned(), self.1.clone(), Default::default())
        }
    }

    impl<'a, Q, I, T>  BorrowMut<[T]> for Section<'a, Q, I, T> 
    where Q: BorrowMut<[T]>, [T]: Index<GenRange<I>, Output=[T]>, I: Clone
    {
        type RefMut = Section<'a, Q::RefMut, I, T>;

        fn borrow_mut(&mut self) -> Self::RefMut {
            Section::new(self.0.borrow_mut(), self.1.clone())
        }
    }

    impl<'a, S, I, T> Deref for Section<'a, S, I, T> 
    where S: Deref<Target=[T]>, [T]: Index<GenRange<I>, Output=[T]> {
        type Target = [T];

        fn deref(&self) -> &Self::Target {
            &self.0.deref()[self.1]
        }
    }

    impl<'a, S, I, T> DerefMut for Section<'a, S, I, T> 
    where S: DerefMut<Target=[T]>, [T]: Index<GenRange<I>, Output=[T]> {

        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0.deref_mut()[self.1]
        }
    }

}

pub mod borrow {
    use std::ops::{DerefMut, Deref};

    pub trait Borrow<T> where T: ?Sized
    {   
        type Ref: Deref<Target=T>;

        fn borrow(&self) -> Self::Ref;
    }

    impl<'a, T: ?Sized, R: ?Sized> Borrow<R> for &'a T where T: Borrow<R> {
        type Ref = T::Ref;

        fn borrow(&self) -> Self::Ref {
            self.borrow()
        }
    } 

    pub struct Ref<'a, T>(T, std::marker::PhantomData<&'a()>);

    impl<'a, T: ?Sized> From<&'a T> for Ref<'a, &'a T> {
        fn from(value: &'a T) -> Self {
            Self(value, Default::default())
        }
    }

    impl<'a, T: ?Sized> From<&'a mut T> for Ref<'a, &'a mut T> {
        fn from(value: &'a mut T) -> Self {
            Self(value, Default::default())
        }
    }

    impl<'a, T: ?Sized> Borrow<T> for Ref<'a, &'a T> {
        type Ref = &'a T;

        fn borrow(&self) -> Self::Ref {
            self.0
        }
    }

    impl<'a, T: ?Sized> Borrow<T> for Ref<'a, &'a mut T> {
        type Ref = &'a T;

        fn borrow(&self) -> Self::Ref {
            self.0
        }
    }


    impl<'a, T: ?Sized> BorrowMut<T> for Ref<'a, &'a mut T> {
        type RefMut = &'a mut T;

        fn borrow_mut(&mut self) -> Self::RefMut {
            self.0
        }
    }
    
    pub trait BorrowMut<T>: Borrow<T> where T: ?Sized
    {
        type RefMut: DerefMut<Target=T>;
    
        fn borrow_mut(&mut self) -> Self::RefMut;
    }
    
    impl<'a, T: ?Sized, R: ?Sized> Borrow<R> for &'a mut T where T: BorrowMut<R> {
        type Ref = T::Ref;

        fn borrow(&self) -> Self::Ref {
            self.borrow()
        }
    } 
    impl<'a, T: ?Sized, R: ?Sized> BorrowMut<R> for &'a mut T where T: BorrowMut<R> {
        type RefMut = T::RefMut;

        fn borrow_mut(&mut self) -> Self::RefMut {
            self.borrow_mut()
        }
    } 
    
    pub trait ToOwned {
        type Owned;

        fn to_owned(&self) -> Self::Owned;
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