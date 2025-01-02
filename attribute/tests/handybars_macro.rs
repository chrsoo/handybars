#[cfg(test)]
mod macro_tests {
    use handybars::{Context, Variable};
    use handybars_attribute::handybars_value;

    #[test]
    fn test() {
        let v = TestObject {
            prop_0: "p0_val".to_owned(),
            prop_1: 1,
            prop_2: "p2_val",
            prop_3: StructVal {
                field_1: 30,
                field_2: "f32_val".to_owned(),
                field_3: "f33_val",
                field_4: SimpleEnumProp::A,
            },
            prop_4: SimpleEnumProp::B,
            // prop_5: ComplexEnumProp::Var2(SimpleEnumProp::C)
        };
        let c = Context::new().with_define(Variable::single("obj"), v);
        assert_eq!("1", c.render("{{ obj.prop_1 }}").unwrap());
        assert_eq!("A", c.render("{{ obj.prop_3.field_4 }}").unwrap());
        assert_eq!("f33_val", c.render("{{ obj.prop_3.field_3 }}").unwrap());
    }

    #[handybars_value]
    struct TestObject<'a> {
        prop_0: String,
        prop_1: u64,
        prop_2: &'a str,
        prop_3: StructVal<'a>,
        prop_4: SimpleEnumProp,
        // prop_5: ComplexEnumProp<'a>,
    }

    #[handybars_value]
    enum SimpleEnumProp {
        A,
        B,
        // C,
    }

    // #[handybar_value]
    // enum ComplexEnumProp<'a> {
    //     Var1,
    //     Var2(SimpleEnumProp),
    //     Var3(String),
    //     Var4(StructVal<'a>),
    // }

    #[handybars_value]
    struct StructVal<'a> {
        field_1: u16,
        field_2: String,
        field_3: &'a str,
        field_4: SimpleEnumProp,
    }
}
