use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
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
            OpCode::Inc => 0x01,
            OpCode::Pop => 0x02,
            OpCode::Dup => 0x03,
            OpCode::Nip => 0x04,
            OpCode::Swp => 0x05,
            OpCode::Ovr => 0x06,
            OpCode::Rot => 0x07,
            OpCode::Equ => 0x08,
            OpCode::Neq => 0x09,
            OpCode::Gth => 0x0a,
            OpCode::Lth => 0x0b,
            OpCode::Jmp => 0x0c,
            OpCode::Jcn => 0x0d,
            OpCode::Jsr => 0x0e,
            OpCode::Sth => 0x0f,
            OpCode::Ldz => 0x10,
            OpCode::Stz => 0x11,
            OpCode::Ldr => 0x12,
            OpCode::Str => 0x13,
            OpCode::Lda => 0x14,
            OpCode::Sta => 0x15,
            OpCode::Dei => 0x16,
            OpCode::Deo => 0x17,
            OpCode::Add => 0x18,
            OpCode::Sub => 0x19,
            OpCode::Mul => 0x1a,
            OpCode::Div => 0x1b,
            OpCode::And => 0x1c,
            OpCode::Ora => 0x1d,
            OpCode::Eor => 0x1e,
            OpCode::Sft => 0x1f,
        };

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
            "INC" => plain_op_object(OpCode::Inc),
            "POP" => plain_op_object(OpCode::Pop),
            "DUP" => plain_op_object(OpCode::Dup),
            "NIP" => plain_op_object(OpCode::Nip),
            "SWP" => plain_op_object(OpCode::Swp),
            "OVR" => plain_op_object(OpCode::Ovr),
            "ROT" => plain_op_object(OpCode::Rot),
            "EQU" => plain_op_object(OpCode::Equ),
            "NEQ" => plain_op_object(OpCode::Neq),
            "GTH" => plain_op_object(OpCode::Gth),
            "LTH" => plain_op_object(OpCode::Lth),
            "JMP" => plain_op_object(OpCode::Jmp),
            "JCN" => plain_op_object(OpCode::Jcn),
            "JSR" => plain_op_object(OpCode::Jsr),
            "STH" => plain_op_object(OpCode::Sth),
            "LDZ" => plain_op_object(OpCode::Ldz),
            "STZ" => plain_op_object(OpCode::Stz),
            "LDR" => plain_op_object(OpCode::Ldr),
            "STR" => plain_op_object(OpCode::Str),
            "LDA" => plain_op_object(OpCode::Lda),
            "STA" => plain_op_object(OpCode::Sta),
            "DEI" => plain_op_object(OpCode::Dei),
            "DEO" => plain_op_object(OpCode::Deo),
            "ADD" => plain_op_object(OpCode::Add),
            "SUB" => plain_op_object(OpCode::Sub),
            "MUL" => plain_op_object(OpCode::Mul),
            "DIV" => plain_op_object(OpCode::Div),
            "AND" => plain_op_object(OpCode::And),
            "ORA" => plain_op_object(OpCode::Ora),
            "EOR" => plain_op_object(OpCode::Eor),
            "SFT" => plain_op_object(OpCode::Sft),
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

