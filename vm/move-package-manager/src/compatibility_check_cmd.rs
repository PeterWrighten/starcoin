// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::releasement::module;
use clap::Parser;
use itertools::Itertools;
use move_binary_format::CompiledModule;
use move_cli::sandbox::utils::PackageContext;
use move_cli::Move;
use move_core_types::resolver::ModuleResolver;
use starcoin_config::BuiltinNetworkID;
use starcoin_move_compiler::check_compiled_module_compat;
use starcoin_transactional_test_harness::remote_state::RemoteStateView;

#[derive(Parser)]
pub struct CompatibilityCheckCommand {
    #[clap(name = "rpc", long)]
    /// use remote starcoin rpc as initial state.
    rpc: Option<String>,
    #[clap(long = "block-number", requires("rpc"))]
    /// block number to read state from. default to latest block number.
    block_number: Option<u64>,

    #[clap(long = "network", short, conflicts_with("rpc"))]
    /// genesis with the network
    network: Option<BuiltinNetworkID>,
}

pub fn handle_compatibility_check(
    move_args: &Move,
    cmd: CompatibilityCheckCommand,
) -> anyhow::Result<()> {
    let pkg_ctx = PackageContext::new(&move_args.package_path, &move_args.build_config)?;
    let pkg = pkg_ctx.package();

    let rpc = cmd.rpc.unwrap_or_else(|| {
        format!(
            "http://{}:{}",
            cmd.network
                .unwrap_or(BuiltinNetworkID::Main)
                .boot_nodes_domain(),
            9850
        )
    });

    let remote_view = RemoteStateView::from_url(&rpc, cmd.block_number)?;

    let mut incompatible_module_ids = vec![];
    for m in pkg.modules()? {
        let m = module(&m.unit)?;
        let old_module = remote_view
            .get_module(&m.self_id())
            .map_err(|e| e.into_vm_status())?;
        if let Some(old) = old_module {
            let old_module = CompiledModule::deserialize(&old)?;
            let compatibility = check_compiled_module_compat(&old_module, m);
            if !compatibility.is_fully_compatible() {
                incompatible_module_ids.push((m.self_id(), compatibility));
            }
        }
    }

    if !incompatible_module_ids.is_empty() {
        eprintln!(
            "Modules {} is incompatible with remote chain: {}!",
            incompatible_module_ids
                .into_iter()
                .map(|(module_id, compat)| format!(
                    "{}(struct_layout:{},struct_and_function_linking:{})",
                    module_id, compat.struct_layout, compat.struct_and_function_linking
                ))
                .join(","),
            &rpc
        );
    } else {
        eprintln!(
            "All modules in {} is full compatible with remote chain: {}!",
            pkg.compiled_package_info.package_name, &rpc
        );
    }
    Ok(())
}
