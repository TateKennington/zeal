use zeal::Compiler;

fn main() {
    let mut compiler = Compiler::default();
    let tokens = compiler.scan_line(
        r#"
        i := 0;
        while i <= 15:
            if i % 3 == 0:
                print "fizz"
            else if i % 5 == 0:
                print "buzz"
            else if i % 3 == 0 && i % 5 == 0:
                print "fizzbuzz"
            else
                print i
        "#,
    );
    println!("{tokens:?}");
    let expr = compiler.parse(tokens);
    println!("{expr:?}");
    let res = compiler.evaluate(expr);
    println!("{res:?}")
}
