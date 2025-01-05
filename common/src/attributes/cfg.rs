use anyhow::Result;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, Lit, MetaList, Token};

mod kw {
    syn::custom_keyword!(all);
    syn::custom_keyword!(any);
    syn::custom_keyword!(not);
}
pub fn handle_cfg(meta_list: &MetaList) -> Result<ConfigurationPredicate> {
    Ok(syn::parse2(meta_list.tokens.clone())?)
}

pub trait CfgEval {
    fn eval(&self, cfg_test: bool) -> bool;
}

#[derive(Debug)]
pub enum ConfigurationPredicate {
    Option(Box<ConfigurationOption>),
    All(Box<ConfigurationAll>),
    Any(Box<ConfigurationAny>),
    Not(Box<ConfigurationNot>),
}

impl Parse for ConfigurationPredicate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(kw::all) {
            Ok(ConfigurationPredicate::All(input.parse()?))
        } else if input.peek(kw::any) {
            Ok(ConfigurationPredicate::Any(input.parse()?))
        } else if input.peek(kw::not) {
            Ok(ConfigurationPredicate::Not(input.parse()?))
        } else {
            Ok(ConfigurationPredicate::Option(input.parse()?))
        }
    }
}

impl CfgEval for ConfigurationPredicate {
    fn eval(&self, cfg_test: bool) -> bool {
        match self {
            ConfigurationPredicate::Option(option) => option.eval(cfg_test),
            ConfigurationPredicate::All(all) => all.eval(cfg_test),
            ConfigurationPredicate::Any(any) => any.eval(cfg_test),
            ConfigurationPredicate::Not(not) => not.eval(cfg_test),
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct ConfigurationOption {
    identifier: Ident,
    eq_token: Option<Token![=]>,
    string: Option<Lit>,
}

impl Parse for ConfigurationOption {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let identifier = input.parse()?;
        if input.peek(Token![=]) {
            let eq_token = Some(input.parse()?);
            let string = Some(input.parse()?);
            Ok(ConfigurationOption {
                identifier,
                eq_token,
                string,
            })
        } else {
            Ok(ConfigurationOption {
                identifier,
                eq_token: None,
                string: None,
            })
        }
    }
}

impl CfgEval for ConfigurationOption {
    fn eval(&self, cfg_test: bool) -> bool {
        if self.identifier.to_string() == "test" {
            return cfg_test;
        }

        if let Some(Lit::Str(lit_str)) = &self.string {
            std::env::var(format!(
                "CARGO_CFG_{}",
                self.identifier.to_string().to_uppercase().replace("-", "_")
            ))
            .unwrap()
            .split(",")
            .any(|s| s == lit_str.value())
        } else {
            std::env::var(format!(
                "CARGO_CFG_{}",
                self.identifier.to_string().to_uppercase().replace("-", "_")
            ))
            .is_ok()
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct ConfigurationAll {
    all: kw::all,
    paren_token: syn::token::Paren,
    list: Punctuated<ConfigurationPredicate, Token![,]>,
}
impl Parse for ConfigurationAll {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let all = input.parse()?;
        let content;
        let paren_token = parenthesized!(content in input);
        let list = content.parse_terminated(ConfigurationPredicate::parse, Token![,])?;
        Ok(ConfigurationAll {
            all,
            paren_token,
            list,
        })
    }
}
impl CfgEval for ConfigurationAll {
    fn eval(&self, cfg_test: bool) -> bool {
        self.list.iter().all(|predicate| predicate.eval(cfg_test))
    }
}
#[derive(Debug)]
#[allow(unused)]
pub struct ConfigurationAny {
    any: Ident,
    paren_token: syn::token::Paren,
    list: Punctuated<ConfigurationPredicate, Token![,]>,
}
impl Parse for ConfigurationAny {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let any = input.parse()?;
        let content;
        let paren_token = parenthesized!(content in input);
        let list = content.parse_terminated(ConfigurationPredicate::parse, Token![,])?;
        Ok(ConfigurationAny {
            any,
            paren_token,
            list,
        })
    }
}

impl CfgEval for ConfigurationAny {
    fn eval(&self, cfg_test: bool) -> bool {
        self.list.iter().any(|predicate| predicate.eval(cfg_test))
    }
}
#[derive(Debug)]
#[allow(unused)]
pub struct ConfigurationNot {
    not: Ident,
    paren_token: syn::token::Paren,
    predicate: ConfigurationPredicate,
}
impl Parse for ConfigurationNot {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let not = input.parse()?;
        let content;
        let paren_token = parenthesized!(content in input);
        let predicate = content.parse()?;
        Ok(ConfigurationNot {
            not,
            paren_token,
            predicate,
        })
    }
}
impl CfgEval for ConfigurationNot {
    fn eval(&self, cfg_test: bool) -> bool {
        !self.predicate.eval(cfg_test)
    }
}
