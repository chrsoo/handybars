# Handybars

## Introduction

This is a small library for template expansion. The syntax is based on
handlebars, but it _only_ support expansion of variables. No `#if` or `#each`,
only `{{ variable }}`.

It has no dependencies and is designed to have a very simple API.


## Usage

```rust
use handybars::{Context, Variable};
let ctx = Context::new().with_define("hello".parse().unwrap(), "world");
assert_eq!(ctx.render("hello {{ hello }}"), Ok("hello world".to_owned()));
```
You can also define objects
```rust
# use handybars::{Context, Variable, Object};
# let mut ctx = Context::new().with_define("hello".parse().unwrap(), "world");
ctx.define("obj".parse().unwrap(), Object::new().with_property("a", "value"));
assert_eq!(ctx.render("object a: {{ obj.a }}"), Ok("object a: value".to_owned()));
```
You can even have nested objects
```rust
# use handybars::{Context, Variable, Object};
# let mut ctx = Context::new().with_define("hello".parse().unwrap(), "world");
ctx.define("obj".parse().unwrap(), Object::new().with_property("b", Object::new().with_property("c", "value")));
assert_eq!(ctx.render("nested: {{ obj.b.c }}"), Ok("nested: value".to_owned()));
```

Note that objects cannot be directly expanded:

```rust
use handybars::{Context, Variable, Object, Error};
let ctx = Context::new().with_define("world".parse().unwrap(), Object::new().with_property("a", "p1"));
assert_eq!(ctx.render("{{world}}"), Err(Error::MissingVariable(Variable::single("world"))));
```
