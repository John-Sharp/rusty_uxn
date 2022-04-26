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

use crate::uxninterface::Uxn;
use crate::uxninterface::UxnError;

struct OpDescription {
    op_code: OpCode,
    byte: u8,
    token: &'static str,
    handler: fn(Box<&mut dyn Uxn>, bool, bool, bool) -> Result<(), UxnError>,
}

mod op_handlers;

const OP_LIST: &'static [OpDescription] = &[
    OpDescription{op_code: OpCode::Brk, byte: 0x00, token: "BRK", handler: op_handlers::lit_handler},
    OpDescription{op_code: OpCode::Inc, byte: 0x01, token: "INC", handler: op_handlers::inc_handler},
    OpDescription{op_code: OpCode::Pop, byte: 0x02, token: "POP", handler: op_handlers::pop_handler},
    OpDescription{op_code: OpCode::Dup, byte: 0x03, token: "DUP", handler: op_handlers::dup_handler},
    OpDescription{op_code: OpCode::Nip, byte: 0x04, token: "NIP", handler: op_handlers::nip_handler},
    OpDescription{op_code: OpCode::Swp, byte: 0x05, token: "SWP", handler: op_handlers::swp_handler},
    OpDescription{op_code: OpCode::Ovr, byte: 0x06, token: "OVR", handler: op_handlers::ovr_handler},
    OpDescription{op_code: OpCode::Rot, byte: 0x07, token: "ROT", handler: op_handlers::rot_handler},
    OpDescription{op_code: OpCode::Equ, byte: 0x08, token: "EQU", handler: op_handlers::equ_handler},
    OpDescription{op_code: OpCode::Neq, byte: 0x09, token: "NEQ", handler: op_handlers::neq_handler},
    OpDescription{op_code: OpCode::Gth, byte: 0x0a, token: "GTH", handler: op_handlers::gth_handler},
    OpDescription{op_code: OpCode::Lth, byte: 0x0b, token: "LTH", handler: op_handlers::lth_handler},
    OpDescription{op_code: OpCode::Jmp, byte: 0x0c, token: "JMP", handler: op_handlers::jmp_handler},
    OpDescription{op_code: OpCode::Jcn, byte: 0x0d, token: "JCN", handler: op_handlers::jcn_handler},
    OpDescription{op_code: OpCode::Jsr, byte: 0x0e, token: "JSR", handler: op_handlers::jsr_handler},
    OpDescription{op_code: OpCode::Sth, byte: 0x0f, token: "STH", handler: op_handlers::sth_handler},
    OpDescription{op_code: OpCode::Ldz, byte: 0x10, token: "LDZ", handler: op_handlers::ldz_handler},
    OpDescription{op_code: OpCode::Stz, byte: 0x11, token: "STZ", handler: op_handlers::stz_handler},
    OpDescription{op_code: OpCode::Ldr, byte: 0x12, token: "LDR", handler: op_handlers::ldr_handler},
    OpDescription{op_code: OpCode::Str, byte: 0x13, token: "STR", handler: op_handlers::str_handler},
    OpDescription{op_code: OpCode::Lda, byte: 0x14, token: "LDA", handler: op_handlers::lda_handler},
    OpDescription{op_code: OpCode::Sta, byte: 0x15, token: "STA", handler: op_handlers::sta_handler},
    OpDescription{op_code: OpCode::Dei, byte: 0x16, token: "DEI", handler: op_handlers::dei_handler},
    OpDescription{op_code: OpCode::Deo, byte: 0x17, token: "DEO", handler: op_handlers::deo_handler},
    OpDescription{op_code: OpCode::Add, byte: 0x18, token: "ADD", handler: op_handlers::add_handler},
    OpDescription{op_code: OpCode::Sub, byte: 0x19, token: "SUB", handler: op_handlers::sub_handler},
    OpDescription{op_code: OpCode::Mul, byte: 0x1a, token: "MUL", handler: op_handlers::mul_handler},
    OpDescription{op_code: OpCode::Div, byte: 0x1b, token: "DIV", handler: op_handlers::div_handler},
    OpDescription{op_code: OpCode::And, byte: 0x1c, token: "AND", handler: op_handlers::and_handler},
    OpDescription{op_code: OpCode::Ora, byte: 0x1d, token: "ORA", handler: op_handlers::ora_handler},
    OpDescription{op_code: OpCode::Eor, byte: 0x1e, token: "EOR", handler: op_handlers::eor_handler},
    OpDescription{op_code: OpCode::Sft, byte: 0x1f, token: "SFT", handler: op_handlers::sft_handler},
];

use crate::instruction::Instruction;
use crate::instruction::InstructionFactory;

pub struct OpObjectFactory {}

