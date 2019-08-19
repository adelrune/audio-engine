extern crate proc_macro;
extern crate proc_quote;
use self::proc_macro::TokenStream;

use proc_quote::{quote, quote_spanned, ToTokens, TokenStreamExt, __rt};
use syn::parse::{Parser ,Parse, ParseStream, Result, };
use syn::spanned::Spanned;
use syn::token;
use syn::punctuated::Punctuated;
use syn::{ExprCall, ExprParen, parenthesized, braced, parse_macro_input, Expr, Ident, Token, Type, Visibility, BinOp, LitFloat, LitInt };
use std::vec;
use std::collections::HashSet;

struct Equation {
    head: Box<OperandNode>,
}

// self.output_states[0] = self.modmod.next(0.3, 300.0, 660.0);
// self.output_states[1] = self.modulator.next(self.output_states[0], 220.0, 440.0);
// self.output_states[2] = self.sine_osc.next(self.output_states[1], 1.0, 0.0);
// self.output_states[3] = self.disto_mod.next(2.3, 3.0, 3.2);
// self.output_states[4] = self.output.next(self.output_states[2] + 0.2 * self.output_states[4], self.output_states[3]);

impl Equation {
    fn get_tokens_terminal(&self, index: i32, audio_objects_mapping: &Vec<String>) -> __rt::TokenStream {
        let toks = self.head.get_tokens(index, audio_objects_mapping);
        let idx_val = index as usize;
        let tokens = quote! {
            self.output_states[#idx_val] = #toks;
        };
        tokens
    }

    fn get_tokens(&self, index: i32, audio_objects_mapping: &Vec<String>) -> __rt::TokenStream {
        self.head.get_tokens(index, audio_objects_mapping)
    }

    fn build_audio_object_mapping(&self, audio_objects_names: &HashSet<String>, audio_objects_mapping: &mut Vec<String>) {
        // this is a sideffectfull filling of a idx->name vector
        self.head.build_audio_object_mapping(audio_objects_names, audio_objects_mapping);
    }
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
impl OperandNode {

    fn build_audio_object_mapping(&self, audio_objects_names: &HashSet<String>, audio_objects_mapping: &mut Vec<String>) {
        // the implication of this is that if there are two root audio object calls in the same equation, one of them is not going to have a direct reference.
        // not sure if it really matters
        // the solution would be to intelligently split in sub expressions
        if self.content.3.is_some() {
            audio_objects_mapping.push(self.content.3.as_ref().unwrap().0.to_string());
        } else if self.next.is_some() {
            self.next.as_ref().unwrap().build_audio_object_mapping(audio_objects_names, audio_objects_mapping);
        }
    }

    fn get_tokens(&self, index: i32, audio_objects_mapping: &Vec<String>) -> __rt::TokenStream {
        let tokens;
        if self.content.0.is_some() {
            // literal int (are tranformed to floats)
            let cont = self.content.0.as_ref().unwrap().value() as f32;
            tokens = quote! {
                #cont
            };

        } else if self.content.1.is_some() {
            // literal float stays the same
            let cont = self.content.1.as_ref().unwrap();
            tokens = quote! {
                #cont
            };

        } else if self.content.2.is_some() {
            // bare identifier
            // finds the index of the call for the object
            let idx = audio_objects_mapping.iter().position(|x| x == &self.content.2.as_ref().unwrap().to_string());
            if idx.is_some() {
                let idx_val = idx.unwrap() as usize;
                // if we have the audio object, use its reference index
                tokens =  quote! {
                    self.output_states[#idx_val]
                };
            } else {
                // if we don't, assume its a constant declared somewhere and let the user deal with it
                let cont = self.content.2.as_ref().unwrap();
                tokens =  quote! {
                    #cont
                };
            }

        } else if self.content.3.is_some() {
            let ident = &self.content.3.as_ref().unwrap().0;
            let eq_tokens = self.content.3.as_ref().unwrap().1.iter().map(|x| x.get_tokens(index, audio_objects_mapping));
            let has_name = audio_objects_mapping.iter().position(|x| x == &ident.to_string()).is_some();
            if !has_name {
                // if we don't have the thing, assume its an external function and let the user deal with it.
                tokens = quote! {
                    #ident(#(#eq_tokens, )*)
                };
            } else {
                // if we have the thing, it means we really want to call its next from ourself.
                tokens = quote! {
                    self.#ident.next(#(#eq_tokens, )*)
                };
            }


        } else {
            // another Equation, just return its tokens
            tokens = self.content.4.as_ref().unwrap().get_tokens(index, audio_objects_mapping)
        }
        let toks;
        if self.next.is_some() {
            let next_toks = self.next.as_ref().unwrap().get_tokens(index, audio_objects_mapping);
            toks = quote! {
                #tokens #next_toks
            };
        } else {
            toks = quote! {
                #tokens
            };
        }
        toks
    }
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
        } else if input.peek(token::Paren){
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

impl OperationNode {
    fn build_audio_object_mapping(&self, audio_objects_names: &HashSet<String>, audio_objects_mapping: &mut Vec<String>) {
        if self.next.is_some() {
            self.next.as_ref().unwrap().build_audio_object_mapping(audio_objects_names, audio_objects_mapping);
        }
    }

    fn get_tokens(&self, index: i32, audio_objects_mapping: &Vec<String>) -> __rt::TokenStream {
        let toks;
        let cont = self.content;
        if self.next.is_some() {
            let next_toks = self.next.as_ref().unwrap().get_tokens(index, audio_objects_mapping);
            toks = quote! {
                #cont #next_toks
            }
        } else {
            toks = quote! {
                #cont
            }
        }
        toks
    }
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

struct AudioDeclaration {
    ident: Ident,
    constructor: ExprCall,
}

impl AudioDeclaration {
    fn get_struct_declaration(&self) -> __rt::TokenStream {
        let ident = &self.ident;
        let obj_name = &self.constructor.func;
        let exp = quote! {
            #ident : #obj_name,
        };
        exp
    }
    fn get_instanciation(&self) -> __rt::TokenStream {
        let ident = &self.ident;
        let obj_name = &self.constructor.func;
        let arguments = &self.constructor.args;
        let exp = quote! {
            #ident : #obj_name::new(#arguments),
        };
        exp
    }
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
        modifier(generator);
    }
)

Output ;


struct SignalChain {
    modmod: NaiveTableOsc,
    modulator: NaiveTableOsc,
    sine_osc: NaiveTableOsc,
    disto_mod: NaiveTableOsc,
    output: TanHWaveshaper,
    output_states: [f32;5]
}

impl SignalChain {
    pub fn new() -> Self {
        SignalChain {
            modmod:NaiveTableOsc::new(&SINE_2048),
            modulator:NaiveTableOsc::new(&SINE_2048),
            sine_osc:NaiveTableOsc::new(&SINE_2048),
            disto_mod:NaiveTableOsc::new(&TRIANGLE_2),
            output:TanHWaveshaper::new(),
            output_states: [0.0;5],
        }
    }

    fn next(&mut self) -> f32 {
        self.output_states[0] = self.modmod.next(0.3, 300.0, 660.0);
        self.output_states[1] = self.modulator.next(self.output_states[0], 220.0, 440.0);
        self.output_states[2] = self.sine_osc.next(self.output_states[1], 1.0, 0.0);
        self.output_states[3] = self.disto_mod.next(2.3, 3.0, 3.2);
        self.output_states[4] = self.output.next(self.output_states[2] + 0.2 * self.output_states[4], self.output_states[3]);
        self.output_states[4]
    }
}



*/



#[proc_macro]
pub fn signal_chain(input: TokenStream) -> TokenStream {
    let SignalChain {
        signal_chain_name,
        audio_objects,
        exprs,
    } = parse_macro_input!(input as SignalChain);
    let mut audio_objects_declarations = Vec::new();
    let mut audio_objects_instanciations = Vec::new();

    let mut audio_objects_names = HashSet::new();

    for audio_obj in audio_objects.iter() {
        audio_objects_declarations.push(audio_obj.get_struct_declaration());
        audio_objects_instanciations.push(audio_obj.get_instanciation());
        audio_objects_names.insert(audio_obj.ident.to_string());
    }

    let mut audio_objects_mapping = Vec::new();
    let num_states = exprs.iter().count();
    let end = num_states - 1;
    exprs.iter().for_each(|expr| expr.build_audio_object_mapping(&audio_objects_names, &mut audio_objects_mapping));
    let expr_tokens = exprs.iter().enumerate().map(|(idx, expr)| expr.get_tokens_terminal(idx as i32, &audio_objects_mapping));
    let expanded = quote! {
        struct #signal_chain_name {
            #(#audio_objects_declarations)*
            output_states: [f32;#num_states]
        }

        impl #signal_chain_name {
            pub fn new() -> Self {
                #signal_chain_name {
                    #(#audio_objects_instanciations)*
                    output_states: [0.0;#num_states]
                }
            }
            pub fn next(&mut self) -> f32 {
                #(#expr_tokens)*
                self.output_states[#end]
            }
        }
    };
    // let expanded = quote! {

    //     struct #signal_chain_name {
    //         modmod: NaiveTableOsc,
    //         modulator: NaiveTableOsc,
    //         sine_osc: NaiveTableOsc,
    //         disto_mod: NaiveTableOsc,
    //         output: TanHWaveshaper,
    //         output_states: [f32;5]
    //     }

    //     impl #signal_chain_name {
    //         pub fn new() -> Self {
    //             #signal_chain_name {
    //                 modmod:NaiveTableOsc::new(&SINE_2048),
    //                 modulator:NaiveTableOsc::new(&SINE_2048),
    //                 sine_osc:NaiveTableOsc::new(&SINE_2048),
    //                 disto_mod:NaiveTableOsc::new(&TRIANGLE_2),
    //                 output:TanHWaveshaper::new(),
    //                 output_states: [0.0;5],
    //             }
    //         }

    //         fn next(&mut self) -> f32 {
    //             self.output_states[0] = self.modmod.next(0.3, 300.0, 660.0);
    //             self.output_states[1] = self.modulator.next(self.output_states[0], 220.0, 440.0);
    //             self.output_states[2] = self.sine_osc.next(self.output_states[1], 1.0, 0.0);
    //             self.output_states[3] = self.disto_mod.next(2.3, 3.0, 3.2);
    //             self.output_states[4] = self.output.next(self.output_states[2] + 0.2 * self.output_states[4], self.output_states[3]);
    //             self.output_states[4]
    //         }
    //     }
    // };
    // let expanded = quote!{fn noop {!unimplemented()} };
    TokenStream::from(expanded)
}
