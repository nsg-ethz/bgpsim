use bgpsim::prelude::*;
use bgpsim_macros::NetworkFormatter;

type Net = Network<SimplePrefix, BasicEventQueue<SimplePrefix>, GlobalOspf>;

#[test]
fn unit_struct() {
    #[derive(NetworkFormatter)]
    struct Foo;

    let x = Foo;
    let net = Net::default();

    assert_eq!(x.fmt(&net), "Foo");
    assert_eq!(x.fmt_multiline(&net), "Foo");
}

#[test]
fn unnamed_struct() {
    #[derive(NetworkFormatter)]
    struct Foo(usize, usize);

    let x = Foo(1, 2);
    let net = Net::default();

    assert_eq!(x.fmt(&net), "Foo(1, 2)");
    assert_eq!(x.fmt_multiline(&net), "Foo(\n  1,\n  2\n)");
}

#[test]
fn named_struct() {
    #[derive(NetworkFormatter)]
    struct Foo {
        bar: usize,
        baz: usize,
    }

    let x = Foo { bar: 1, baz: 2 };
    let net = Net::default();

    assert_eq!(x.fmt(&net), "Foo { bar: 1, baz: 2 }");
    assert_eq!(x.fmt_multiline(&net), "Foo {\n  bar: 1,\n  baz: 2\n}");
}

#[test]
fn named_struct_with_generics() {
    #[derive(NetworkFormatter)]
    struct Foo<P: Prefix> {
        bar: usize,
        baz: P,
    }

    let x = Foo {
        bar: 1,
        baz: prefix!("10.0.0.0/8" as Ipv4Prefix),
    };
    let net = Net::default();

    assert_eq!(x.fmt(&net), "Foo { bar: 1, baz: 10.0.0.0/8 }");
    assert_eq!(
        x.fmt_multiline(&net),
        "Foo {\n  bar: 1,\n  baz: 10.0.0.0/8\n}"
    );
}

#[test]
fn union() {
    #[derive(NetworkFormatter)]
    enum Foo<P: Prefix> {
        Single,
        Unnamed(usize, P),
        Named { foo: usize, bar: P },
    }

    let x = Foo::Single::<Ipv4Prefix>;
    let y = Foo::Unnamed(1, prefix!("10.0.0.0/8" as Ipv4Prefix));
    let z = Foo::Named {
        foo: 1,
        bar: prefix!("10.0.0.0/8" as Ipv4Prefix),
    };
    let net = Net::default();

    assert_eq!(x.fmt(&net), "Single");
    assert_eq!(y.fmt(&net), "Unnamed(1, 10.0.0.0/8)");
    assert_eq!(z.fmt(&net), "Named { foo: 1, bar: 10.0.0.0/8 }");

    assert_eq!(x.fmt_multiline(&net), "Single");
    assert_eq!(y.fmt_multiline(&net), "Unnamed(\n  1,\n  10.0.0.0/8\n)");
    assert_eq!(
        z.fmt_multiline(&net),
        "Named {\n  foo: 1,\n  bar: 10.0.0.0/8\n}"
    );
}
