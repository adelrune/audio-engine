#![recursion_limit = "128"]
extern crate proc_macro;
extern crate proc_quote;
use self::proc_macro::TokenStream;

use proc_quote::{quote, quote_spanned};
use syn::parse::{Parser ,Parse, ParseStream, Result, };
use syn::spanned::Spanned;
use syn::token;
use syn::punctuated::Punctuated;
use syn::{ExprCall, ExprParen, parenthesized, braced, parse_macro_input, Expr, Ident, Token, Type, Visibility, BinOp, LitFloat, LitInt };
use std::vec;

struct Equation {
    head: Box<OperandNode>,
}

impl Parse for Equation {
    fn parse(input: ParseStream) -> Result<Self> {
        let head: OperandNode = input.parse()?;
        Ok(Equation {
            head: Box::new(head)
        }
        )
    }
}

struct OperandNode {
    // enums are too stupid
    content: (Option<LitInt>, Option<LitFloat>, Option<Ident>, Option<(Ident,Punctuated<Equation, Token![,]>)>, Option<Equation>),
    next: Option<Box<OperationNode>>
}

impl Parse for OperandNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut content:(Option<LitInt>, Option<LitFloat>, Option<Ident>, Option<(Ident, Punctuated<Equation, Token![,]>)>, Option<Equation>);
        // this is really ugly code.
        if input.peek(LitInt) {
            content = (Some(input.parse()?), None, None, None, None);
        } else if input.peek(LitFloat) {
            content = (None, Some(input.parse()?), None, None, None);
        // case for reference
        } else if input.peek(Ident) && !input.peek2(token::Paren){
            content = (None, None, Some(input.parse()?), None, None);
        // case for ExprCall
        } else if input.peek(Ident) && input.peek2(token::Paren){
            let paramParser = Equation::parse;
            let part_1: Ident = input.parse()?;
            let c;
            parenthesized!(c in input);
            let part_2: Punctuated<Equation, Token![,]> = c.parse_terminated(paramParser)?;
            content = (None, None, None, Some((part_1, part_2)), None);
        } else if input.peek(token::Paren ){
            content = (None, None, None, None, Some(input.parse()?));
        } else {
            content = (None, None, None, None, None);
        }
        let next: Option<Box<OperationNode>>;
        if ! (input.peek(Token![;]) || input.peek(Token![,]) || input.is_empty()) {
            next = Some(Box::new(input.parse()?));
        } else {
            next = None;
        }
        Ok( OperandNode {
            content: content,
            next: next
        }
        )
    }
}

struct OperationNode {
    content: BinOp, next: Option<Box<OperandNode>>
}

impl Parse for OperationNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let content: BinOp = input.parse()?;
        let next: Option<Box<OperandNode>>;
        if !(input.peek(Token![;]) || input.peek(Token![,]) || input.is_empty()) {
            next = Some(Box::new(input.parse()?));
        } else {
            next = None;
        }
        Ok( OperationNode {
            content: content,
            next: next
        }
        )
    }
}

// enum Operand {
//     Literal(LitFloat),
//     Reference(Ident),
//     StepAndSampleCall(ExprCall),
//     Eq(Equation),
// }

struct AudioDeclaration {
    ident: Ident,
    constructor: ExprCall,
}

impl Parse for AudioDeclaration {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let constructor: ExprCall = input.parse()?;
        Ok(AudioDeclaration {
            ident: ident,
            constructor:constructor
        }
        )
    }
}

// (Ident, Type, Expr)
struct SignalChain {
    signal_chain_name: Ident,
    audio_objects:  Punctuated<AudioDeclaration, Token![,]>,
    exprs: Punctuated<Equation, Token![;]>
}

impl Parse for SignalChain {
    fn parse(input: ParseStream) -> Result<Self> {
        let signal_chain_name: Ident = input.parse()?;
        // input.parse()::<Token![{]>()?;
        let audio_obj_content;
        let equation_content;
        // parses the declarations
        parenthesized!(audio_obj_content in input);
        let audio_obj_parser = AudioDeclaration::parse;
        let audio_objects: Punctuated<AudioDeclaration, Token![,]>= audio_obj_content.parse_terminated(audio_obj_parser)?;
        // parses the signal steps
        braced!(equation_content in input);
        let equation_parser = Equation::parse;
        let equations: Punctuated<Equation, Token![;]> = equation_content.parse_terminated(equation_parser)?;
        Ok(SignalChain {
            signal_chain_name:signal_chain_name,
            audio_objects:audio_objects,
            exprs: equations,
        })
    }
}


/* Input :
signal_chain!(
    my_signal_chain (

        modulator: Naivetableosc(&TRIANGLE_2),
        generator: Naivetableosc(&SINE_2048),
        modifier: TanhWaveshaper(),

    )
    {
        modulator(1.3, 220) + 660;
        generator(modulator, whatevershit(3,4,5) + 3);
        modifier(generator)
    }
)

Output ;

struct

*/



#[proc_macro]
pub fn signal_chain(input: TokenStream) -> TokenStream {
    let SignalChain {
        signal_chain_name,
        audio_objects,
        exprs,
    } = parse_macro_input!(input as SignalChain);
    let expanded = quote! {
        fn #signal_chain_name () -> &'static str {"#signal_chain_name"}
    };
    TokenStream::from(expanded)
}
