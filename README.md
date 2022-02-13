gosh-remote can turn any multiprocess parallelism script into remote
distribution across multiple nodes in HPC environment.


# how to use

1.  install scheduler and workers automatically using mpirun in batch system
    
    run-gosh-remote.sh
    
        # for OpenMPI, Intel MPI, MPICH or MVAPICH
        mpirun gosh-remote -v mpi-bootstrap
    
    By default, the scheduler will be installed using process of MPI global rank
    0, and the workers will be installed using other MPI ranks.
    
    The above works as a normal batch script, that can be submitted to batch
    system using command such as bsub:
    
        bsub -J test -R "span[ptile=24]" -n 72 ./run-gosh-remote.sh
    
    The above script request 3 nodes for remote executions.

2.  change job script
    
    the master script will call test.sh in parallel using 3 processes:
    
        xargs -P 3 -n 1 gosh-remote client run <<<$'./test.sh ./test.sh ./test.sh'
    
    job script of test.sh:
    
        #! /usr/bin/env bash
        echo running on $(hostname)
    
    example output
    
        "Ok(\"{\\\"JobCompleted\\\":\\\"running on node037\\\\n\\\"}\")"
        "Ok(\"{\\\"JobCompleted\\\":\\\"running on node038\\\\n\\\"}\")"
        "Ok(\"{\\\"JobCompleted\\\":\\\"running on node042\\\\n\\\"}\")"


# Real example in action (for magman)


## run.sh

the main script for install scheduler and workers, and running magman

    #! /usr/bin/env bash
    
    set -x
    #export SPDKIT_RANDOM_SEED=2227866437669085292
    
    LOCK_FILE="gosh-remote-scheduler.lock"
    # run MAX_PROCS processes at a time
    MAX_NPROC=8
    
    # start remote execution services
    (
    # place one process (slot) on each node
    #
    # for Intel MPI or MPICH
    mpirun hostname | sort | uniq |xargs -I{} echo {}:1 >machines
    # one additional slot for scheduler on the master node
    echo `hostname`:1 >>machine
    mpirun -machinefile machines gosh-remote -vv mpi-bootstrap -w "$LOCK_FILE" 2>&1 | tee gosh-remote.log
    ) &
    sleep 2
    
    magman -j $MAX_NPROC -r -vv 2>&1 | tee magman.log
    
    sleep 1
    pkill gosh-remote
    # or better
    # mpirun pkill gosh-remote


## run-vasp.sh

the script to call VASP

    #! /usr/bin/env bash
    
    # get root directory path of this script file
    SCRIPT_DIR=$(dirname $(realpath "${BASH_SOURCE[0]:-$0}"))
    LOCK_FILE="$SCRIPT_DIR/gosh-remote-scheduler.lock"
    
    # NOTE: the "-host" option is required for avoiding process migration due to
    # nested mpirun call
    gosh-remote -vv client -w "$LOCK_FILE" run "mpirun -host \$(hostname) vasp"


# FAQ


## how to avoid conflict due to nested mpirun

in the client side script, make sure vasp run in the worker node, without
migration to other nodes:

    mpirun -host `hostname` vasp

