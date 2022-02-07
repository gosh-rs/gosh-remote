// [[file:../remote.note::*imports][imports:1]]
use super::*;
// imports:1 ends here

// [[file:../remote.note::ae1f2b04][ae1f2b04]]
/// Return MPI local rank ID
pub fn get_local_rank_id() -> Option<usize> {
    std::env::vars().find_map(|(k, v)| match k.as_str() {
        "MPI_LOCALRANKID" => {
            debug!("found rank ID for MPICH/Intel MPI.");
            v.parse().ok()
        }
        "OMPI_COMM_WORLD_LOCAL_RANK" => {
            debug!("found rank ID for OpenMPI.");
            v.parse().ok()
        }
        _ => None,
    })
}
// ae1f2b04 ends here
