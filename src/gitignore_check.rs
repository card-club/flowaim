use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn add_flowaim_config_filename_if_needed(
    flowaim_config_filename: &str,
) -> Result<(), std::io::Error> {
    let gitignore_path = Path::new(".gitignore");

    let mut lines = if gitignore_path.exists() {
        // If the .gitignore file exists, read its contents into a Vec
        let file = std::fs::File::open(&gitignore_path)?;
        let reader = BufReader::new(file);
        reader.lines().collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    if !lines.contains(&flowaim_config_filename.to_string()) {
        // If the filename is not already in the .gitignore file, add it
        lines.push(flowaim_config_filename.to_string());

        // Open the .gitignore file for writing. This will create it if it doesn't exist.
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&gitignore_path)?;

        // Write the lines back to the .gitignore file, one per line
        for line in lines {
            writeln!(file, "{}", line)?;
        }
    }

    Ok(())
}
