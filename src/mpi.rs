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
