use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum OpCode {
    Brk,
    Deo,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OpObject {
    keep: bool,
    ret: bool,
    short: bool,
    op_code: OpCode,
}

impl OpObject {
    pub fn get_bytes(&self) -> Vec<u8> {
        let byte = match self.op_code {
            OpCode::Brk => 0x00,
            OpCode::Deo => 0x17,
        };

        let byte = if self.keep { byte | 0b10000000 } else { byte };

        let byte = if self.ret { byte | 0b01000000 } else { byte };

        let byte = if self.short { byte | 0b00100000 } else { byte };

        return vec![byte];
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseOpObjectError {}

impl FromStr for OpObject {
    type Err = ParseOpObjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 3 {
            return Err(ParseOpObjectError {});
        }

        let opcode = s.get(0..3);
        if opcode.is_none() {
            return Err(ParseOpObjectError {});
        }

        let opcode = opcode.unwrap();

        let mut ret = match opcode {
            "BRK" => {
                if s.len() > 3 {
                    return Err(ParseOpObjectError {});
                }

                OpObject {
                    keep: false,
                    ret: false,
                    short: false,
                    op_code: OpCode::Brk,
                }
            }
            "LIT" => OpObject {
                keep: true,
                ret: false,
                short: false,
                op_code: OpCode::Brk,
            },
            "DEO" => OpObject {
                keep: false,
                ret: false,
                short: false,
                op_code: OpCode::Deo,
            },
            _ => return Err(ParseOpObjectError {}),
        };

        for mode_flag in s.chars().skip(3) {
            match mode_flag {
                '2' => {
                    ret.short = true;
                }
                'k' => {
                    if ret.op_code == OpCode::Brk {
                        return Err(ParseOpObjectError {});
                    }

                    ret.keep = true;
                }
                'r' => {
                    ret.ret = true;
                }
                _ => return Err(ParseOpObjectError {}),
            };
        }

        return Ok(ret);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test `get_bytes` function; for each possible op code,
    // verify that the correct sequence of bytes is produced for it
    #[test]
    fn test_get_bytes_happy() {
        let inputs = [(OpCode::Brk, vec![0x00]), (OpCode::Deo, vec![0x17])];

        for (input, expected_output) in inputs.into_iter() {
            let input = OpObject {
                keep: false,
                ret: false,
                short: false,
                op_code: input,
            };

            let output = input.get_bytes();
            assert_eq!(output, expected_output);
        }
    }

    // test `get_bytes` function; verify that a selection
    // of modifier flags leads to the correct output
    #[test]
    fn test_get_bytes_happy_with_modifiers() {
        let input = OpObject {
            keep: true,
            ret: false,
            short: true,
            op_code: OpCode::Deo,
        };
        let expected_output = vec![0xb7];
        let output = input.get_bytes();

        assert_eq!(output, expected_output);

        let input = OpObject {
            keep: true,
            ret: true,
            short: false,
            op_code: OpCode::Deo,
        };
        let expected_output = vec![0xd7];
        let output = input.get_bytes();

        assert_eq!(output, expected_output);
    }

    // test `from_str` function for operation
    // strings with no modifier flags
    #[test]
    fn test_from_str_happy() {
        let inputs = [("BRK", OpCode::Brk), ("DEO", OpCode::Deo)];

        for (input, expected_output) in inputs {
            let output = input.parse::<OpObject>();
            let expected_output = Ok(OpObject {
                keep: false,
                ret: false,
                short: false,
                op_code: expected_output,
            });

            assert_eq!(output, expected_output);
        }
    }

    // test `from_str` function for LIT operation
    // string with no modifier flags
    #[test]
    fn test_from_str_happy_lit() {
        let output = "LIT".parse::<OpObject>();
        let expected_output = Ok(OpObject {
            keep: true,
            ret: false,
            short: false,
            op_code: OpCode::Brk,
        });

        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_from_str_happy_mode_flags() {
        let input = "DEO2rk";
        let expected_output = Ok(OpObject {
            keep: true,
            ret: true,
            short: true,
            op_code: OpCode::Deo,
        });

        let output = input.parse::<OpObject>();
        assert_eq!(output, expected_output);

        let input = "DEOkr2";
        let expected_output = Ok(OpObject {
            keep: true,
            ret: true,
            short: true,
            op_code: OpCode::Deo,
        });

        let output = input.parse::<OpObject>();
        assert_eq!(output, expected_output);

        let input = "DEOr2";
        let expected_output = Ok(OpObject {
            keep: false,
            ret: true,
            short: true,
            op_code: OpCode::Deo,
        });

        let output = input.parse::<OpObject>();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_from_str_forbidden_mode_flags() {
        let inputs = ["BRKr", "BRKk", "BRK2", "LITk"];

        for input in inputs {
            let output = input.parse::<OpObject>();
            assert_eq!(output, Err(ParseOpObjectError {}));
        }
    }

    #[test]
    fn test_from_str_unrecognised_op_string() {
        let inputs = ["BRKK", "BOK", "BK"];

        for input in inputs {
            let output = input.parse::<OpObject>();
            assert_eq!(output, Err(ParseOpObjectError {}));
        }
    }
}

