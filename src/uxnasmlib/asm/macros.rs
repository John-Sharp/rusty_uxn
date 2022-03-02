use super::tokens::UxnToken;
use super::AsmError;
use std::collections::HashMap;
use std::mem;

enum MacroState {
    MainBody,
    MacroDefinitionHead {
        macro_name: String,
    },
    MacroDefinitionBody {
        macro_name: String,
        macro_body: Vec<UxnToken>,
    },
}

const MAX_MACRO_DEPTH: u8 = 5;

fn expand_macro(macro_name: &str, macros: &HashMap<String, Vec<UxnToken>>, depth: u8) -> Vec<Result<UxnToken, AsmError>>
{
    if depth >= MAX_MACRO_DEPTH {
        return vec!(Err(AsmError::MaxMacroCallDepthExceeded{macro_name: macro_name.to_owned()}));
    }

    if let Some(macro_body) = macros.get(macro_name) {
        return macro_body.iter().cloned()
            .flat_map(|e| {
                match e {
                    UxnToken::MacroInvocation(ref macro_name) => {
                        expand_macro(macro_name, macros, depth+1)
                    }
                    _ => {
                        vec!(Ok(e))
                    }
                }
            }).collect::<Vec<_>>();
    } else {
        return vec!(Err(AsmError::UnrecognisedMacro{macro_name: macro_name.to_owned()}));
    }
}

