#![crate_type="dylib"]
#![feature(plugin_registrar, quote)]
#![feature(rustc_private)]

extern crate syntax;
extern crate rustc_plugin;
extern crate rustc_data_structures;

use syntax::codemap::Span;
use rustc_data_structures::small_vec::SmallVec;
use syntax::tokenstream::TokenTree;
use syntax::ast::Ident;
use syntax::ext::base::{ExtCtxt,MacResult,DummyResult,MacEager,IdentTT,get_single_str_from_tts};
use syntax::symbol::Symbol;
use rustc_plugin::Registry;

fn expand_mod_path<'a>(cx: &'a mut ExtCtxt,
                       sp: Span,
                       ident: Ident,
                       tts: Vec<TokenTree>)
                       -> Box<MacResult + 'a> {
    let path = match get_single_str_from_tts(cx, sp, tts.as_slice(), "mod_path!") {
        Some(string) => string,
        None => return DummyResult::expr(sp),
    };

    MacEager::items(SmallVec::one(quote_item!(cx,

        #[path = $path]
        mod $ident;

    )
        .unwrap()))
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(Symbol::intern("mod_path"),
                                  IdentTT(Box::new(expand_mod_path), None, false));
}
