#[cfg(test)]
mod tests {
    // macro to test different parameters

    use diff_assert::try_diff;
    use test_case::test_case;
    use tree_display::TreeDisplay;
    use tree_display_macros::TreeDisplay;

    #[derive(TreeDisplay)]
    enum TestEnum1 {
        First(usize),
        Second(TestStruct2),
        Third {
            seventh: usize,
            eigthth: usize,
            derp: usize,
        },
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

    pub fn run_test<T: TreeDisplay>(
        test_name: &str,
        data: T,
        show_types: bool,
    ) -> Result<(), String> {
        let expected_file = if show_types {
            format!("../tests/data/{}_typed.txt", test_name)
        } else {
            format!("../tests/data/{}.txt", test_name)
        };
        let actual = if show_types {
            data.tree_print_typed()
        } else {
            data.tree_print()
        };
        let expected = match std::fs::read_to_string(&expected_file) {
            Ok(s) => s.replace('\r', ""),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    eprintln!("Expected output file {expected_file} not found");
                    let diff_file = format!("{expected_file}.actual");
                    eprintln!("Didn't get expected output, writing to {diff_file}",);
                    std::fs::write(diff_file, actual).expect("Unable to write file");
                    return Err(format!("{}", e));
                } else {
                    let err = format!(
                        "Error reading expected output file {}: {}",
                        expected_file, e
                    );
                    return Err(err);
                }
            }
        };

        let diff_file = format!("{expected_file}.actual");
        if let Err(e) = try_diff!(expected, actual) {
            eprintln!("Didn't get expected output, writing to {diff_file}");
            std::fs::write(diff_file, actual).expect("Unable to write file");
            return Err(e);
        } else {
            let _ = std::fs::remove_file(diff_file);
        }

        Ok(())
    }

    #[test_case("all_inclusive/all_inclusive1", all_inclusive)]
    #[test_case("vec/vec_usize", vec_usize)]
    #[test_case("vec/vec_str", vec_str)]
    fn testing<T: TreeDisplay>(test_name: &str, data_func: fn() -> T) {
        let mut to_panic = false;

        if let Err(e) = run_test(test_name, &data_func(), true) {
            eprintln!("{}", e);
            to_panic = true;
        }

        if let Err(e) = run_test(test_name, &data_func(), false) {
            eprintln!("{}", e);
            to_panic = true;
        }

        if to_panic {
            panic!();
        }
    }

    fn all_inclusive() -> TestStruct1<'static, bool> {
        let derp = Box::leak(Box::new(TestStruct5));
        TestStruct1 {
            first: TestStruct2 {
                third: 1,
                fourth: TestStruct3 { fifth: 2, sixth: 3 },
            },
            second: TestStruct3 { fifth: 4, sixth: 5 },
            tenth: TestStruct4(&6, "7".to_string(), true),
            eleventh: TestStruct5,
            derp,
            t: Box::new(true),
            nineth: TestEnum1::Third {
                seventh: 8,
                eigthth: 9,
                derp: 10,
            },
        }
    }

    fn vec_usize() -> Vec<usize> {
        vec![1, 2, 3, 4]
    }

    fn vec_str() -> Vec<&'static str> {
        vec!["abc", "123", "def", "ab2b"]
    }
}
