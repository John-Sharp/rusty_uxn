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

// strips macro definitions out of token stream, and expands
// macro invocations
pub fn process_macros<I>(input: I) -> impl Iterator<Item = Result<UxnToken, AsmError>>
where
    I: Iterator<Item = Result<UxnToken, AsmError>>,
{
    let mut macros = HashMap::new();
    let mut state = MacroState::MainBody;

    input.filter_map(move |t| match t {
        Err(e) => Some(Err(e)),
        Ok(UxnToken::MacroDefine(ref macro_name)) => match state {
            MacroState::MainBody => {
                if macros.contains_key(macro_name) {
                    return Some(Err(AsmError::DoubleMacroDefine {
                        macro_name: macro_name.clone(),
                    }));
                }

                state = MacroState::MacroDefinitionHead {
                    macro_name: macro_name.clone(),
                };
                return None;
            }
            MacroState::MacroDefinitionHead { macro_name: ref _m } => {
                let macro_name = macro_name.clone();
                return Some(Err(AsmError::MalformedMacroDefine { macro_name }));
            }
            MacroState::MacroDefinitionBody {
                macro_name: ref outer_macro_name,
                macro_body: ref _b,
            } => {
                let inner_macro_name = macro_name.clone();
                let outer_macro_name = outer_macro_name.clone();
                return Some(Err(AsmError::MacroDefineWithinMacro {
                    outer_macro_name,
                    inner_macro_name,
                }));
            }
        },
        Ok(UxnToken::MacroStartDelimiter) => match state {
            MacroState::MainBody => {
                return Some(Err(AsmError::MacroStartDelimiterMisplaced));
            }
            MacroState::MacroDefinitionHead { ref macro_name } => {
                state = MacroState::MacroDefinitionBody {
                    macro_name: macro_name.clone(),
                    macro_body: Vec::new(),
                };
                return None;
            }
            MacroState::MacroDefinitionBody {
                macro_name: ref _name,
                macro_body: ref _body,
            } => {
                return Some(Err(AsmError::MacroStartDelimiterMisplaced));
            }
        },
        Ok(UxnToken::MacroEndDelimiter) => match state {
            MacroState::MainBody => {
                return Some(Err(AsmError::MacroEndDelimiterMisplaced));
            }
            MacroState::MacroDefinitionHead { macro_name: _ } => {
                return Some(Err(AsmError::MacroEndDelimiterMisplaced));
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

                return None;
            }
        },
        // TODO macro invocations
        Ok(t) => match state {
            MacroState::MainBody => {
                return Some(Ok(t));
            }
            MacroState::MacroDefinitionHead { ref macro_name } => {
                return Some(Err(AsmError::MalformedMacroDefine {
                    macro_name: macro_name.clone(),
                }));
            }
            MacroState::MacroDefinitionBody {
                macro_name: _,
                ref mut macro_body,
            } => {
                macro_body.push(t);
                return None;
            }
        },
    })
}
