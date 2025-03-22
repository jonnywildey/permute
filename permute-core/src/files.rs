use std::{fs, io, path::Path};

const SUB_DIR_PREFIX: &str = "permutes";

pub fn get_output_run(output_dir: String) -> io::Result<String> {
    let output_path = Path::new(output_dir.as_str());
    let mut run_max: i32 = 0;

    for entry in fs::read_dir(output_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let dir = path
                .file_name()
                .expect("error parsing output directory")
                .to_str()
                .unwrap();
            let n = dir.find(SUB_DIR_PREFIX);
            let n: i32 = match n {
                None => 0,
                Some(i) => {
                    let s = dir.get((i + SUB_DIR_PREFIX.len())..dir.len());
                    s.unwrap_or("").parse::<i32>().unwrap_or(0)
                }
            };
            if n > run_max {
                run_max = n;
            }
        }
    }
    let sub_dir = output_path.join(format!("{}{}", SUB_DIR_PREFIX, run_max + 1));
    fs::create_dir(sub_dir.clone())?;
    let sub_dir = sub_dir.to_str().unwrap_or(output_dir.as_str());

    Ok(sub_dir.to_string())
}

pub fn generate_file_name(
    file: String,
    output: String,
    permutation_count: usize,
    output_file_as_wav: bool,
) -> String {
    let mut dir_path = Path::new(&output).canonicalize().unwrap();
    let file_path = Path::new(&file);
    let file_stem = file_path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");
    let extension = match output_file_as_wav {
        true => "wav",
        false => file_path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or(""),
    };
    let new_filename = [file_stem, &permutation_count.to_string(), ".", extension].concat();

    dir_path.push(new_filename);
    dir_path.into_os_string().into_string().unwrap()
}