// strips macro definitions out of token stream, and expands
// macro invocations
pub fn process_macros<I>(input: I) -> impl Iterator<Item = Result<UxnToken, AsmError>>
where
    I: Iterator<Item = Result<UxnToken, AsmError>>,
{
    let mut macros = HashMap::new();
    let mut state = MacroState::MainBody;

    input.flat_map(move |t| match t {
        Err(e) => vec!(Err(e)),
        Ok(UxnToken::MacroDefine(ref macro_name)) => match state {
            MacroState::MainBody => {
                if macros.contains_key(macro_name) {
                    return vec!(Err(AsmError::DoubleMacroDefine {
                        macro_name: macro_name.clone(),
                    }));
                }

                state = MacroState::MacroDefinitionHead {
                    macro_name: macro_name.clone(),
                };
                return vec!();
            }
            MacroState::MacroDefinitionHead { macro_name: ref _m } => {
                let macro_name = macro_name.clone();
                return vec!(Err(AsmError::MalformedMacroDefine { macro_name }));
            }
            MacroState::MacroDefinitionBody {
                macro_name: ref outer_macro_name,
                macro_body: ref _b,
            } => {
                let inner_macro_name = macro_name.clone();
                let outer_macro_name = outer_macro_name.clone();
                return vec!(Err(AsmError::MacroDefineWithinMacro {
                    outer_macro_name,
                    inner_macro_name,
                }));
            }
        },
        Ok(UxnToken::MacroStartDelimiter) => match state {
            MacroState::MainBody => {
                return vec!(Err(AsmError::MacroStartDelimiterMisplaced));
            }
            MacroState::MacroDefinitionHead { ref macro_name } => {
                state = MacroState::MacroDefinitionBody {
                    macro_name: macro_name.clone(),
                    macro_body: Vec::new(),
                };
                return vec!();
            }
            MacroState::MacroDefinitionBody {
                macro_name: ref _name,
                macro_body: ref _body,
            } => {
                return vec!(Err(AsmError::MacroStartDelimiterMisplaced));
            }
        },
        Ok(UxnToken::MacroEndDelimiter) => match state {
            MacroState::MainBody => {
                return vec!(Err(AsmError::MacroEndDelimiterMisplaced));
            }
            MacroState::MacroDefinitionHead { macro_name: _ } => {
                return vec!(Err(AsmError::MacroEndDelimiterMisplaced));
            }
            MacroState::MacroDefinitionBody {
                macro_name: _,
                macro_body: _,
            } => {
                let old_state = mem::replace(&mut state, MacroState::MainBody);
                if let MacroState::MacroDefinitionBody {
                    macro_name,
                    macro_body,
                } = old_state
                {
                    macros.insert(macro_name.clone(), macro_body);
                }

                return vec!();
            }
        },
        Ok(UxnToken::MacroInvocation(ref macro_name)) => match state {
            MacroState::MainBody => {
                return expand_macro(macro_name, &macros, 0);
            }
            MacroState::MacroDefinitionHead { ref macro_name } => {
                return vec!(Err(AsmError::MalformedMacroDefine {
                    macro_name: macro_name.clone(),
                }));
            }
            MacroState::MacroDefinitionBody {
                macro_name: _,
                ref mut macro_body,
            } => {
                macro_body.push(t.unwrap());
                return vec!();
            }
        }
        Ok(t) => match state {
            MacroState::MainBody => {
                return vec!(Ok(t));
            }
            MacroState::MacroDefinitionHead { ref macro_name } => {
                return vec!(Err(AsmError::MalformedMacroDefine {
                    macro_name: macro_name.clone(),
                }));
            }
            MacroState::MacroDefinitionBody {
                macro_name: _,
                ref mut macro_body,
            } => {
                macro_body.push(t);
                return vec!();
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // test `process_macros` function; that it strips correctly defined,
    // but unused, macros from the input stream
    #[test]
    fn test_unused_macro_strip() {
        let input = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawByte(0xbb)),
            Ok(UxnToken::MacroDefine("test_macro".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::MacroEndDelimiter),
            Ok(UxnToken::RawByte(0xcc)),
            Ok(UxnToken::RawByte(0xdd)),
            Ok(UxnToken::MacroDefine("test_macro2".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x88)),
            Ok(UxnToken::RawByte(0x88)),
            Ok(UxnToken::MacroEndDelimiter),
            Ok(UxnToken::RawByte(0xee)),
            Ok(UxnToken::RawByte(0xff)),
        ];

        let expected_output = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawByte(0xbb)),
            Ok(UxnToken::RawByte(0xcc)),
            Ok(UxnToken::RawByte(0xdd)),
            Ok(UxnToken::RawByte(0xee)),
            Ok(UxnToken::RawByte(0xff)),
        ];

        let output = process_macros(input.into_iter()).collect::<Vec<_>>();

        assert_eq!(output, expected_output);
    }

    // test `process_macros` function; that it generates the correct error
    // when a macro is double defined
    #[test]
    fn test_double_define_error() {
        let input = vec![
            Ok(UxnToken::MacroDefine("test_macro".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::MacroEndDelimiter),
            Ok(UxnToken::RawByte(0xcc)),
            Ok(UxnToken::RawByte(0xdd)),
            Ok(UxnToken::MacroDefine("test_macro".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x79)),
            Ok(UxnToken::RawByte(0x97)),
            Ok(UxnToken::MacroEndDelimiter),
        ];

        let output = process_macros(input.into_iter()).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(
            output,
            Err(AsmError::DoubleMacroDefine {
                macro_name: "test_macro".to_owned()
            })
        );
    }

    // test `process_macros` function; that a macro declaration followed
    // by another macro declaration with no opening bracket encounted
    // generates a malformed macro define error
    #[test]
    fn test_malformed_macro_define_double_head() {
        let input = vec![
            Ok(UxnToken::MacroDefine("test_macro".to_owned())),
            Ok(UxnToken::MacroDefine("test_macro_b".to_owned())),
        ];

        let output = process_macros(input.into_iter()).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(
            output,
            Err(AsmError::MalformedMacroDefine {
                macro_name: "test_macro_b".to_owned()
            })
        );
    }

    // test that attempting to define a macro inside a macro results
    // in an appropriate error
    #[test]
    fn test_macro_define_within_macro_error() {
        let input = vec![
            Ok(UxnToken::MacroDefine("test_macro".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::MacroDefine("inner_macro".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0xf9)),
            Ok(UxnToken::MacroEndDelimiter),
            Ok(UxnToken::MacroEndDelimiter),
        ];

        let output = process_macros(input.into_iter()).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(
            output,
            Err(AsmError::MacroDefineWithinMacro {
                outer_macro_name: "test_macro".to_owned(),
                inner_macro_name: "inner_macro".to_owned(),
            })
        );
    }

    // test that the opening curly bracket of a macro, when on
    // its own in the main program body, results in an appropriate
    // error
    #[test]
    fn test_macro_start_delimiter_misplaced_in_main_body() {
        let input = vec![
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0xf9)),
        ];

        let output = process_macros(input.into_iter()).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(output, Err(AsmError::MacroStartDelimiterMisplaced));
    }

    #[test]
    fn test_macro_start_delimiter_misplaced_in_macro_body() {
        let input = vec![
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::MacroDefine("test_macro".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0xf9)),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0xf9)),
            Ok(UxnToken::MacroEndDelimiter),
            Ok(UxnToken::MacroEndDelimiter),
        ];

        let output = process_macros(input.into_iter()).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(output, Err(AsmError::MacroStartDelimiterMisplaced));
    }

    #[test]
    fn test_macro_end_delimiter_misplaced_in_main_body() {
        let input = vec![
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::MacroEndDelimiter),
            Ok(UxnToken::RawByte(0xf9)),
        ];

        let output = process_macros(input.into_iter()).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(output, Err(AsmError::MacroEndDelimiterMisplaced));
    }

    #[test]
    fn test_macro_end_delimiter_misplaced_macro_head() {
        let input = vec![
            Ok(UxnToken::MacroDefine("test_macro".to_owned())),
            Ok(UxnToken::MacroEndDelimiter),
        ];

        let output = process_macros(input.into_iter()).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(output, Err(AsmError::MacroEndDelimiterMisplaced));
    }

    #[test]
    fn test_incomplete_head_error() {
        let input = vec![
            Ok(UxnToken::MacroDefine("test_macro".to_owned())),
            Ok(UxnToken::RawByte(0x99)),
        ];

        let output = process_macros(input.into_iter()).collect::<Result<Vec<_>, AsmError>>();
        assert_eq!(
            output,
            Err(AsmError::MalformedMacroDefine {
                macro_name: "test_macro".to_owned()
            })
        );
    }

    #[test]
    fn test_macro_invocation() {
        let input = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawByte(0xbb)),
            Ok(UxnToken::MacroDefine("test_macro".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::MacroEndDelimiter),
            Ok(UxnToken::RawByte(0xcc)),
            Ok(UxnToken::RawByte(0xdd)),
            Ok(UxnToken::MacroInvocation("test_macro".to_owned())),
            Ok(UxnToken::RawByte(0xff)),
        ];

        let expected_output = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawByte(0xbb)),
            Ok(UxnToken::RawByte(0xcc)),
            Ok(UxnToken::RawByte(0xdd)),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::RawByte(0xff)),
        ];

        let output = process_macros(input.into_iter()).collect::<Vec<_>>();

        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_nested_macro_invocation() {
        let input = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xaa)),

            Ok(UxnToken::MacroDefine("test_macro3".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x79)),
            Ok(UxnToken::RawByte(0x79)),
            Ok(UxnToken::MacroInvocation("test_macro4".to_owned())),
            Ok(UxnToken::MacroEndDelimiter),

            Ok(UxnToken::RawByte(0xbb)),

            Ok(UxnToken::MacroDefine("test_macro".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::MacroInvocation("test_macro2".to_owned())),
            Ok(UxnToken::RawByte(0x99)),
            Ok(UxnToken::MacroEndDelimiter),

            Ok(UxnToken::RawByte(0xff)),

            Ok(UxnToken::MacroDefine("test_macro2".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x88)),
            Ok(UxnToken::MacroInvocation("test_macro3".to_owned())),
            Ok(UxnToken::RawByte(0x88)),
            Ok(UxnToken::MacroEndDelimiter),

            Ok(UxnToken::MacroDefine("test_macro4".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x11)),
            Ok(UxnToken::MacroInvocation("test_macro5".to_owned())),
            Ok(UxnToken::MacroEndDelimiter),

            Ok(UxnToken::MacroDefine("test_macro5".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x12)),
            Ok(UxnToken::MacroEndDelimiter),

            Ok(UxnToken::RawByte(0xcc)),
            Ok(UxnToken::MacroInvocation("test_macro".to_owned())),
            Ok(UxnToken::RawByte(0xdd)),
        ];

        let expected_output = vec![
            Ok(UxnToken::PadAbs(0x100)),
            Ok(UxnToken::RawByte(0xaa)),
            Ok(UxnToken::RawByte(0xbb)),
            Ok(UxnToken::RawByte(0xff)),
            Ok(UxnToken::RawByte(0xcc)),

            // invoking 'test_macro'
            Ok(UxnToken::RawByte(0x99)),

            // invoking 'test_macro2' (inside 'test_macro')
            Ok(UxnToken::RawByte(0x88)),

            // invoking 'test_macro3' (inside 'test_macro2')
            Ok(UxnToken::RawByte(0x79)),
            Ok(UxnToken::RawByte(0x79)),

            // invoking 'test_macro4' (inside 'test_macro3')
            Ok(UxnToken::RawByte(0x11)),

            // invoking 'test_macro5' (inside 'test_macro4')
            Ok(UxnToken::RawByte(0x12)),

            // leaving 'test_macro5'
            // leaving 'test_macro4'
            // leaving 'test_macro3'

            Ok(UxnToken::RawByte(0x88)),
            // leaving 'test_macro2'

            Ok(UxnToken::RawByte(0x99)),
            // leaving 'test_macro'

            Ok(UxnToken::RawByte(0xdd)),
        ];

        let output = process_macros(input.into_iter()).collect::<Vec<_>>();

        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_nested_macro_invocation_beyond_max() {
        let input = vec![
            Ok(UxnToken::MacroDefine("test_macro1".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x01)),
            Ok(UxnToken::MacroInvocation("test_macro2".to_owned())),
            Ok(UxnToken::MacroEndDelimiter),

            Ok(UxnToken::MacroDefine("test_macro2".to_owned())),
            Ok(UxnToken::MacroStartDelimiter),
            Ok(UxnToken::RawByte(0x02)),
            Ok(UxnToken::MacroInvocation("test_macro1".to_owned())),
            Ok(UxnToken::MacroEndDelimiter),

            Ok(UxnToken::MacroInvocation("test_macro1".to_owned())),
        ];

        let output = process_macros(input.into_iter()).collect::<Result<Vec<_>, AsmError>>();

        assert_eq!(output, Err(AsmError::MaxMacroCallDepthExceeded{
            macro_name: "test_macro2".to_owned()}));
    }
}
