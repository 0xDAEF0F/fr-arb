use numfmt::*;

#[tokio::main]
async fn main() {
    let mut f = Formatter::new() // start with blank representation
        .separator(',')
        .unwrap()
        .prefix("AU$")
        .unwrap()
        .precision(Precision::Decimals(2));

    let a = f.fmt2(0.52);
    let b = f.fmt2(0.52);

    // assert_eq!(f.fmt2(0.52), "AU$0.52");
    // assert_eq!(f.fmt2(1234.567), "AU$1,234.56");
    // assert_eq!(f.fmt2(12345678900.0), "AU$12,345,678,900.0");
}
