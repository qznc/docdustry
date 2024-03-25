use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn generate_random_markdown_files(output_dir: &Path, num_dirs: usize, files_per_dir: usize) {
    for dir_num in 0..num_dirs {
        let dir_name = format!("dir{}", dir_num);
        let dir_path = (output_dir).join(&dir_name);
        fs::create_dir_all(&dir_path).expect("Failed to create directory");

        for file_num in 0..files_per_dir {
            let file_name = format!("file{}.md", file_num);
            let file_path = dir_path.join(&file_name);
            let mut file = File::create(&file_path).expect("Failed to create file");

            write!(file, "# Random Markdown {} {}\n", dir_num, file_num)
                .expect("Failed to write to file");
            file.write(MARKDOWN).expect("Failed to write to file");
        }
    }
}

const MARKDOWN: &[u8] = r#"
Some text here

Multiple paragraphs even

> Also a quote

```
and code
```

Ending here.
"#
.as_bytes();
