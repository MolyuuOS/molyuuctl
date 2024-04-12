macro_rules! generate_error_enum {
    ($enum_name:ident, {$($variant:ident: $description:expr),+,}) => {
        #[derive(Debug, Eq, PartialEq)]
        pub enum $enum_name {
            $($variant,)+
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.get_description())
            }
        }

        impl std::error::Error for $enum_name {}

        impl $enum_name {
            pub fn get_description(&self) -> &'static str {
                match self {
                    $(
                        $enum_name::$variant => $description,
                    )+
                }
            }
        }
    };
}

pub(crate) use generate_error_enum;