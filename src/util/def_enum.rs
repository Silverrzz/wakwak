#[macro_export]
macro_rules! def_enum {
    (
        $(#[$($enum_attr:meta)+])*
        $vis:vis enum $name:ident : $int_ty:ty {
            $(
                $(#[$($var_attr:meta)+])*
                $var:ident
            ),+
        }
    ) => {
        $(#[$($enum_attr)+])*
        #[repr($int_ty)]
        $vis enum $name {
            $(
                $(#[$($var_attr)+])*
                $var
            ),+
        }

        impl $name {
            /// # Safety
            /// `0 <= i < Self::COUNT` must hold.
            #[inline]
            $vis const unsafe fn index_unchecked(i: $int_ty) -> Self {
                // SAFETY: It is implied by the precondition that `i` is a valid discriminant.
                unsafe { ::core::mem::transmute(i) }
            }

            #[inline]
            $vis const fn index(i: usize) -> Self {
                Self::try_index(i).expect(concat!(stringify!($name), "::index(i) Index out of bounds"))
            }

            #[inline]
            $vis const fn try_index(i: usize) -> Option<Self> {
                if i < Self::COUNT {
                    return Some(unsafe { Self::index_unchecked(i as $int_ty) });
                }

                None
            }

            $vis const COUNT: usize = <[Self]>::len(&[$(Self::$var),+]);
            $vis const ALL: &[Self; Self::COUNT] = &[$(Self::$var),+];
        }

        impl<T> ::core::ops::Index<$name> for [T; $name::COUNT] {
            type Output = T;

            #[inline]
            fn index(&self, index: $name) -> &Self::Output {
                self.index(index as usize)
            }
        }

        impl<T> ::core::ops::IndexMut<$name> for [T; $name::COUNT] {
            #[inline]
            fn index_mut(&mut self, index: $name) -> &mut Self::Output {
                self.index_mut(index as usize)
            }
        }
    }
}
