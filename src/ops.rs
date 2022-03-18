use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OpCode {
    Brk,
    Inc,
    Pop,
    Dup,
    Nip,
    Swp,
    Ovr,
    Rot,
    Equ,
    Neq,
    Gth,
    Lth,
    Jmp,
    Jcn,
    Jsr,
    Sth,
    Ldz,
    Stz,
    Ldr,
    Str,
    Lda,
    Sta,
    Dei,
    Deo,
    Add,
    Sub,
    Mul,
    Div,
    And,
    Ora,
    Eor,
    Sft,
}

struct OpDescription {
    op_code: OpCode,
    byte: u8,
    token: &'static str,

    // TODO
    // #[cfg(any(test, feature = "asm"))]
    // handler
    
}

const OP_LIST: &'static [OpDescription] = &[
    OpDescription{op_code: OpCode::Brk, byte: 0x00, token: "BRK"},
    OpDescription{op_code: OpCode::Inc, byte: 0x01, token: "INC"},
    OpDescription{op_code: OpCode::Pop, byte: 0x02, token: "POP"},
    OpDescription{op_code: OpCode::Dup, byte: 0x03, token: "DUP"},
    OpDescription{op_code: OpCode::Nip, byte: 0x04, token: "NIP"},
    OpDescription{op_code: OpCode::Swp, byte: 0x05, token: "SWP"},
    OpDescription{op_code: OpCode::Ovr, byte: 0x06, token: "OVR"},
    OpDescription{op_code: OpCode::Rot, byte: 0x07, token: "ROT"},
    OpDescription{op_code: OpCode::Equ, byte: 0x08, token: "EQU"},
    OpDescription{op_code: OpCode::Neq, byte: 0x09, token: "NEQ"},
    OpDescription{op_code: OpCode::Gth, byte: 0x0a, token: "GTH"},
    OpDescription{op_code: OpCode::Lth, byte: 0x0b, token: "LTH"},
    OpDescription{op_code: OpCode::Jmp, byte: 0x0c, token: "JMP"},
    OpDescription{op_code: OpCode::Jcn, byte: 0x0d, token: "JCN"},
    OpDescription{op_code: OpCode::Jsr, byte: 0x0e, token: "JSR"},
    OpDescription{op_code: OpCode::Sth, byte: 0x0f, token: "STH"},
    OpDescription{op_code: OpCode::Ldz, byte: 0x10, token: "LDZ"},
    OpDescription{op_code: OpCode::Stz, byte: 0x11, token: "STZ"},
    OpDescription{op_code: OpCode::Ldr, byte: 0x12, token: "LDR"},
    OpDescription{op_code: OpCode::Str, byte: 0x13, token: "STR"},
    OpDescription{op_code: OpCode::Lda, byte: 0x14, token: "LDA"},
    OpDescription{op_code: OpCode::Sta, byte: 0x15, token: "STA"},
    OpDescription{op_code: OpCode::Dei, byte: 0x16, token: "DEI"},
    OpDescription{op_code: OpCode::Deo, byte: 0x17, token: "DEO"},
    OpDescription{op_code: OpCode::Add, byte: 0x18, token: "ADD"},
    OpDescription{op_code: OpCode::Sub, byte: 0x19, token: "SUB"},
    OpDescription{op_code: OpCode::Mul, byte: 0x1a, token: "MUL"},
    OpDescription{op_code: OpCode::Div, byte: 0x1b, token: "DIV"},
    OpDescription{op_code: OpCode::And, byte: 0x1c, token: "AND"},
    OpDescription{op_code: OpCode::Ora, byte: 0x1d, token: "ORA"},
    OpDescription{op_code: OpCode::Eor, byte: 0x1e, token: "EOR"},
    OpDescription{op_code: OpCode::Sft, byte: 0x1f, token: "SFT"},
];

#[derive(Debug, PartialEq, Clone)]
pub struct OpObject {
    keep: bool,
    ret: bool,
    short: bool,
    op_code: OpCode,
}

