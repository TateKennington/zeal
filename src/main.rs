use zeal::Compiler;

fn main() {
    let mut compiler = Compiler::default();
    let tokens = compiler.scan_line(
        r#"
        a.map fn i -> 
          b.map fn c -> 
            c
        |> join "\n"
        |> prin;
        "#,
    );
    println!("{tokens:?}");
    let expr = compiler.parse(tokens);
    println!("{expr:?}");
    // let res = compiler.evaluate(expr);
    // println!("{res:?}")
}
