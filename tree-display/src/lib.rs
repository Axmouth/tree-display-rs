pub use tree_display_macros;

pub trait TreeDisplay {
    fn tree_fmt<'a>(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        show_types: bool, // Change to enum to show types, names or both? Also variants to rename any combination of the two
                          // flag to use edit friendly characters (better for tests)
    ) -> std::fmt::Result;
}

macro_rules! tree_display_impl_primitive {
    ($($t:ty),*) => {
        $(
            impl TreeDisplay for $t {
                fn tree_fmt(&self, f: &mut std::fmt::Formatter<'_>, indent: &str, show_types: bool) -> std::fmt::Result {
                    if show_types {
                        writeln!(f, "({})\n{}└─{}\n{}", stringify!($t), indent, self, indent)
                    } else {
                        writeln!(f, "\n{}└─{}\n{}", indent, self, indent)
                    }
                }
            }
        )*
    };
}

impl<T> TreeDisplay for &T
where
    T: TreeDisplay,
{
    fn tree_fmt<'a>(
        &'a self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        show_types: bool,
    ) -> std::fmt::Result {
        (**self).tree_fmt(f, indent, show_types)
    }
}

impl<T> TreeDisplay for Box<T>
where
    T: TreeDisplay,
{
    fn tree_fmt<'a>(
        &'a self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        show_types: bool,
    ) -> std::fmt::Result {
        (**self).tree_fmt(f, indent, show_types)
    }
}


impl<T> TreeDisplay for [T]
where
    T: TreeDisplay,
{
    fn tree_fmt<'a>(
        &'a self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        show_types: bool,
    ) -> std::fmt::Result {
        if show_types {
            writeln!(f, "({})", "Vec")?;
        }
        let mut new_indent = indent.to_string();
        new_indent.push_str("|  ");
        for (i, item) in self.iter().enumerate() {
            if i < self.len() - 1 {
                write!(f, "{}├─", indent)?;
            } else {
                write!(f, "{}└─", indent)?;
            }
            item.tree_fmt(f, &format!("{}  ", indent), show_types)?;
        }
        Ok(())
    }
}

impl<T> TreeDisplay for Vec<T>
where
    T: TreeDisplay,
{
    fn tree_fmt<'a>(
        &'a self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        show_types: bool,
    ) -> std::fmt::Result {
        self[..].tree_fmt(f, indent, show_types)
    }
}

impl TreeDisplay for () {
    fn tree_fmt<'a>(
        &'a self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        _: bool,
    ) -> std::fmt::Result {
        writeln!(f, "({})\n{}└─", stringify!(()), indent)
    }
}

tree_display_impl_primitive!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, bool, char, &str,
    String
);

// Do maps, sets, vectors, arrays etc.
// Do for references too and test
// Do Box and such

#[cfg(test)]
mod tests {
    use crate::TreeDisplay;
    use pretty_assertions::assert_eq;
    use tree_display_macros::TreeDisplay;

    #[derive(TreeDisplay)]
    enum TestEnum1 {
        First(usize),
        Second(TestStruct2),
        Third { seventh: usize, eigthth: usize },
    }

    #[derive(TreeDisplay)]
    struct TestStruct5;

    #[derive(TreeDisplay)]
    struct TestStruct4<'a, T>(&'a usize, String, T)
    where
        T: TreeDisplay;

    #[derive(TreeDisplay)]
    struct TestStruct3 {
        pub fifth: usize,
        pub sixth: usize,
    }

    #[derive(TreeDisplay)]
    struct TestStruct2 {
        pub third: usize,
        pub fourth: TestStruct3,
    }

    #[derive(TreeDisplay)]
    struct TestStruct1<'a, T>
    where
        T: TreeDisplay,
    {
        pub first: TestStruct2,
        pub second: TestStruct3,
        pub tenth: TestStruct4<'a, T>,
        pub eleventh: TestStruct5,
        pub derp: &'a TestStruct5,
        pub t: Box<T>,
        pub nineth: TestEnum1,
    }

    impl<'a, T: TreeDisplay> std::fmt::Display for TestStruct1<'a, T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.tree_fmt(f, "", false)
        }
    }

    #[test]
    fn test_struct_1_no_types() {
        let expected = "
|
├──first 
|  |
|  ├──third 
|  |  └─1
|  |  
|  └──fourth 
|     |
|     ├──fifth 
|     |  └─2
|     |  
|     └──sixth 
|        └─3
|        
└──second 
   |
   ├──fifth 
   |  └─4
   |  
   └──sixth 
      └─5
      
";

        let derp = TestStruct5;
        let data = TestStruct1 {
            first: TestStruct2 {
                third: 1,
                fourth: TestStruct3 { fifth: 2, sixth: 3 },
            },
            second: TestStruct3 { fifth: 4, sixth: 5 },
            tenth: TestStruct4(&6, "7".to_string(), true),
            eleventh: TestStruct5,
            derp: &derp,
            t: Box::new(true),
            nineth: TestEnum1::First(6),
        };
        let actual = data.to_string();
        eprintln!("{}", data);
        assert_eq!(expected, actual);
    }
}