impl OpObject {
    pub fn get_bytes(&self) -> Vec<u8> {
        let byte = OP_LIST.iter().find(
            |e| e.op_code == self.op_code)
            .expect("No matching OP_LIST entry for OpCode")
            .byte;

        let byte = if self.keep { byte | 0b10000000 } else { byte };

        let byte = if self.ret { byte | 0b01000000 } else { byte };

        let byte = if self.short { byte | 0b00100000 } else { byte };

        return vec![byte];
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseOpObjectError {}

fn plain_op_object(op_code: OpCode) -> OpObject {
    OpObject{
        keep: false,
        ret: false,
        short: false,
        op_code
    }
}

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
            _ => {
                if let Some(op_description) = OP_LIST.iter().find(
                    |e| e.token == opcode
                    ) {
                    plain_op_object(op_description.op_code)
                } else {
                    return Err(ParseOpObjectError {})
                }
            },
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
        let inputs = [
            (OpCode::Brk, vec![0x00]),
            (OpCode::Inc, vec![0x01]),
            (OpCode::Pop, vec![0x02]),
            (OpCode::Dup, vec![0x03]),
            (OpCode::Nip, vec![0x04]),
            (OpCode::Swp, vec![0x05]),
            (OpCode::Ovr, vec![0x06]),
            (OpCode::Rot, vec![0x07]),
            (OpCode::Equ, vec![0x08]),
            (OpCode::Neq, vec![0x09]),
            (OpCode::Gth, vec![0x0a]),
            (OpCode::Lth, vec![0x0b]),
            (OpCode::Jmp, vec![0x0c]),
            (OpCode::Jcn, vec![0x0d]),
            (OpCode::Jsr, vec![0x0e]),
            (OpCode::Sth, vec![0x0f]),
            (OpCode::Ldz, vec![0x10]),
            (OpCode::Stz, vec![0x11]),
            (OpCode::Ldr, vec![0x12]),
            (OpCode::Str, vec![0x13]),
            (OpCode::Lda, vec![0x14]),
            (OpCode::Sta, vec![0x15]),
            (OpCode::Dei, vec![0x16]),
            (OpCode::Deo, vec![0x17]),
            (OpCode::Add, vec![0x18]),
            (OpCode::Sub, vec![0x19]),
            (OpCode::Mul, vec![0x1a]),
            (OpCode::Div, vec![0x1b]),
            (OpCode::And, vec![0x1c]),
            (OpCode::Ora, vec![0x1d]),
            (OpCode::Eor, vec![0x1e]),
            (OpCode::Sft, vec![0x1f]),];

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
        let inputs = [
            ("BRK", OpCode::Brk),
            ("INC", OpCode::Inc),
            ("POP", OpCode::Pop),
            ("DUP", OpCode::Dup),
            ("NIP", OpCode::Nip),
            ("SWP", OpCode::Swp),
            ("OVR", OpCode::Ovr),
            ("ROT", OpCode::Rot),
            ("EQU", OpCode::Equ),
            ("NEQ", OpCode::Neq),
            ("GTH", OpCode::Gth),
            ("LTH", OpCode::Lth),
            ("JMP", OpCode::Jmp),
            ("JCN", OpCode::Jcn),
            ("JSR", OpCode::Jsr),
            ("STH", OpCode::Sth),
            ("LDZ", OpCode::Ldz),
            ("STZ", OpCode::Stz),
            ("LDR", OpCode::Ldr),
            ("STR", OpCode::Str),
            ("LDA", OpCode::Lda),
            ("STA", OpCode::Sta),
            ("DEI", OpCode::Dei),
            ("DEO", OpCode::Deo),
            ("ADD", OpCode::Add),
            ("SUB", OpCode::Sub),
            ("MUL", OpCode::Mul),
            ("DIV", OpCode::Div),
            ("AND", OpCode::And),
            ("ORA", OpCode::Ora),
            ("EOR", OpCode::Eor),
            ("SFT", OpCode::Sft),
        ];

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

