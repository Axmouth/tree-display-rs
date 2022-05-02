#[cfg(test)]
mod tests {
    use diff_assert::try_diff;
    use test_case::test_case;
    use tree_display::TreeDisplay;
    use tree_display_macros::TreeDisplay;

    #[derive(TreeDisplay)]
    //#[tree_display(transparent)]
    enum TestEnum1 {
        First(usize),
        Second(TestStruct2),
        Third {
        #[tree_display(transparent, skip, rename(name = fdf))]
        // #[transparent]
            seventh: usize,
            eigthth: usize,
            derp: usize,
        },
        Fourth,
    }

    #[derive(TreeDisplay)]
    struct TestStruct5;

    #[derive(TreeDisplay)]
    struct TestStruct4<'a, T>(&'a usize, String, T)
    where
        T: TreeDisplay;

    #[derive(TreeDisplay)]
    struct TestStruct3 {
        //#[tree_display(transparent, skip)]
        //#[tree_display(rename)]
        pub fifth: usize,
        #[tree_display(rename = "fdf", skip)]
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

    pub fn run_test<T: TreeDisplay>(
        expected_file: &str,
        data: T,
        show_types: bool,
        dense: bool,
    ) -> Result<(), String> {
        let actual = data.tree_print(show_types, dense);
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

    #[test_case("complex/complex_1", complex_1)]
    #[test_case("vec/vec_usize", vec_usize)]
    #[test_case("vec/vec_str", vec_str)]
    #[test_case("enum/enum_tuple_prim", enum_tuple_prim)]
    #[test_case("enum/enum_nested_struct", enum_nested_struct)]
    #[test_case("enum/enum_named_fields", enum_named_fields)]
    #[test_case("enum/enum_unit", enum_unit)]
    #[test_case("option/option_prim", option_prim)]
    #[test_case("option/option_struct", option_struct)]
    #[test_case("option/option_enum", option_enum)]
    #[test_case("option/option_vec", option_vec)]
    #[test_case("option/option_none", option_none)]
    #[test_case("option/option_vec_none", option_vec_none)]
    #[test_case("tuple/tuple_2", tuple_2)]
    #[test_case("tuple/tuple_3", tuple_3)]
    #[test_case("tuple/tuple_4", tuple_4)]
    #[test_case("tuple/tuple_5", tuple_5)]
    #[test_case("tuple/tuple_6", tuple_6)]
    #[test_case("tuple/tuple_7", tuple_7)]
    #[test_case("tuple/tuple_mixed", tuple_mixed)]
    #[test_case("tuple/tuple_mixed_2", tuple_mixed_2)]
    #[test_case("result/result_ok", result_ok)]
    #[test_case("result/result_ok_enum", result_ok_enum)]
    #[test_case("result/result_ok_struct", result_ok_struct)]
    #[test_case("result/result_ok_vec", result_ok_vec)]
    #[test_case("result/result_err", result_err)]
    #[test_case("result/result_err_enum", result_err_enum)]
    #[test_case("result/result_err_struct", result_err_struct)]
    #[test_case("result/result_err_vec", result_err_vec)]
    fn testing<T: TreeDisplay>(test_name: &str, data_func: fn() -> T) {
        let mut to_panic = false;

        if let Err(e) = run_test(
            &format!("../tests/data/{}_dense.txt", test_name),
            &data_func(),
            false,
            true,
        ) {
            eprintln!("{}", e);
            to_panic = true;
        }

        if let Err(e) = run_test(
            &format!("../tests/data/{}_dense_typed.txt", test_name),
            &data_func(),
            true,
            true,
        ) {
            eprintln!("{}", e);
            to_panic = true;
        }

        if let Err(e) = run_test(
            &format!("../tests/data/{}_typed.txt", test_name),
            &data_func(),
            true,
            false,
        ) {
            eprintln!("{}", e);
            to_panic = true;
        }

        if let Err(e) = run_test(
            &format!("../tests/data/{}.txt", test_name),
            &data_func(),
            false,
            false,
        ) {
            eprintln!("{}", e);
            to_panic = true;
        }

        if to_panic {
            panic!();
        }
    }

    fn complex_1() -> TestStruct1<'static, bool> {
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

    fn enum_tuple_prim() -> TestEnum1 {
        TestEnum1::First(1)
    }

    fn enum_nested_struct() -> TestEnum1 {
        TestEnum1::Second(TestStruct2 {
            third: 1,
            fourth: TestStruct3 { fifth: 2, sixth: 3 },
        })
    }

    fn enum_named_fields() -> TestEnum1 {
        TestEnum1::Third {
            seventh: 1,
            eigthth: 2,
            derp: 3,
        }
    }
    fn enum_unit() -> TestEnum1 {
        TestEnum1::Fourth
    }

    fn option_prim() -> Option<usize> {
        Some(1)
    }

    fn option_struct() -> Option<TestStruct1<'static, bool>> {
        Some(TestStruct1 {
            first: TestStruct2 {
                third: 1,
                fourth: TestStruct3 { fifth: 2, sixth: 3 },
            },
            second: TestStruct3 { fifth: 4, sixth: 5 },
            tenth: TestStruct4(&6, "7".to_string(), true),
            eleventh: TestStruct5,
            derp: Box::leak(Box::new(TestStruct5)),
            t: Box::new(true),
            nineth: TestEnum1::Third {
                seventh: 8,
                eigthth: 9,
                derp: 10,
            },
        })
    }

    fn option_enum() -> Option<TestEnum1> {
        Some(TestEnum1::Third {
            seventh: 1,
            eigthth: 2,
            derp: 3,
        })
    }

    fn option_vec() -> Option<Vec<usize>> {
        Some(vec![1, 2, 3, 4])
    }

    fn option_none() -> Option<usize> {
        None
    }

    fn option_vec_none() -> Option<Vec<usize>> {
        None
    }

    fn tuple_2() -> (usize, usize) {
        (1, 2)
    }

    fn tuple_3() -> (usize, usize, usize) {
        (1, 2, 3)
    }

    fn tuple_4() -> (usize, usize, usize, usize) {
        (1, 2, 3, 4)
    }

    fn tuple_5() -> (usize, usize, usize, usize, usize) {
        (1, 2, 3, 4, 5)
    }

    fn tuple_6() -> (usize, usize, usize, usize, usize, usize) {
        (1, 2, 3, 4, 5, 6)
    }

    fn tuple_7() -> (usize, usize, usize, usize, usize, usize, usize) {
        (1, 2, 3, 4, 5, 6, 7)
    }

    fn tuple_mixed() -> (usize, TestStruct1<'static, bool>, usize) {
        (1, TestStruct1 {
            first: TestStruct2 {
                third: 1,
                fourth: TestStruct3 { fifth: 2, sixth: 3 },
            },
            second: TestStruct3 { fifth: 4, sixth: 5 },
            tenth: TestStruct4(&6, "7".to_string(), true),
            eleventh: TestStruct5,
            derp: Box::leak(Box::new(TestStruct5)),
            t: Box::new(true),
            nineth: TestEnum1::Third {
                seventh: 8,
                eigthth: 9,
                derp: 10,
            },
        }, 2)
    }

    fn tuple_mixed_2() -> (usize, TestStruct1<'static, bool>, usize) {
        (1, TestStruct1 {
            first: TestStruct2 {
                third: 1,
                fourth: TestStruct3 { fifth: 2, sixth: 3 },
            },
            second: TestStruct3 { fifth: 4, sixth: 5 },
            tenth: TestStruct4(&6, "7".to_string(), true),
            eleventh: TestStruct5,
            derp: Box::leak(Box::new(TestStruct5)),
            t: Box::new(true),
            nineth: TestEnum1::Third {
                seventh: 8,
                eigthth: 9,
                derp: 10,
            },
        }, 2)
    }

    fn result_ok() -> Result<usize, usize> {
        Ok(1)
    }

    fn result_err() -> Result<usize, usize> {
        Err(2)
    }

    fn result_ok_struct() -> Result<TestStruct1<'static, bool>, usize> {
        Ok(TestStruct1 {
            first: TestStruct2 {
                third: 1,
                fourth: TestStruct3 { fifth: 2, sixth: 3 },
            },
            second: TestStruct3 { fifth: 4, sixth: 5 },
            tenth: TestStruct4(&6, "7".to_string(), true),
            eleventh: TestStruct5,
            derp: Box::leak(Box::new(TestStruct5)),
            t: Box::new(true),
            nineth: TestEnum1::Third {
                seventh: 8,
                eigthth: 9,
                derp: 10,
            },
        })
    }

    fn result_err_struct() -> Result<String, TestStruct1<'static, bool>> {
        Err(TestStruct1 {
            first: TestStruct2 {
                third: 1,
                fourth: TestStruct3 { fifth: 2, sixth: 3 },
            },
            second: TestStruct3 { fifth: 4, sixth: 5 },
            tenth: TestStruct4(&6, "7".to_string(), true),
            eleventh: TestStruct5,
            derp: Box::leak(Box::new(TestStruct5)),
            t: Box::new(true),
            nineth: TestEnum1::Third {
                seventh: 8,
                eigthth: 9,
                derp: 10,
            },
        })
    }

    fn result_ok_enum() -> Result<TestEnum1, usize> {
        Ok(TestEnum1::Third {
            seventh: 1,
            eigthth: 2,
            derp: 3,
        })
    }

    fn result_err_enum() -> Result<usize, TestEnum1> {
        Err(TestEnum1::Third {
            seventh: 1,
            eigthth: 2,
            derp: 3,
        })
    }

    fn result_ok_vec() -> Result<Vec<usize>, usize> {
        Ok(vec![1, 2, 3, 4])
    }

    fn result_err_vec() -> Result<Vec<String>, Vec<usize>> {
        Err(vec![1, 2, 3, 4])
    }
}
