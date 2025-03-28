use ethereum_types::H256;
use ssz::{Decode, DecodeError, Encode};
use ssz_derive::{Decode, Encode};

mod round_trip {
    use super::*;

    fn round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(items: Vec<T>) {
        for item in items {
            let encoded = &item.as_ssz_bytes();
            assert_eq!(T::from_ssz_bytes(&encoded), Ok(item));
        }
    }

    #[test]
    fn bool() {
        let items: Vec<bool> = vec![true, false];

        round_trip(items);
    }

    #[test]
    fn u8_array_4() {
        let items: Vec<[u8; 4]> = vec![[0, 0, 0, 0], [1, 0, 0, 0], [1, 2, 3, 4], [1, 2, 0, 4]];

        round_trip(items);
    }

    #[test]
    fn h256() {
        let items: Vec<H256> = vec![H256::zero(), H256::from([1; 32]), H256::random()];

        round_trip(items);
    }

    #[test]
    fn vec_of_h256() {
        let items: Vec<Vec<H256>> = vec![
            vec![],
            vec![H256::zero(), H256::from([1; 32]), H256::random()],
        ];

        round_trip(items);
    }

    #[test]
    fn vec_u16() {
        let items: Vec<Vec<u16>> = vec![
            vec![],
            vec![255],
            vec![0, 1, 2],
            vec![100; 64],
            vec![255, 0, 255],
        ];

        round_trip(items);
    }

