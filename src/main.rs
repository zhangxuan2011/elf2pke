//! The main entry of this program.
use clap::Parser;
use colored::Colorize;
use elf::abi::{SHF_ALLOC, SHF_EXECINSTR};
use elf::section::SectionHeader;
use elf::{ElfBytes, endian::AnyEndian};
use elf2pke::{Args, Sections};
use log::{debug, error, info, warn};
use proka_exec::Builder;
use proka_exec::header::ExecMode;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

/// Main entry of this program.
fn main() {
    // Initialize basic environment
    elf2pke::logger::init();

    // Print basic info
    info!("{} v{}", "elf2pke".bold(), env!("CARGO_PKG_VERSION"));
    info!("Copyright (C) zhangxuan2011 2026. Licensed under GNU GPLv3.");
    info!("");

    // Run this main handler to get result.
    let result = handler();

    // Check out the result.
    if let Err(e) = result {
        error!(
            "An error has been occurred in this program. \n\nCaused by: {}",
            e.to_string().bold()
        );
    }
}

fn handler() -> Result<(), Box<dyn std::error::Error>> {
    // Parse args
    let arg = Args::parse();

    // Print passed arguments and basic informations
    info!("{}", "Basic information:".bold());
    info!("\tSource file: {}", arg.path.display().to_string().bold());
    info!(
        "\tDestination file: {}",
        arg.output_path
            .clone()
            .unwrap_or(PathBuf::from("out.pke"))
            .display()
            .to_string()
            .bold()
    );
    info!(
        "\tApp name: {}",
        arg.appname.clone().unwrap_or("appname".to_string()).bold()
    );
    info!(
        "\tAuthor: {}",
        arg.author.clone().unwrap_or("example".to_string()).bold()
    );
    info!("\tIs driver: {}", arg.is_driver.to_string().bold());
    info!("");

    /* Parse the origin ELF file */
    // Read and use crate to parse...
    info!("Parsing the origin ELF file...");
    let input_content = fs::read(arg.path)?;
    let input_elf = ElfBytes::<AnyEndian>::minimal_parse(&input_content)?;

    // Get each section headers and string table
    let mut sections = Vec::<Sections>::new();
    // Get the section header table alongside its string table
    let (shdrs_opt, strtab_opt) = input_elf.section_headers_with_strtab()?;
    let (shdrs, strtab) = (
        shdrs_opt.ok_or(Box::new(Error::new(
            std::io::ErrorKind::InvalidData,
            "no section headers",
        )))?,
        strtab_opt.ok_or(Box::new(Error::new(
            std::io::ErrorKind::InvalidData,
            "no strtab",
        )))?,
    );

    // Parse the shdrs and collect them into a map keyed on their zero-copied name
    let with_names: Vec<(&str, SectionHeader)> = shdrs
        .iter()
        .map(|shdr| {
            (
                strtab
                    .get(shdr.sh_name as usize)
                    .expect("Failed to get section name"),
                shdr,
            )
        })
        .collect();

    // Iterate each sections
    for (idx, (name, shdr)) in with_names.iter().enumerate() {
        let is_loadable = (shdr.sh_flags & SHF_ALLOC as u64) != 0;
        let is_executable = (shdr.sh_flags & SHF_EXECINSTR as u64) != 0;
        debug!(
            "Found a section indexed {idx} called {name}, is loadable: {is_loadable}, is executable: {is_executable}"
        );

        // According to the args, should we pass the unloadable sections?
        if arg.strip_unloadable && !is_loadable {
            debug!("This section is being stripped.");
            continue;
        }

        let section = Sections {
            section_name: name,
            is_loadable: (shdr.sh_flags & SHF_ALLOC as u64) != 0,
            is_executable: (shdr.sh_flags & SHF_EXECINSTR as u64) != 0,
            vaddr: shdr.sh_addr,
            size: shdr.sh_size,
            data: input_elf.section_data(&shdr)?.0,
        };
        sections.push(section);
    }

    /* Construct a builder to build PKE file */
    // Init builder
    info!("Generating PKE file...");
    let mut builder = Builder::new();

    // Set up builder
    let author = arg.author.clone().unwrap_or("example".to_string());
    let name = arg.appname.clone().unwrap_or("appname".to_string());
    builder.set_author(&author);
    builder.set_name(&name);
    builder.set_min([0, 1, 0]);
    builder.set_max([0, 1, 1]);

    // Decide the flags of the PKE file
    if arg.is_driver {
        builder.set_mode(ExecMode::CoreDrv);
    } else {
        builder.set_mode(ExecMode::UserApp);
    }

    // Append each section to the builder
    let entry = input_elf.ehdr.e_entry;
    let executable_sections = sections
        .iter()
        .enumerate()
        .filter(|(_, item)| item.is_executable)
        .filter(|(_, item)| item.vaddr <= entry && entry < item.vaddr + item.size)
        .map(|(idx, item)| (idx, entry - item.vaddr))
        .collect::<Vec<_>>();

    // Check: Is executable sections exist
    if executable_sections.is_empty() {
        return Err("No executable section that contains entry point found".into());
    }

    // Check: Is length of executable sections is 1
    if executable_sections.len() != 1 {
        warn!("More than one executable section found, only the first one will be used");
    }

    let (seg, off) = executable_sections[0];
    debug!(
        "Found executable section indexed {seg} at offset {off}"
    );

    for (idx, section) in sections.iter().enumerate() {
        if idx == seg {
            builder
                .append(
                    section.data,
                    &section.section_name,
                    section.is_loadable,
                    section.is_executable,
                    Some(off as u32),
                )
                .map_err(|e| {
                    format!(
                        "Failed to append executable section ({idx}) because of {:?}",
                        e
                    )
                })?;
        } else {
            builder
                .append(
                    section.data,
                    &section.section_name,
                    section.is_loadable,
                    section.is_executable,
                    None,
                )
                .map_err(|e| {
                    format!(
                        "Failed to append non-executable section ({idx}) because of {:?}",
                        e
                    )
                })?;
        }
    }

    // Finally build
    info!("Content generation successful");
    let pke_content = builder.build().map_err(|_| "Failed to build PKE content")?;

    // Then write to output dir
    let output_path = arg.output_path.unwrap_or(PathBuf::from("out.pke"));
    std::fs::write(output_path, pke_content)?;
    info!("PKE file generated successfully");

    Ok(())
}
