// [[file:../remote.note::*imports][imports:1]]
use super::*;
// imports:1 ends here

// [[file:../remote.note::ae1f2b04][ae1f2b04]]
/// Return MPI local rank ID
fn get_local_rank_id() -> Option<usize> {
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
fn get_global_rank_id() -> Option<usize> {
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
fn get_local_number_of_ranks() -> Option<usize> {
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
fn get_global_number_of_ranks() -> Option<usize> {
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

// [[file:../remote.note::c9ad7115][c9ad7115]]
pub struct Mpi {
    pub global_rank: usize,
    pub local_rank: usize,
    pub global_size: usize,
    pub local_size: usize,
}

impl Mpi {
    pub fn try_from_env() -> Option<Self> {
        let global_rank = get_global_rank_id()?;
        let local_rank = get_local_rank_id()?;
        let global_size = get_global_number_of_ranks()?;
        let local_size = get_local_number_of_ranks()?;
        remove_mpi_env_vars();

        Self {
            global_rank,
            local_rank,
            global_size,
            local_size,
        }
        .into()
    }
}
// c9ad7115 ends here
