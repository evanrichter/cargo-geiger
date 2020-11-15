mod table;

use crate::format::print_config::{OutputFormat, PrintConfig};
use crate::graph::Graph;

use super::find::find_unsafe;
use super::{package_metrics, ScanMode, ScanParameters};

use table::scan_forbid_to_table;

use crate::mapping::CargoMetadataParameters;
use cargo::core::PackageSet;
use cargo::{CliResult, Config};
use cargo_geiger_serde::{QuickReportEntry, QuickSafetyReport};

pub fn scan_forbid_unsafe(
    cargo_metadata_parameters: &CargoMetadataParameters,
    graph: &Graph,
    package_set: &PackageSet,
    root_package_id: cargo_metadata::PackageId,
    scan_parameters: &ScanParameters,
) -> CliResult {
    match scan_parameters.args.output_format {
        Some(output_format) => scan_forbid_to_report(
            cargo_metadata_parameters,
            scan_parameters.config,
            graph,
            output_format,
            package_set,
            scan_parameters.print_config,
            root_package_id,
        ),
        None => scan_forbid_to_table(
            cargo_metadata_parameters,
            scan_parameters.config,
            graph,
            package_set,
            scan_parameters.print_config,
            root_package_id,
        ),
    }
}

fn scan_forbid_to_report(
    cargo_metadata_parameters: &CargoMetadataParameters,
    config: &Config,
    graph: &Graph,
    output_format: OutputFormat,
    package_set: &PackageSet,
    print_config: &PrintConfig,
    root_package_id: cargo_metadata::PackageId,
) -> CliResult {
    let geiger_context = find_unsafe(
        cargo_metadata_parameters,
        config,
        ScanMode::EntryPointsOnly,
        package_set,
        print_config,
    )?;
    let mut report = QuickSafetyReport::default();
    for (package, package_metrics) in package_metrics(
        cargo_metadata_parameters,
        &geiger_context,
        graph,
        package_set,
        root_package_id,
    ) {
        let pack_metrics = match package_metrics {
            Some(m) => m,
            None => {
                report.packages_without_metrics.insert(package.id);
                continue;
            }
        };
        let forbids_unsafe = pack_metrics.rs_path_to_metrics.iter().all(
            |(_, rs_file_metrics_wrapper)| {
                rs_file_metrics_wrapper.metrics.forbids_unsafe
            },
        );
        let entry = QuickReportEntry {
            package,
            forbids_unsafe,
        };
        report.packages.insert(entry.package.id.clone(), entry);
    }
    let s = match output_format {
        OutputFormat::Json => serde_json::to_string(&report).unwrap(),
    };
    println!("{}", s);
    Ok(())
}
