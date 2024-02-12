"""
Batch simulation runner

Wraps functionality to initialise a snapshot
of the simulation environment and then
run the simulation across a sequence of
parameter samples and random seeds.

This batch executor uses joblib to execute
simulation samples in parallel.
"""

import itertools
import typing

import joblib

import verbs


def batch_run(
    sim_func: typing.Callable,
    *,
    n_steps: int,
    n_samples: int,
    parameters_samples: typing.List[typing.Dict],
    snapshot=None,
    cache=None,
    n_jobs=-2,
    verbose=10,
    **sim_kwargs,
):
    """
    Run a batch of simulations across a set or parameters

    Convenience function to run simulations across a range
    of parameters and random seeds. Uses joblib to run
    simulation samples in parallel.

    Will generate ``n_sample`` Monte-Carlo samples for
    each of the provided parameter samples.

    The simulation environment for each sample can optionally
    be initialised from either a provided snapshot
    (generated using the
    :py:meth:`verbs.envs.ForkEnv.export_snapshot` method),
    or a cache (generated using the
    :py:meth:`verbs.envs.ForkEnv.export_cache` method).

    Parameters
    ----------
    sim_func: typing.Callable
        Simulation execution function, should
        have the signature

        .. code-block:: python

           def sim_func(
               env, seed, n_steps, **params, **sim_kwargs
           ) -> typing.Any:
               ...

        where arguments are:

        * ``env`` a simulation environment
        * ``seed`` a random seed
        * ``n_steps`` number of simulation steps
        * ``**params`` keyword simulation parameters
        * ``**sim_kwars`` keyword simulation fixed
          arguments/parameters

    n_steps: int
        Number of simulation steps.
    n_samples: int
        Number of monte-carlo samples to create for
        each parameter sample.
    parameters_samples: typing.List[typing.Dict]
        List of dictionaries containing simulation
        parameters. These parameters will be passed as
        keyword arguments to the ``sim_func`` function.
    snapshot: typing.Tuple , optional
        Optional snapshot used to initialise the
        simulation environment for each execution.
    cache: verbs.types.Cache, optional
        Optional cache used to initialise the simulation
        environment for each execution.
    n_jobs: int, optional
        Number of jobs to run simultaneously, default is
        ``-2`` i.e. 1 less than the number of available
        processors.
    verbose: int, optional
        Verbosity of joblib logging, default is ``10`` (
        full logging).
    **sim_kwargs
        Any additional keyword arguments passed directly
        to the ``sim_func`` function (i.e. they are shared
        across all the executions).

    Returns
    -------
    typing.List[typing.Dict]
        List of results, grouped by their parameters. Each
        entry is a dictionary containing ``"params"`` the
        parameters to produce the samples, and ``"samples"``
        the list of data produced by each Monte-Carlo sample.
    """
    assert not (
        snapshot is not None and cache is not None
    ), "Either a snapshot or cache should be provided, not both"

    n_param_samples = len(parameters_samples)
    arg_set = itertools.product(parameters_samples, range(n_samples))

    def inner_runner(params, seed):
        env = verbs.envs.EmptyEnv(seed, cache=cache, snapshot=snapshot)
        return sim_func(env, seed, n_steps, **params, **sim_kwargs)

    results = joblib.Parallel(n_jobs=n_jobs, verbose=verbose)(
        joblib.delayed(inner_runner)(p, s) for p, s in arg_set
    )

    grouped_results = list()

    for i in range(n_param_samples):
        idx = i * n_samples

        grouped_results.append(
            dict(params=parameters_samples[i], samples=results[idx : idx + n_samples])
        )

    return grouped_results
