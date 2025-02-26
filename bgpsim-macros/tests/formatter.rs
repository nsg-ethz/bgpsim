use std::collections::BTreeMap;

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

#[test]
fn non_nesting() {
    #[derive(NetworkFormatter)]
    enum Foo {
        Single,
        Unnamed(usize, usize),
        NoNest {
            foo: usize,
            #[formatter(multiline = "fmt")]
            bar: Vec<usize>,
        },
        Nest {
            foo: usize,
            #[formatter(multiline = "fmt_multiline")]
            bar: Vec<usize>,
        },
    }

    let x = Foo::Single;
    let y = Foo::Unnamed(1, 2);
    let z = Foo::NoNest {
        foo: 1,
        bar: vec![2, 3, 4],
    };
    let w = Foo::Nest {
        foo: 1,
        bar: vec![2, 3, 4],
    };
    let net = Net::default();

    assert_eq!(x.fmt(&net), "Single");
    assert_eq!(y.fmt(&net), "Unnamed(1, 2)");
    assert_eq!(z.fmt(&net), "NoNest { foo: 1, bar: [2, 3, 4] }");
    assert_eq!(w.fmt(&net), "Nest { foo: 1, bar: [2, 3, 4] }");

    assert_eq!(x.fmt_multiline(&net), "Single");
    assert_eq!(y.fmt_multiline(&net), "Unnamed(\n  1,\n  2\n)");
    assert_eq!(
        z.fmt_multiline(&net),
        "NoNest {\n  foo: 1,\n  bar: [2, 3, 4]\n}"
    );
    assert_eq!(
        w.fmt_multiline(&net),
        "Nest {\n  foo: 1,\n  bar: [\n    2,\n    3,\n    4\n  ]\n}"
    );
}

