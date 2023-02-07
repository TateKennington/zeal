use std::io::stdout;

use zeal::Compiler;

fn main() {
    let mut stdout = stdout().lock();
    let mut compiler = Compiler::new(&mut stdout);
    let tokens = compiler.scan_line(
        r#"
        is_even := fn x -> x % 2 == 0;
        is_even 1;
        is_even 2;
        (fn x -> x % 2 == 0) 3;
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
    use std::io::stdout;
    use zeal::{parser::Value, Compiler};

    #[test]
    pub fn interprets_fizzbuzz() {
        let mut output = vec![];
        let mut compiler = Compiler::new(&mut output);
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
        compiler.evaluate(expr);

        let output = String::from_utf8_lossy(&output);
        assert_eq!(
            output,
            "[Int(1)]\n[Int(2)]\n[String(\"fizz\")]\n[Int(4)]\n[String(\"buzz\")]\n[String(\"fizz\")]\n[Int(7)]\n[Int(8)]\n[String(\"fizz\")]\n[String(\"buzz\")]\n[Int(11)]\n[String(\"fizz\")]\n[Int(13)]\n[Int(14)]\n[String(\"fizzbuzz\")]\n"
        )
    }

    #[test]
    pub fn interprets_scopes() {
        let mut output = vec![];
        let mut compiler = Compiler::new(&mut output);
        let tokens = compiler.scan_line(
            r#"
                a := 0;
                if true:
                    print a;
                    a = a + 1;
                    print a;
                    a := 10;
                    print a;
                    if true:
                        print a;
                        a = a + 1;
                        print a;
                        a := 100;
                        print a;
                    print a;
                print a;
            "#,
        );
        let expr = compiler.parse(tokens);
        compiler.evaluate(expr);
        let output = String::from_utf8_lossy(&output);

        assert_eq!(
            output, 
            "[Int(0)]\n[Int(1)]\n[Int(10)]\n[Int(10)]\n[Int(11)]\n[Int(100)]\n[Int(11)]\n[Int(1)]\n"
        )
    }

    #[test]
    pub fn interprets_variables() {
        let mut stdout = stdout();
        let mut compiler = Compiler::new(&mut stdout);
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
