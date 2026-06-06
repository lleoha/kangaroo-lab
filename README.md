# Kangaroo Lab
This repository is an experimental study of the discrete logarithm problem in a
known interval. Given a cyclic group written additively, a generator $G$, and a
target $Q = xG$, where $x$ is known to lie in the interval $l \le x < h$, the
objective is to recover $x$. If $N=h-l$ denotes the interval length, the
classical algorithms considered here all require $O(\sqrt{N})$ group operations.
The practical question is therefore the leading constant $k$ in $ops(N) \approx k\sqrt{N}$.

The code in this repository estimates this constant experimentally for several
families of interval-DLP algorithms:
- Pollard kangaroo methods with two, three, and four kangaroos [[GPR10]];
- Gaudry-Schost collision search variants, including variants using
  equivalence classes under the negation map [[GPR10]] [[GR10]] [[ZZYLL19]] [[RC24]];
- baby-step giant-step and variants using the negation map [[GWZ15]].

The emphasis is deliberately narrow: this repository counts abstract group
operations. It does not attempt to benchmark a production implementation,
and it does not include all costs or overheads that appear in real distributed implementations.
This makes the experiments useful for comparing leading constants, but the
numbers should be read as idealized operation-count measurements.

## Implementation and Experimental Model
The implementation is intentionally idealized. Its purpose is to estimate the
leading group-operation constant. The reported statistics are the mean, standard deviation,
95% confidence interval for the mean, and median of $k$.

The current experiments use a simple additive group over a 63-bit
prime-order scalar field. This choice is deliberate. The algorithms studied
here only require group addition, negation, and scalar multiplication
by known offsets. The leading constant should not depend on whether the group is
this additive group, an elliptic-curve group, or another cyclic group of
comparable order. A real group would change engineering costs, memory layout,
and related overheads, but not the idealized number of group operations.

If the measured value of $k$ depended strongly on the chosen group, that would
suggest a problem with the sampling or walk model rather than an algorithmic constant.

The implementations also use a simplified collision model. Kangaroo
variants store all visited points instead of storing only distinguished points.
Gaudry-Schost variants sample directly from their tame and wild regions rather
than simulating bounded pseudorandom walks that restart
after distinguished points. These choices make the experiments closer to the
ideal birthday-paradox model and more optimistic than a complete walk-based
distributed implementation.

## Implemented Variants

| Family        | Solver                         | Main idea                                                             |                             Theory average $k$ |
|---------------|--------------------------------|-----------------------------------------------------------------------|-----------------------------------------------:|
| BSGS          | `BabyStepGiantStepBasic`       | Textbook table with $m \approx \sqrt(N)$                              |                                1.5 (2.0 worst) |
| BSGS          | `BabyStepGiantStepNegMap`      | Uses equivalent classes $R \sim -R$                                   |                                1.0 (1.5 worst) |
| BSGS          | `BabyStepGiantStepInterleaved` | Interleaved baby and giant, equivalent classes, $m \approx \sqrt(2N)$ |                            0.943 (1.414 worst) |
| Kangaroo      | `PollardKangarooBasic`         | One tame and one wild walk                                            |                                            2.0 |
| Kangaroo      | `PollardKangarooThree`         | One tame, two wild walks                                              |                                          1.819 |
| Kangaroo      | `PollardKangarooFour`          | Two tame with parity trick, two wild walks                            |                                          1.715 |
| Gaudry-Schost | `GaudrySchostBasicSim`         | Tame-wild birthday search                                             |                                           2.08 |
| Gaudry-Schost | `GaudrySchostThreeSet`         | Improved three-set geometry with $\gamma = 0.588$                     |                                          1.761 |
| Gaudry-Schost | `GaudrySchostFourSet`          | Improved Four-set with parity trick                                   |                                          1.661 |
| Gaudry-Schost | `GaudrySchostNegMap`           | Equivalence classes and narrower wild set                             |                                           1.36 |
| Gaudry-Schost | `GaudrySchostImprovedNegMap`   | Parameterized narrower wild set                                       | 1.275 ($\alpha = 0.1$), 1.253 ($\alpha \to 0$) |
| Gaudry-Schost | `GaudrySchostSotaV2`           | Smaller tame set plus wild-wild collisions, same wild set parity      |                                              ? |
| Gaudry-Schost | `GaudrySchostSotaV2Plus`       | "Cheap point" version                                                 |                                              ? |

## Experimental Results
The results below were computed with interval size $N=2^{48}$ and $32768$ iterations for each variant.

### Baby-Step Giant-Step

| Method            | Code name                      | Theory mean $k$ | Mean $k$ | Std dev | 95% CI | Median $k$ |
|-------------------|--------------------------------|----------------:|---------:|--------:|-------:|-----------:|
| Basic BSGS        | `BabyStepGiantStepBasic`       |             1.5 |    1.500 |   0.288 | [1.497, 1.503] |      1.502 |
| Negation-map BSGS | `BabyStepGiantStepNegMap`      |             1.0 |    0.999 |   0.289 | [0.996, 1.002] |      0.998 |
| Interleaved BSGS  | `BabyStepGiantStepInterleaved` |           0.943 |    0.944 |   0.333 | [0.941, 0.948] |      1.002 |

