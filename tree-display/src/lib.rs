pub use tree_display_macros;

pub trait TreeDisplay {
    fn tree_fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        show_types: bool,
        dense: bool, // Change to enum to show types, names or both? Also variants to rename any combination of the two
                     // flag to use edit friendly characters (better for tests)
                     // dense or sparse
    ) -> std::fmt::Result;

    fn tree_print(&self, show_types: bool, dense: bool) -> String
    where
        Self: Sized,
    {
        DataContainer {
            data: self,
            show_types,
            dense,
        }
        .to_string()
    }

    fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

pub struct DataContainer<T: TreeDisplay> {
    pub data: T,
    pub show_types: bool,
    pub dense: bool,
}

impl<T: TreeDisplay> std::fmt::Display for DataContainer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.tree_fmt(f, "", self.show_types, self.dense)
    }
}

macro_rules! tree_display_impl_primitive {
    ($($t:ty),*) => {
        $(
            impl TreeDisplay for $t {
                fn tree_fmt(&self, f: &mut std::fmt::Formatter<'_>, indent: &str, _: bool, dense: bool) -> std::fmt::Result {
                    writeln!(f, "{}└─{:?}", indent, self)?;
                    if !dense {
                        writeln!(f, "{}", indent)?;
                    }
                    Ok(())
                }

                fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, " ({})", stringify!($t))
                }
            }
        )*
    };
}

tree_display_impl_primitive!(
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    bool,
    char,
    String,
    ()
);

impl TreeDisplay for &str {
    fn tree_fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        _: bool,
        dense: bool,
    ) -> std::fmt::Result {
        writeln!(f, "{}└─{:?}", indent, &self)?;
        if !dense {
            writeln!(f, "{}", indent)?;
        }
        Ok(())
    }

    fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " -> (str)")
    }
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
        dense: bool,
    ) -> std::fmt::Result {
        (**self).tree_fmt(f, indent, show_types, dense)
    }

    fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " ->")?;
        T::type_name_fmt(&self, f)
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
        dense: bool,
    ) -> std::fmt::Result {
        (**self).tree_fmt(f, indent, show_types, dense)
    }

    fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " (Box) ->")?;
        T::type_name_fmt(&self, f)
    }
}

// TODO: show types (?)
impl<T> TreeDisplay for [T]
where
    T: TreeDisplay,
{
    fn tree_fmt<'a>(
        &'a self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        show_types: bool,
        dense: bool,
    ) -> std::fmt::Result {
        if !dense && !self.is_empty() {
            writeln!(f, "{}|", indent)?;
        }
        let mut new_indent = indent.to_string();
        new_indent.push_str("|  ");
        for (i, item) in self.iter().enumerate() {
            if i < self.len() - 1 {
                write!(f, "{}├─[{}]", indent, i)?;
            } else {
                write!(f, "{}└─[{}]", indent, i)?;
                new_indent = indent.to_string();
                new_indent.push_str("   ");
            }
            if show_types {
                item.type_name_fmt(f)?;
            }
            writeln!(f)?;
            item.tree_fmt(f, &new_indent, show_types, dense)?;
        }
        if !dense && self.is_empty() {
            writeln!(f, "{}", indent)?;
        }
        Ok(())
    }

    fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " (Array)")
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
        dense: bool,
    ) -> std::fmt::Result {
        self[..].tree_fmt(f, indent, show_types, dense)
    }

    fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " (Vec)")
    }
}

// TODO: Do maps, sets, vectors, arrays etc.
// TODO: Do for references too and test
// TODO: Indication for references/pointers?
// TODO: Do Box and such
// TODO: Do Result/Option
// TODO: Serde based version too (?)

// TODO: Enum tuple, indicate enum ?

impl<T> TreeDisplay for Option<T>
where
    T: TreeDisplay,
{
    fn tree_fmt<'a>(
        &'a self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        show_types: bool,
        dense: bool,
    ) -> std::fmt::Result {
        let mut new_indent = indent.to_string();
        new_indent.push_str("   ");
        if !dense {
            writeln!(f, "{}|", new_indent)?;
        }
        write!(f, "{}└─", new_indent)?;
        if let Some(item) = self {
            writeln!(f, "Some")?;
            new_indent.push_str("   ");
            item.tree_fmt(f, &new_indent, show_types, dense)?;
        } else {
            writeln!(f, "None")?;
            if !dense {
                writeln!(f, "{}", new_indent)?;
            }
        }
        Ok(())
    }

    fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " (Option)")
    }
}

impl<T, E> TreeDisplay for Result<T, E>
where
    T: TreeDisplay,
    E: TreeDisplay,
{
    fn tree_fmt<'a>(
        &'a self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        show_types: bool,
        dense: bool,
    ) -> std::fmt::Result {
        let mut new_indent = indent.to_string();
        new_indent.push_str("   ");
        if !dense {
            writeln!(f, "{}|", new_indent)?;
        }
        write!(f, "{}└─", new_indent)?;
        if let Ok(item) = self {
            writeln!(f, "Ok")?;
            new_indent.push_str("   ");
            item.tree_fmt(f, &new_indent, show_types, dense)?;
        } else if let Err(item) = self {
            writeln!(f, "Err")?;
            new_indent.push_str("   ");
            item.tree_fmt(f, &new_indent, show_types, dense)?;
        }
        Ok(())
    }

    fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " (Result)")
    }
}

macro_rules! tree_display_impl_tuple_primitive {
    ( $($typ:ident),* $(,)? ) => {
            #[allow(non_snake_case)]
            impl<T, $($typ,)*> TreeDisplay for (T, $($typ,)* ) where
                T: TreeDisplay,
                $( $typ: TreeDisplay,)* {
                fn tree_fmt(&self, f: &mut std::fmt::Formatter<'_>, indent: &str, show_types: bool, dense: bool) -> std::fmt::Result {
                    let (t, $($typ,)*) = self;
                    #[allow(unused_mut)]
                    let mut i = 0;
                    let mut new_indent = indent.to_string();
                    new_indent.push_str("|  ");
                    if !dense {
                        writeln!(f, "{}|", indent)?;
                    }
                    $(
                        write!(f, "{}├──{}", indent, i)?;
                        if show_types {
                            $typ::type_name_fmt(&$typ, f)?;
                        };
                        writeln!(f)?;
                        $typ.tree_fmt(f, &new_indent, show_types, dense)?;
                        i += 1;
                    )*
                    write!(f, "{}└──{}", indent, i)?;
                    if show_types {
                        t.type_name_fmt(f)?;
                    };
                    writeln!(f)?;
                    t.tree_fmt(f, &format!("   {}", indent), show_types, dense)?;
                    Ok(())
                }

                fn type_name_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, " (Tuple)")
                }
            }
    };
}

tree_display_impl_tuple_primitive!();
tree_display_impl_tuple_primitive!(T1);
tree_display_impl_tuple_primitive!(T1, T2);
tree_display_impl_tuple_primitive!(T1, T2, T3);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4, T5);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4, T5, T6);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4, T5, T6, T7);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4, T5, T6, T7, T8);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
tree_display_impl_tuple_primitive!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31
);
tree_display_impl_tuple_primitive!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);