    #[test]
    fn vec_of_vec_u16() {
        let items: Vec<Vec<Vec<u16>>> = vec![
            vec![],
            vec![vec![]],
            vec![vec![1, 2, 3]],
            vec![vec![], vec![]],
            vec![vec![], vec![1, 2, 3]],
            vec![vec![1, 2, 3], vec![1, 2, 3]],
            vec![vec![1, 2, 3], vec![], vec![1, 2, 3]],
            vec![vec![], vec![], vec![1, 2, 3]],
            vec![vec![], vec![1], vec![1, 2, 3]],
            vec![vec![], vec![1], vec![1, 2, 3]],
        ];

        round_trip(items);
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    struct FixedLen {
        a: u16,
        b: u64,
        c: u32,
    }

    #[test]
    fn fixed_len_struct_encoding() {
        let items: Vec<FixedLen> = vec![
            FixedLen { a: 0, b: 0, c: 0 },
            FixedLen { a: 1, b: 1, c: 1 },
            FixedLen { a: 1, b: 0, c: 1 },
        ];

        let expected_encodings = vec![
            //  | u16--| u64----------------------------| u32----------|
            vec![00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00],
            vec![01, 00, 01, 00, 00, 00, 00, 00, 00, 00, 01, 00, 00, 00],
            vec![01, 00, 00, 00, 00, 00, 00, 00, 00, 00, 01, 00, 00, 00],
        ];

        for i in 0..items.len() {
            assert_eq!(
                items[i].as_ssz_bytes(),
                expected_encodings[i],
                "Failed on {}",
                i
            );
        }
    }

    #[test]
    fn fixed_len_excess_bytes() {
        let fixed = FixedLen { a: 1, b: 2, c: 3 };

        let mut bytes = fixed.as_ssz_bytes();
        bytes.append(&mut vec![0]);

        assert_eq!(
            FixedLen::from_ssz_bytes(&bytes),
            Err(DecodeError::InvalidByteLength {
                len: 15,
                expected: 14,
            })
        );
    }

    #[test]
    fn vec_of_fixed_len_struct() {
        let items: Vec<FixedLen> = vec![
            FixedLen { a: 0, b: 0, c: 0 },
            FixedLen { a: 1, b: 1, c: 1 },
            FixedLen { a: 1, b: 0, c: 1 },
        ];

        round_trip(items);
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    struct VariableLen {
        a: u16,
        b: Vec<u16>,
        c: u32,
    }

    #[test]
    fn offset_into_fixed_bytes() {
        let bytes = vec![
            //  1   2   3   4   5   6   7   8   9   10  11  12  13  14  15
            //      | offset        | u32           | variable
            01, 00, 09, 00, 00, 00, 01, 00, 00, 00, 00, 00, 01, 00, 02, 00,
        ];

        assert_eq!(
            VariableLen::from_ssz_bytes(&bytes),
            Err(DecodeError::OutOfBoundsByte { i: 9 })
        );
    }

    #[test]
    fn variable_len_excess_bytes() {
        let variable = VariableLen {
            a: 1,
            b: vec![2],
            c: 3,
        };

        let mut bytes = variable.as_ssz_bytes();
        bytes.append(&mut vec![0]);

        // The error message triggered is not so helpful, it's caught by a side-effect. Just
        // checking there is _some_ error is fine.
        assert!(VariableLen::from_ssz_bytes(&bytes).is_err());
    }

    #[test]
    fn first_offset_skips_byte() {
        let bytes = vec![
            //  1   2   3   4   5   6   7   8   9   10  11  12  13  14  15
            //      | offset        | u32           | variable
            01, 00, 11, 00, 00, 00, 01, 00, 00, 00, 00, 00, 01, 00, 02, 00,
        ];

        assert_eq!(
            VariableLen::from_ssz_bytes(&bytes),
            Err(DecodeError::OutOfBoundsByte { i: 11 })
        );
    }

    #[test]
    fn variable_len_struct_encoding() {
        let items: Vec<VariableLen> = vec![
            VariableLen {
                a: 0,
                b: vec![],
                c: 0,
            },
            VariableLen {
                a: 1,
                b: vec![0],
                c: 1,
            },
            VariableLen {
                a: 1,
                b: vec![0, 1, 2],
                c: 1,
            },
        ];

        let expected_encodings = vec![
            //   00..................................09
            //  | u16--| vec offset-----| u32------------| vec payload --------|
            vec![00, 00, 10, 00, 00, 00, 00, 00, 00, 00],
            vec![01, 00, 10, 00, 00, 00, 01, 00, 00, 00, 00, 00],
            vec![
                01, 00, 10, 00, 00, 00, 01, 00, 00, 00, 00, 00, 01, 00, 02, 00,
            ],
        ];

        for i in 0..items.len() {
            assert_eq!(
                items[i].as_ssz_bytes(),
                expected_encodings[i],
                "Failed on {}",
                i
            );
        }
    }

    #[test]
    fn vec_of_variable_len_struct() {
        let items: Vec<VariableLen> = vec![
            VariableLen {
                a: 0,
                b: vec![],
                c: 0,
            },
            VariableLen {
                a: 255,
                b: vec![0, 1, 2, 3],
                c: 99,
            },
            VariableLen {
                a: 255,
                b: vec![0],
                c: 99,
            },
            VariableLen {
                a: 50,
                b: vec![0],
                c: 0,
            },
        ];

        round_trip(items);
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    struct ThreeVariableLen {
        a: u16,
        b: Vec<u16>,
        c: Vec<u16>,
        d: Vec<u16>,
    }

    #[test]
    fn three_variable_len() {
        let vec: Vec<ThreeVariableLen> = vec![ThreeVariableLen {
            a: 42,
            b: vec![0],
            c: vec![1],
            d: vec![2],
        }];

        round_trip(vec);
    }

    #[test]
    fn offsets_decreasing() {
        let bytes = vec![
            //  1   2   3   4   5   6   7   8   9   10  11  12  13  14  15
            //      | offset        | offset        | offset        | variable
            01, 00, 14, 00, 00, 00, 15, 00, 00, 00, 14, 00, 00, 00, 00, 00,
        ];

        assert_eq!(
            ThreeVariableLen::from_ssz_bytes(&bytes),
            Err(DecodeError::OutOfBoundsByte { i: 14 })
        );
    }

    #[derive(Debug, PartialEq, Encode, Decode)]
    struct TwoVariableLenOptions {
        a: u16,
        b: Option<u16>,
        c: Option<Vec<u16>>,
        d: Option<Vec<u16>>,
    }

    #[test]
    fn two_variable_len_options_encoding() {
        let s = TwoVariableLenOptions {
            a: 42,
            b: None,
            c: Some(vec![0]),
            d: None,
        };

        let bytes = vec![
            //  1   2   3   4   5   6   7   8   9   10  11  12  13  14  15  16  17  18  19  20  21
            //      | option<u16>   | offset        | offset        | option<u16    | 1st list
            42, 00, 14, 00, 00, 00, 18, 00, 00, 00, 24, 00, 00, 00, 00, 00, 00, 00, 01, 00, 00, 00,
            //  23  24  25  26  27
            //      | 2nd list
            00, 00, 00, 00, 00, 00,
        ];

        assert_eq!(s.as_ssz_bytes(), bytes);
    }

    #[test]
    fn two_variable_len_options_round_trip() {
        let vec: Vec<TwoVariableLenOptions> = vec![
            TwoVariableLenOptions {
                a: 42,
                b: Some(12),
                c: Some(vec![0]),
                d: Some(vec![1]),
            },
            TwoVariableLenOptions {
                a: 42,
                b: Some(12),
                c: Some(vec![0]),
                d: None,
            },
            TwoVariableLenOptions {
                a: 42,
                b: None,
                c: Some(vec![0]),
                d: None,
            },
            TwoVariableLenOptions {
                a: 42,
                b: None,
                c: None,
                d: None,
            },
        ];

        round_trip(vec);
    }
}