### Pollard Kangaroo

| Method          | Code name              | Theory mean $k$ | Mean $k$ | Std dev | 95% CI | Median $k$ |
|-----------------|------------------------|----------------:|---------:|--------:|-------:|-----------:|
| Basic kangaroo  | `PollardKangarooBasic` |           2.000 |    1.984 |   1.144 | [1.971, 1.996] |      1.830 |
| Three kangaroos | `PollardKangarooThree` |           1.819 |    1.813 |   1.011 | [1.802, 1.823] |      1.670 |
| Four kangaroos  | `PollardKangarooFour`  |           1.715 |    1.718 |   0.968 | [1.708, 1.729] |      1.586 |

### Gaudry-Schost

| Method                   | Code name                    | Parameters              | Theory mean k | Mean k | Std dev | 95% CI | Median k |
|--------------------------|------------------------------|-------------------------|--------------:|-------:|--------:|-------:|---------:|
| Basic two-set            | `GaudrySchostBasicSim`       |                         |         2.080 |  2.072 |   1.107 | [2.060, 2.084] |    1.930 |
| Improved three-set       | `GaudrySchostThreeSet`       |                         |         1.761 |  1.770 |   0.971 | [1.759, 1.781] |    1.633 |
| Improved four-set        | `GaudrySchostFourSet`        |                         |         1.661 |  1.659 |   0.915 | [1.649, 1.669] |    1.532 |
| Negation map             | `GaudrySchostNegMap`         |                         |         1.360 |  1.367 |   0.732 | [1.359, 1.375] |    1.274 |
| Narrow-wild negation map | `GaudrySchostImprovedNegMap` | $\alpha = 0.1$          |         1.275 |  1.274 |   0.677 | [1.267, 1.282] |    1.194 |
| Narrow-wild negation map | `GaudrySchostImprovedNegMap` | $\alpha = 0.05$         |         1.264 |  1.259 |   0.662 | [1.252, 1.267] |    1.184 |
| SOTA v2                  | `GaudrySchostSotaV2`         | $\alpha = \frac{1}{8}$  |             ? |  1.129 |   0.620 | [1.123, 1.136] |    1.047 |
| SOTA v2                  | `GaudrySchostSotaV2`         | $\alpha = \frac{1}{64}$ |             ? |  1.108 |   0.597 | [1.101, 1.114] |    1.033 |
| SOTA v2 plus             | `GaudrySchostSotaV2Plus`     | $\alpha = \frac{1}{64}$ |             ? |  1.057 |   0.564 | [1.050, 1.063] |    0.989 |

## Conclusion

The repository is organized around the leading constant in interval-DLP
algorithms. BSGS gives the clearest deterministic baseline but requires
$O(\sqrt{N})$ storage. The negation map and interleaving substantially improve
the average constant when inversion is cheap.

Kangaroo methods provide low-storage random-walk algorithms. The transition
from two to three and four kangaroos is driven by adding useful collision types,
especially collisions involving $Q$ and $-Q$.

Gaudry-Schost methods provide the most flexible collision-search framework.
Their ideal constants can be better than kangaroo constants, especially with
carefully shaped sets and equivalence classes. However, practical performance
depends more heavily on pseudorandom-walk quality, distinguished-point tuning,
walk restarts, region boundaries, and fruitless-cycle handling.

## References

- [[GPR10]] Steven D. Galbraith, John M. Pollard, and Raminder S. Ruprai,
  "Computing Discrete Logarithms in an Interval".
- [[GR10]] Steven D. Galbraith and Raminder S. Ruprai,
  "Using Equivalence Classes to Accelerate Solving the Discrete Logarithm
  Problem in a Short Interval".
- [[GWZ15]] Steven D. Galbraith, Ping Wang, and Fangguo Zhang,
  "Computing Elliptic Curve Discrete Logarithms with Improved Baby-step
  Giant-step Algorithm".
- [[ZZYLL19]] Yuqing Zhu, Jincheng Zhuang, Hairong Yi, Chang Lv, and Dongdai Lin,
  "A variant of the Galbraith-Ruprai algorithm for discrete logarithms with
  improved complexity".
- [[RC24]] RetiredC, "Kang-1".

[GPR10]: <https://eprint.iacr.org/2010/617.pdf>
[GR10]: <https://eprint.iacr.org/2010/615.pdf>
[GWZ15]: <https://eprint.iacr.org/2015/605.pdf>
[ZZYLL19]: <https://doi.org/10.1007/s10623-018-0492-3>
[RC24]: <https://github.com/RetiredC/Kang-1>