impl InstructionFactory for OpObjectFactory {
    fn from_byte(&self, byte: u8) -> Box<dyn Instruction> {
        return Box::new(OpObject::from_byte(byte));
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OpObject {
    keep: bool,
    ret: bool,
    short: bool,
    op_code: OpCode,
    handler_index: usize,
}

impl Instruction for OpObject {
    fn execute(&self, uxn: Box::<&mut dyn Uxn>) -> Result<(), UxnError> {
        (OP_LIST[self.handler_index].handler)(uxn, self.keep, self.short, self.ret)
    }
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

    pub fn from_byte(byte: u8) -> Self {
        let keep: bool = if byte & 0b10000000 > 0 { true } else { false };
        let ret: bool = if byte & 0b01000000 > 0 { true } else { false };
        let short: bool = if byte & 0b00100000 > 0 { true } else { false };

        let byte = byte & 0x1f;

        let index = OP_LIST.iter().position(
            |e| e.byte == byte)
            .expect("No matching OP_LIST entry for byte");

        let op_code = OP_LIST[index].op_code;

        return OpObject {keep, ret, short, op_code, handler_index: index}
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseOpObjectError {}

fn plain_op_object(op_code: OpCode, handler_index: usize) -> OpObject {
    OpObject{
        keep: false,
        ret: false,
        short: false,
        op_code,
        handler_index,
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
                    handler_index: OP_LIST.iter().position(
                        |e| e.op_code == OpCode::Brk
                    ).expect("could not find OpDescription for Brk")
                }
            }
            "LIT" => OpObject {
                keep: true,
                ret: false,
                short: false,
                op_code: OpCode::Brk,
                handler_index: OP_LIST.iter().position(
                    |e| e.op_code == OpCode::Brk
                ).expect("could not find OpDescription for Brk")
            },
            _ => {
                if let Some(op_description_index) = OP_LIST.iter().position(
                    |e| e.token == opcode
                    ) {
                    plain_op_object(OP_LIST[op_description_index].op_code,
                                    op_description_index)
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
                handler_index: 0,
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
            handler_index: 0,
        };
        let expected_output = vec![0xb7];
        let output = input.get_bytes();

        assert_eq!(output, expected_output);

        let input = OpObject {
            keep: true,
            ret: true,
            short: false,
            op_code: OpCode::Deo,
            handler_index: 0,
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
            ("BRK", (OpCode::Brk, 0)),
            ("INC", (OpCode::Inc, 1)),
            ("POP", (OpCode::Pop, 2)),
            ("DUP", (OpCode::Dup, 3)),
            ("NIP", (OpCode::Nip, 4)),
            ("SWP", (OpCode::Swp, 5)),
            ("OVR", (OpCode::Ovr, 6)),
            ("ROT", (OpCode::Rot, 7)),
            ("EQU", (OpCode::Equ, 8)),
            ("NEQ", (OpCode::Neq, 9)),
            ("GTH", (OpCode::Gth, 10)),
            ("LTH", (OpCode::Lth, 11)),
            ("JMP", (OpCode::Jmp, 12)),
            ("JCN", (OpCode::Jcn, 13)),
            ("JSR", (OpCode::Jsr, 14)),
            ("STH", (OpCode::Sth, 15)),
            ("LDZ", (OpCode::Ldz, 16)),
            ("STZ", (OpCode::Stz, 17)),
            ("LDR", (OpCode::Ldr, 18)),
            ("STR", (OpCode::Str, 19)),
            ("LDA", (OpCode::Lda, 20)),
            ("STA", (OpCode::Sta, 21)),
            ("DEI", (OpCode::Dei, 22)),
            ("DEO", (OpCode::Deo, 23)),
            ("ADD", (OpCode::Add, 24)),
            ("SUB", (OpCode::Sub, 25)),
            ("MUL", (OpCode::Mul, 26)),
            ("DIV", (OpCode::Div, 27)),
            ("AND", (OpCode::And, 28)),
            ("ORA", (OpCode::Ora, 29)),
            ("EOR", (OpCode::Eor, 30)),
            ("SFT", (OpCode::Sft, 31)),
        ];

        for (input, expected_output) in inputs {
            let output = input.parse::<OpObject>();
            let expected_output = Ok(OpObject {
                keep: false,
                ret: false,
                short: false,
                op_code: expected_output.0,
                handler_index: expected_output.1,
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
            handler_index: 0,
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
            handler_index: 23,
        });

        let output = input.parse::<OpObject>();
        assert_eq!(output, expected_output);

        let input = "DEOkr2";
        let expected_output = Ok(OpObject {
            keep: true,
            ret: true,
            short: true,
            op_code: OpCode::Deo,
            handler_index: 23,
        });

        let output = input.parse::<OpObject>();
        assert_eq!(output, expected_output);

        let input = "DEOr2";
        let expected_output = Ok(OpObject {
            keep: false,
            ret: true,
            short: true,
            op_code: OpCode::Deo,
            handler_index: 23,
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

