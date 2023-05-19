use std::{borrow::Cow, collections::BTreeMap};

pub enum Value {
    String(Cow<'static, str>),
}
