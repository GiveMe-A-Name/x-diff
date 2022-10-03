use anyhow::{Ok, Result};
use x_diff::DiffConfig;

fn main() -> Result<()> {
    // 在程序编译前，先读取文件。将文件内容作为字符串包含进文件
    // 传入的路径时相当于当前程序文件的编译路径。
    let content = include_str!("../fixtures/test.yaml");
    let diff_config = DiffConfig::from_yaml(content)?;
    println!("{:#?}", diff_config.get_profile("rust"));
    Ok(())
}
