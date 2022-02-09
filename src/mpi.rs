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
fn is_mpi_rank_for_scheduler() -> bool {
    mpi::get_global_rank_id() == Some(0)
}

pub fn install_scheduler_or_worker() -> Result<bool> {
    let node = hostname();
    debug!("Install scheduler/workers on node {node} ...");
    match (
        get_global_number_of_ranks(),
        get_local_number_of_ranks(),
        get_global_rank_id(),
        get_local_rank_id(),
    ) {
        (Some(n), Some(m), Some(i), Some(j)) => {
            debug!("Found {n} global ranks, {m} local ranks on node {node}");
            if i == 0 {
                info!("install scheduler on rank {i}/{j} of node {node}");
                return Ok(true);
            } else {
                info!("install worker on rank {i}/{j} of node {node}");
                return Ok(false);
            }
        }
        _ => {
            bail!("no relevant MPI env vars found!")
        }
    }
}
// 7e536226 ends here
