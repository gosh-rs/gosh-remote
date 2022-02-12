// [[file:../remote.note::*imports][imports:1]]
use super::*;
// imports:1 ends here

// [[file:../remote.note::ae1f2b04][ae1f2b04]]
/// Return MPI local rank ID
pub fn get_local_rank_id() -> Option<usize> {
    std::env::vars().find_map(|(k, v)| match k.as_str() {
        // Intel MPI, MPICH
        "MPI_LOCALRANKID" => v.parse().ok(),
        // OpenMPI
        "OMPI_COMM_WORLD_LOCAL_RANK" => v.parse().ok(),
        // MVAPICH
        "MV2_COMM_WORLD_LOCAL_RANK" => v.parse().ok(),
        _ => None,
    })
}

fn remove_mpi_env_vars() {
    debug!("remove all MPI relevant env vars ...");
    for (k, _) in std::env::vars() {
        if k.contains("MPI") {
            std::env::remove_var(k);
        }
    }
}

/// Return MPI global rank ID
pub fn get_global_rank_id() -> Option<usize> {
    std::env::vars().find_map(|(k, v)| match k.as_str() {
        // Intel MPI, MPICH
        "PMI_RANK" => v.parse().ok(),
        // OpenMPI
        "OMPI_COMM_WORLD_RANK" => v.parse().ok(),
        // MVAPICH
        "MV2_COMM_WORLD_RANK" => v.parse().ok(),
        _ => None,
    })
}

/// Return MPI local number of ranks
pub fn get_local_number_of_ranks() -> Option<usize> {
    std::env::vars().find_map(|(k, v)| match k.as_str() {
        // Intel MPI, MPICH
        "MPI_LOCALNRANKS" => v.parse().ok(),
        // OpenMPI
        "OMPI_COMM_WORLD_LOCAL_SIZE" => v.parse().ok(),
        // MVAPICH
        "MV2_COMM_WORLD_LOCAL_SIZE" => v.parse().ok(),
        _ => None,
    })
}

/// Return MPI global number of ranks
pub fn get_global_number_of_ranks() -> Option<usize> {
    std::env::vars().find_map(|(k, v)| match k.as_str() {
        // Intel MPI, MPICH
        "PMI_SIZE" => v.parse().ok(),
        // OpenMPI
        "OMPI_COMM_WORLD_SIZE" => v.parse().ok(),
        // MVAPICH
        "MV2_COMM_WORLD_SIZE" => v.parse().ok(),
        _ => None,
    })
}
// ae1f2b04 ends here

// [[file:../remote.note::7e536226][7e536226]]
pub fn install_scheduler_or_worker(share_node: bool) -> Result<bool> {
    let node = hostname();
    debug!("Install scheduler/workers on node {node} ...");
    match (
        get_global_number_of_ranks(),
        get_local_number_of_ranks(),
        get_global_rank_id(),
        get_local_rank_id(),
    ) {
        (Some(n), Some(m), Some(i), Some(j)) => {
            remove_mpi_env_vars();
            debug!("Found {n} global ranks, {m} local ranks on node {node}");
            let x = n/m;
            debug!("Will install {x} workers on {x} nodes");
            if i == 0 {
                info!("install scheduler on rank {i}/{j} of node {node}");
                return Ok(true);
            }
            // also install worker on the same node as the scheduler
            if i == j && j == 1 {
                if share_node {
                    info!("install worker on rank {i}/{j} of node {node}");
                    return Ok(false);
                }
            }
            if j == 0 {
                info!("install worker on rank {i}/{j} of node {node}");
                return Ok(false);
            }
            bail!("ignore rank {i}/{j}");
        }
        _ => {
            bail!("no relevant MPI env vars found!")
        }
    }
}
// 7e536226 ends here
