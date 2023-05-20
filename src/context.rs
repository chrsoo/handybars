use std::{borrow::Cow, collections::HashMap};

use crate::{
    parse::{self, Tokenize},
    value::Value,
    Variable,
};

/// Context for expanding templates
///
/// ```
/// # use handybars::*;
/// let mut ctx = Context::new();
/// ctx.define(Variable::single("a"), "b");
/// assert_eq!(ctx.render("{{ a }}"), Ok("b".to_owned()));
/// ```
#[derive(Debug, Default)]
pub struct Context<'a> {
    vars: HashMap<Variable<'a>, Cow<'a, str>>,
}
type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Parse(parse::Error),
    MissingVariable { name: Variable<'static> },
}
impl From<parse::Error> for Error {
    fn from(value: parse::Error) -> Self {
        Self::Parse(value)
    }
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    /// Map a variable to a value for template expansion
    ///
    pub fn define(&mut self, var: Variable<'a>, value: impl Into<Value<'a>>) -> &mut Self {
        match value.into() {
            Value::Object(obj) => {
                for (v, c) in obj.values {
                    self.define(var.clone().join(Variable::single(v)), c);
                }
            }
            Value::String(s) => {
                self.vars.insert(var, s);
            }
        }
        self
    }
    /// Render a template
    pub fn render<'b>(&self, input: &'b str) -> Result<String> {
        let mut output = String::new();
        for token in Tokenize::<'b>::new(input) {
            let token = token?;
            match token {
                parse::Token::Variable(v) => {
                    output.push_str(self.vars.get(&v).ok_or_else(|| Error::MissingVariable {
                        name: v.into_owned(),
                    })?)
                }
                parse::Token::Str(s) => {
                    output.push_str(s);
                }
            }
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Object;

    #[test]
    fn defining_an_object_variable_creates_path() {
        let mut ctx = Context::new();
        ctx.define(
            Variable::single("hello"),
            Object::new().with_property("world", Object::new().with_property("test", "val")),
        );
        assert_eq!(ctx.render("{{hello.world.test}}"), Ok("val".to_owned()));
    }

    #[test]
    fn context_can_register_single_variables() {
        let mut ctx = Context::new();
        ctx.define(
            Variable::single("hello"),
            Object::new().with_property("world", "sup"),
        );
    }
    #[test]
    fn context_renders_templates_using_defines() {
        let mut ctx = Context::new();
        ctx.define(Variable::single("t1"), "value");
        let expanded = ctx.render("{{ t1 }}").unwrap();
        assert_eq!(expanded, "value");
        assert_eq!(
            ctx.render("{{ notexist }}"),
            Err(Error::MissingVariable {
                name: Variable::single("notexist")
            }),
            "missing defines cause an error"
        );
    }
}
