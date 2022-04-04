pub use tree_display_macros;

pub trait TreeDisplay {
    fn tree_fmt<'a>(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        show_types: bool, // Change to enum to show types, names or both? Also variants to rename any combination of the two
                          // flag to use edit friendly characters (better for tests)
                          // dense or sparse
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
// Do Result/Option
