use facet_pretty::FacetPretty;

#[test]
fn test_arg_parse() {
    use facet::Facet;

    #[derive(Facet)]
    struct Args {
        #[facet(positional)]
        path: String,

        #[facet(named, short = 'v')]
        verbose: bool,

        #[facet(named, short = 'j')]
        concurrency: usize,
    }

    let args: Args =
        facet_args::from_slice(&["--path", "example.rs", "--verbose", "--concurrency", "14"]);
    eprintln!("args: {}", args.pretty());
}
