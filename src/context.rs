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
#[derive(Debug, Default, Clone)]
pub struct Context<'a> {
    vars: HashMap<Variable<'a>, Cow<'a, str>>,
}
type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that may happen during rendering
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Forwarded from parsing
    Parse(parse::Error),
    /// Tried to expand a template variable that we don't have a value for
    MissingVariable(Variable<'static>),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(p) => f.write_fmt(format_args!("parse: {p}")),
            Error::MissingVariable(var) => {
                f.write_fmt(format_args!("missing variable in template: '{var}'"))
            }
        }
    }
}

impl From<parse::Error> for Error {
    fn from(value: parse::Error) -> Self {
        Self::Parse(value)
    }
}

impl<'a> Context<'a> {
    /// Create a new context with no variables defined
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
    /// Builder version of [`define`](Context::define)
    pub fn with_define(mut self, var: Variable<'a>, value: impl Into<Value<'a>>) -> Self {
        self.define(var, value);
        self
    }

    /// Render a template
    pub fn render<'b>(&self, input: &'b str) -> Result<String> {
        let mut output = String::new();
        for token in Tokenize::<'b>::new(input) {
            let token = token?;
            match token {
                parse::Token::Variable(v) => output.push_str(
                    self.vars
                        .get(&v)
                        .ok_or_else(|| Error::MissingVariable(v.into_owned()))?,
                ),
                parse::Token::Str(s) => {
                    output.push_str(s);
                }
            }
        }
        Ok(output)
    }
    /// Append another `Context`'s variables
    ///
    /// This operates in place, see [`merge`](Context::merge) for a streamable version
    ///
    /// ```
    /// # use handybars::{Context, Variable};
    /// let mut ctx = Context::new();
    /// ctx.append(Context::new().with_define(Variable::single("a"), "b"));
    /// assert_eq!(ctx.render("{{a}}"), Ok("b".to_owned()));
    /// ```
    pub fn append(&mut self, other: Self) -> &mut Self {
        self.vars.extend(other.vars.into_iter());
        self
    }
    /// Stream version of `append`
    pub fn merge(mut self, other: Self) -> Self {
        self.append(other);
        self
    }
}
impl<'a> Extend<(Variable<'a>, Value<'a>)> for Context<'a> {
    /// Extend a `Context` with an iterator of defines
    ///
    /// ```
    /// # use handybars::{Context, Variable, Value};
    /// let mut ctx = Context::new();
    /// ctx.extend([(Variable::single("a"), Value::String("b".into()))].into_iter());
    /// assert_eq!(ctx.render("{{a}}"), Ok("b".to_owned()));
    /// ```
    fn extend<T: IntoIterator<Item = (Variable<'a>, Value<'a>)>>(&mut self, iter: T) {
        for (var, val) in iter {
            self.define(var, val);
        }
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
            Err(Error::MissingVariable(Variable::single("notexist"))),
            "missing defines cause an error"
        );
    }
}