#[test]
fn path_attributes() {
    #[derive(NetworkFormatter)]
    enum Foo {
        Path(#[formatter(fmt = "fmt_path")] Vec<usize>),
        PathOptions(
            #[formatter(fmt = "fmt_path_options", multiline = "fmt_path_multiline")]
            Vec<Vec<usize>>,
        ),
    }

    let x = Foo::Path(vec![1, 2, 3]);
    let y = Foo::PathOptions(vec![vec![1, 2], vec![3, 4, 5]]);
    let net = Net::default();

    assert_eq!(x.fmt(&net), "Path(1 -> 2 -> 3)");
    assert_eq!(y.fmt(&net), "PathOptions(1 -> 2 | 3 -> 4 -> 5)");
    assert_eq!(x.fmt_multiline(&net), "Path(1 -> 2 -> 3)");
    assert_eq!(
        y.fmt_multiline(&net),
        "PathOptions({\n  1 -> 2,\n  3 -> 4 -> 5\n})"
    );
}

#[derive(NetworkFormatter)]
struct Ext {
    a: usize,
    b: usize,
}

impl<'n, P: Prefix, Q, Ospf: OspfImpl> NetworkFormatterExt<'n, P, Q, Ospf> for Ext {
    fn fmt_ext(&self, _net: &'n Network<P, Q, Ospf>) -> String {
        format!("({}--{})", self.a, self.b)
    }
}

#[test]
fn single_line_attributes() {
    #[derive(NetworkFormatter)]
    enum Foo {
        Set(#[formatter(fmt = "fmt_set")] Vec<usize>),
        Map(#[formatter(fmt = "fmt_map")] BTreeMap<usize, usize>),
        List(#[formatter(fmt = "fmt_list")] Vec<usize>),
        Path(#[formatter(fmt = "fmt_path")] Vec<usize>),
        PathOptions(#[formatter(fmt = "fmt_path_options")] Vec<Vec<usize>>),
        PathSet(#[formatter(fmt = "fmt_path_set")] Vec<Vec<usize>>),
        Ext(#[formatter(fmt = "fmt_ext")] Ext),
    }

    let net = Net::default();

    let v1 = Foo::Set(vec![1, 2]);
    let v2 = Foo::Map([(1, 10), (2, 20)].into_iter().collect());
    let v3 = Foo::List(vec![1, 2]);
    let v4 = Foo::Path(vec![1, 2]);
    let v5 = Foo::PathOptions(vec![vec![1, 2], vec![3, 4]]);
    let v6 = Foo::PathSet(vec![vec![1, 2], vec![3, 4]]);
    let v7 = Foo::Ext(Ext { a: 1, b: 2 });

    assert_eq!(v1.fmt(&net), "Set({1, 2})");
    assert_eq!(v2.fmt(&net), "Map({1: 10, 2: 20})");
    assert_eq!(v3.fmt(&net), "List([1, 2])");
    assert_eq!(v4.fmt(&net), "Path(1 -> 2)");
    assert_eq!(v5.fmt(&net), "PathOptions(1 -> 2 | 3 -> 4)");
    assert_eq!(v6.fmt(&net), "PathSet({1 -> 2, 3 -> 4})");
    assert_eq!(v7.fmt(&net), "Ext((1--2))");

    assert_eq!(v1.fmt_multiline(&net), "Set({\n  1,\n  2\n})");
    assert_eq!(v2.fmt_multiline(&net), "Map({\n  1: 10,\n  2: 20\n})");
    assert_eq!(v3.fmt_multiline(&net), "List([\n  1,\n  2\n])");
    assert_eq!(v4.fmt_multiline(&net), "Path(1 -> 2)");
    assert_eq!(v5.fmt_multiline(&net), "PathOptions(1 -> 2 | 3 -> 4)");
    assert_eq!(v6.fmt_multiline(&net), "PathSet({\n  1 -> 2,\n  3 -> 4\n})");
    assert_eq!(v7.fmt_multiline(&net), "Ext((1--2))");
}

#[test]
fn multi_line_attributes() {
    #[derive(NetworkFormatter)]
    enum Foo {
        Set(#[formatter(multiline = "fmt_set_multiline")] Vec<usize>),
        Map(#[formatter(multiline = "fmt_map_multiline")] BTreeMap<usize, usize>),
        List(#[formatter(multiline = "fmt_list_multiline")] Vec<usize>),
        PathSet(#[formatter(multiline = "fmt_path_multiline")] Vec<Vec<usize>>),
    }

    let net = Net::default();

    let v1 = Foo::Set(vec![1, 2]);
    let v2 = Foo::Map([(1, 10), (2, 20)].into_iter().collect());
    let v3 = Foo::List(vec![1, 2]);
    let v6 = Foo::PathSet(vec![vec![1, 2], vec![3, 4]]);

    assert_eq!(v1.fmt(&net), "Set([1, 2])");
    assert_eq!(v2.fmt(&net), "Map({1: 10, 2: 20})");
    assert_eq!(v3.fmt(&net), "List([1, 2])");
    assert_eq!(v6.fmt(&net), "PathSet([[1, 2], [3, 4]])");

    assert_eq!(v1.fmt_multiline(&net), "Set({\n  1,\n  2\n})");
    assert_eq!(v2.fmt_multiline(&net), "Map({\n  1: 10,\n  2: 20\n})");
    assert_eq!(v3.fmt_multiline(&net), "List([\n  1,\n  2\n])");
    assert_eq!(v6.fmt_multiline(&net), "PathSet({\n  1 -> 2,\n  3 -> 4\n})");
}

#[test]
fn custom_attribute() {
    #[derive(NetworkFormatter)]
    enum Foo {
        Bar(#[formatter(fmt = fmt_bar)] usize),
        Baz(#[formatter(fmt = fmt_baz)] usize),
        MultiBar(#[formatter(multiline = fmt_multi_bar)] usize),
        MultiBaz(#[formatter(multiline = fmt_multi_baz)] usize),
    }

    let net = Net::default();

    let v1 = Foo::Bar(1);
    let v2 = Foo::Baz(1);
    let v3 = Foo::MultiBar(1);
    let v4 = Foo::MultiBaz(1);

    assert_eq!(v1.fmt(&net), "Bar(bar)");
    assert_eq!(v2.fmt(&net), "Baz(baz)");
    assert_eq!(v3.fmt(&net), "MultiBar(1)");
    assert_eq!(v4.fmt(&net), "MultiBaz(1)");

    assert_eq!(v1.fmt_multiline(&net), "Bar(bar)");
    assert_eq!(v2.fmt_multiline(&net), "Baz(baz)");
    assert_eq!(v3.fmt_multiline(&net), "MultiBar(multibar)");
    assert_eq!(v4.fmt_multiline(&net), "MultiBaz(multibaz)");
}

#[test]
fn skip_attribute() {
    #[derive(NetworkFormatter)]
    enum Foo {
        UnnamedSkip1(#[formatter(skip)] usize, usize),
        UnnamedSkip2(usize, #[formatter(skip)] usize),
        UnnamedSkipAll(#[formatter(skip)] usize, #[formatter(skip)] usize),
        NamedSkip1 {
            #[formatter(skip)]
            a: usize,
            b: usize,
        },
        NamedSkip2 {
            a: usize,
            #[formatter(skip)]
            b: usize,
        },
        NamedSkipAll {
            #[formatter(skip)]
            a: usize,
            #[formatter(skip)]
            b: usize,
        },
    }

    let net = Net::default();

    let v1 = Foo::UnnamedSkip1(1, 2);
    let v2 = Foo::UnnamedSkip2(1, 2);
    let v3 = Foo::UnnamedSkipAll(1, 2);
    let v4 = Foo::NamedSkip1 { a: 1, b: 2 };
    let v5 = Foo::NamedSkip2 { a: 1, b: 2 };
    let v6 = Foo::NamedSkipAll { a: 1, b: 2 };

    assert_eq!(v1.fmt(&net), "UnnamedSkip1(2)");
    assert_eq!(v2.fmt(&net), "UnnamedSkip2(1)");
    assert_eq!(v3.fmt(&net), "UnnamedSkipAll()");
    assert_eq!(v4.fmt(&net), "NamedSkip1 { b: 2 }");
    assert_eq!(v5.fmt(&net), "NamedSkip2 { a: 1 }");
    assert_eq!(v6.fmt(&net), "NamedSkipAll {}");

    assert_eq!(v1.fmt_multiline(&net), "UnnamedSkip1(2)");
    assert_eq!(v2.fmt_multiline(&net), "UnnamedSkip2(1)");
    assert_eq!(v3.fmt_multiline(&net), "UnnamedSkipAll()");
    assert_eq!(v4.fmt_multiline(&net), "NamedSkip1 {\n  b: 2\n}");
    assert_eq!(v5.fmt_multiline(&net), "NamedSkip2 {\n  a: 1\n}");
    assert_eq!(v6.fmt_multiline(&net), "NamedSkipAll {}");
}

fn fmt_bar<P: Prefix, Q, Ospf: OspfImpl>(_: &usize, _: &Network<P, Q, Ospf>) -> String {
    String::from("bar")
}

fn fmt_baz<P: Prefix, Q, Ospf: OspfImpl>(_: &usize, _: &Network<P, Q, Ospf>) -> String {
    String::from("baz")
}

fn fmt_multi_bar<P: Prefix, Q, Ospf: OspfImpl>(
    _: &usize,
    _: &Network<P, Q, Ospf>,
    _: usize,
) -> String {
    String::from("multibar")
}

fn fmt_multi_baz<P: Prefix, Q, Ospf: OspfImpl>(
    _: &usize,
    _: &Network<P, Q, Ospf>,
    _: usize,
) -> String {
    String::from("multibaz")
}
