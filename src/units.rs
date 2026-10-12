use pastey::paste;

pub const PI: f64 = 3.141592653589793;

pub const MM_PER_INCH: f64 = 25.4;
pub const DEG_PER_RAD: f64 = 360.0 / (2.0 * PI);

mod private {
    pub trait Sealed {}
}

macro_rules! number_types {
    ($callback:ident) => {
        $callback!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
    };
}

macro_rules! from_decls {
    ($($ty:ident),+) => {
        $(paste! {
            #[doc(hidden)]
            fn [<from_ $ty>](v: $ty) -> Self;
        })+
    };
}

macro_rules! from_fns {
    ($($ty:ident),+) => {
        $(paste! {
            #[doc(hidden)]
            fn [<from_ $ty>](v: $ty) -> Self {
                v as Self
            }
        })+
    };
}

pub trait Number: private::Sealed + Sized {
    fn cast<T: Number>(self) -> T;

    number_types!(from_decls);
}

macro_rules! impl_number {
    ($($ty:ty),+) => {
        $(
            impl private::Sealed for $ty {}

            impl Number for $ty {
                fn cast<T: Number>(self) -> T {
                    paste! {
                        T::[<from_ $ty>](self)
                    }
                }

                number_types!(from_fns);
            }
        )+
    };
}

number_types!(impl_number);

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Distance(f64);

impl Distance {
    pub fn mm(&self) -> f64 {
        self.0
    }

    pub fn inches(&self) -> f64 {
        self.0 / MM_PER_INCH
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Angle(f64);

impl Angle {
    pub fn deg(&self) -> f64 {
        self.0
    }

    pub fn rad(&self) -> f64 {
        self.0 / DEG_PER_RAD
    }
}

pub trait Units: private::Sealed {
    fn mm(self) -> Distance;
    fn inches(self) -> Distance;
    fn deg(self) -> Angle;
    fn rad(self) -> Angle;
}

impl<T: Number> Units for T {
    fn mm(self) -> Distance {
        Distance(self.cast())
    }

    fn inches(self) -> Distance {
        Distance(self.cast::<f64>() * MM_PER_INCH)
    }

    fn deg(self) -> Angle {
        Angle(self.cast())
    }

    fn rad(self) -> Angle {
        Angle(self.cast::<f64>() * DEG_PER_RAD)
    }
}
