use x_diff::DiffConfig;

fn main() {
    let content = include_str!("../fixtures/test.json");
    let diff_config = DiffConfig::from_json(content).unwrap();
    println!("{:#?}", diff_config);
}
