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
pub fn is_mpi_rank_for_scheduler() -> bool {
    mpi::get_global_rank_id() == Some(0)
}

pub fn is_mpi_rank_for_worker() -> bool {
    match (mpi::get_global_rank_id(), mpi::get_local_rank_id()) {
        (Some(m), Some(1)) => {
            let name = hostname();
            info!("start worker on global rank {m}, on local rank 1 of node {name}");
            true
        }
        _ => false,
    }
}

pub fn install_scheduler_and_workers() -> Result<()> {
    let node = hostname();
    debug!("Install scheduler/workers on node {node} ...");
    match (get_global_number_of_ranks(), get_local_number_of_ranks()) {
        (Some(n), Some(m)) => {
            debug!("Found {n} global ranks, {m} local ranks on node {node}");
        }
        _ => {
            bail!("no relevant MPI env vars found!")
        }
    }

    Ok(())
}
// 7e536226 ends here
