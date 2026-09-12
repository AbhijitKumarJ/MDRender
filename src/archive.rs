use crate::config::ArchiveArgs;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Determines the next sequence number by finding the maximum number
/// used in existing subfolders of archive_dir.
pub fn determine_next_sequence(archive_dir: &Path) -> u32 {
    if !archive_dir.exists() {
        return 1;
    }

    let mut max_seq = 0;

    if let Ok(entries) = fs::read_dir(archive_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(num) = extract_leading_number(name) {
                        if num > max_seq {
                            max_seq = num;
                        }
                    }
                }
            }
        }
    }

    max_seq + 1
}

/// Extracts a leading integer from names like "001_Tutorials", "02_Basics", "3_Guide"
pub fn extract_leading_number(name: &str) -> Option<u32> {
    let digits: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
        digits.parse::<u32>().ok()
    } else {
        None
    }
}

/// Formats the target archive folder name, e.g. "003_GraphTutorials".
/// If the preferred name already starts with a number prefix (e.g. "003_GraphTutorials"),
/// it strips the prefix to prevent double prefixing like "003_003_GraphTutorials".
pub fn format_archive_folder_name(preferred_name: &str, sequence: u32) -> String {
    let clean_name = if let Some(idx) = preferred_name.find('_') {
        let prefix = &preferred_name[..idx];
        if prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty() {
            preferred_name[idx + 1..].trim()
        } else {
            preferred_name.trim()
        }
    } else {
        preferred_name.trim()
    };

    let sanitized: String = clean_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();

    let final_name = if sanitized.is_empty() {
        "Archive".to_string()
    } else {
        sanitized
    };

    format!("{:03}_{}", sequence, final_name)
}

/// Recursively copies directory `src` to `dest`
pub fn copy_dir_all(src: &Path, dest: &Path) -> Result<usize> {
    let mut copied_count = 0;
    fs::create_dir_all(dest)
        .with_context(|| format!("Failed to create directory {}", dest.display()))?;

    for entry in WalkDir::new(src).follow_links(true) {
        let entry = entry?;
        let entry_path = entry.path();
        let rel_path = entry_path.strip_prefix(src)?;
        let target_path = dest.join(rel_path);

        if entry_path.is_dir() {
            fs::create_dir_all(&target_path)?;
        } else if entry_path.is_file() {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry_path, &target_path)
                .with_context(|| format!("Failed to copy {} to {}", entry_path.display(), target_path.display()))?;
            copied_count += 1;
        }
    }

    Ok(copied_count)
}

/// Cleans all contents inside `dir`, keeping the directory itself.
pub fn clean_directory_contents(dir: &Path) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }

    let mut removed_count = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("Failed to remove directory {}", path.display()))?;
        } else {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove file {}", path.display()))?;
        }
        removed_count += 1;
    }

    Ok(removed_count)
}

/// Cleans the output directory, keeping the reusable `assets/` subfolder.
pub fn clean_output_preserving_assets(output_dir: &Path) -> Result<usize> {
    if !output_dir.exists() {
        return Ok(0);
    }

    let mut removed_count = 0;
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Check if this is the reusable "assets" directory
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "assets" {
                continue; // Preserve reusable assets!
            }
        }

        if path.is_dir() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("Failed to remove directory {}", path.display()))?;
        } else {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove file {}", path.display()))?;
        }
        removed_count += 1;
    }

    Ok(removed_count)
}

/// Executes the archive command.
pub fn run_archive(args: &ArchiveArgs) -> Result<()> {
    println!("\n📦 MDRender Archive Process");
    println!("  Archive directory: {}", args.archive_dir.display());
    println!("  Input directory:   {}", args.input.display());
    println!("  Output directory:  {}", args.output.display());

    if !args.input.exists() && !args.output.exists() {
        anyhow::bail!("Neither input directory '{}' nor output directory '{}' exists to archive.",
            args.input.display(), args.output.display());
    }

    // 1. Determine next sequence number and destination folder
    let next_seq = determine_next_sequence(&args.archive_dir);
    let folder_name = format_archive_folder_name(&args.name, next_seq);
    let dest_dir = args.archive_dir.join(&folder_name);

    println!("  Target archive:    {}\n", dest_dir.display());

    // 2. Copy input folder
    if args.input.exists() {
        let input_dest = dest_dir.join("input");
        let count = copy_dir_all(&args.input, &input_dest)?;
        println!("  ✓ Copied input folder ({} files) -> {}", count, input_dest.display());
    } else {
        println!("  • Input folder {} does not exist, skipping input copy", args.input.display());
    }

    // 3. Copy output folder
    if args.output.exists() {
        let output_dest = dest_dir.join("output");
        let count = copy_dir_all(&args.output, &output_dest)?;
        println!("  ✓ Copied output folder ({} files) -> {}", count, output_dest.display());
    } else {
        println!("  • Output folder {} does not exist, skipping output copy", args.output.display());
    }

    // 4. Clean input and output folders if not disabled
    if !args.no_clean {
        println!("\n  🧹 Cleaning input and output folders...");

        let input_cleaned = clean_directory_contents(&args.input)?;
        println!("  ✓ Emptied input folder ({} items removed)", input_cleaned);

        let output_cleaned = clean_output_preserving_assets(&args.output)?;
        println!("  ✓ Cleaned output folder ({} items removed, preserved reusable `assets/`)", output_cleaned);
    } else {
        println!("\n  ℹ️ Cleanup skipped (--no-clean was specified)");
    }

    println!("\n✨ Successfully archived to `{}`!\n", dest_dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_leading_number() {
        assert_eq!(extract_leading_number("001_Tutorials"), Some(1));
        assert_eq!(extract_leading_number("042_Advanced"), Some(42));
        assert_eq!(extract_leading_number("123"), Some(123));
        assert_eq!(extract_leading_number("GraphTutorials"), None);
        assert_eq!(extract_leading_number("no_numbers_here"), None);
    }

    #[test]
    fn test_format_archive_folder_name() {
        assert_eq!(format_archive_folder_name("GraphTutorials", 3), "003_GraphTutorials");
        assert_eq!(format_archive_folder_name("001_GraphTutorials", 5), "005_GraphTutorials");
        assert_eq!(format_archive_folder_name("My Notes & Docs", 1), "001_My_Notes___Docs");
    }
}
