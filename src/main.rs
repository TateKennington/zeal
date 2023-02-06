use zeal::Compiler;

fn main() {
    let mut compiler = Compiler::default();
    let tokens = compiler.scan_line(
        r#"
        i := 1;
        while i <= 15:
            if i % 3 == 0 && i % 5 == 0:
                print "fizzbuzz"
            else if i % 5 == 0:
                print "buzz"
            else if i % 3 == 0:
                print "fizz"
            else:
                print i
            i = i + 1;
        "#,
    );
    println!("{tokens:?}");
    let expr = compiler.parse(tokens);
    println!("{expr:?}");
    let res = compiler.evaluate(expr);
    println!("{res:?}")
}

#[cfg(test)]
pub mod test_main {
    use zeal::{parser::Value, Compiler};

    #[test]
    pub fn interprets_fizzbuzz() {
        let mut compiler = Compiler::default();
        let tokens = compiler.scan_line(
            r#"
        i := 1;
        while i <= 15:
            if i % 3 == 0 && i % 5 == 0:
                print "fizzbuzz"
            else if i % 5 == 0:
                print "buzz"
            else if i % 3 == 0:
                print "fizz"
            else:
                print i
            i = i + 1;
        "#,
        );
        let expr = compiler.parse(tokens);
        let res = compiler.evaluate(expr);
    }

    #[test]
    pub fn interprets_variables() {
        let mut compiler = Compiler::default();
        let tokens = compiler.scan_line(
            r#"
        i := 1;
        i + 1 == 2;
        -i;
        i = i + 1;
        "#,
        );
        let expr = compiler.parse(tokens);
        let res = compiler.evaluate(expr);
        assert_eq!(
            res,
            [
                Value::Int(1),
                Value::Bool(true),
                Value::Int(-1),
                Value::Int(2)
            ]
        )
    }
}
