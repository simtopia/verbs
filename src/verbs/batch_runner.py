import itertools
import typing

import joblib


class BatchRunner:
    def __init__(
        self,
        init_func: typing.Callable,
        exec_func: typing.Callable,
    ):
        self.init_func = init_func
        self.exec_func = exec_func

    def run(
        self,
        n_steps: int,
        n_samples: int,
        param_samples: typing.List[typing.Dict],
        init_args: typing.Optional[typing.Tuple] = None,
        init_kwargs: typing.Optional[typing.Dict] = None,
        n_jobs=-2,
        verbose=10,
    ) -> typing.List[typing.Dict]:
        init_args = () if init_args is None else init_args
        init_kwargs = dict() if init_kwargs is None else init_kwargs

        snapshot, env_data = self.init_func(*init_args, **init_kwargs)

        n_param_samples = len(param_samples)
        arg_set = itertools.product(param_samples, range(n_samples))

        results = joblib.Parallel(n_jobs=n_jobs, verbose=verbose)(
            joblib.delayed(self.exec_func)(snapshot, n_steps, seed, env_data, **params)
            for params, seed in arg_set
        )

        grouped_results = list()

        for i in range(n_param_samples):
            idx = i * n_samples

            grouped_results.append(
                dict(params=param_samples[i], samples=results[idx : idx + n_samples])
            )

        return grouped_results
