//! Double exponential quadrature: tanh-sinh on finite intervals, exp-sinh and sinh-sinh on infinite
//! ones; where its levels stall, on a kink or an oscillation, adaptive Gauss–Legendre panels over
//! the same transformed variable take over.

use super::{Estimate, Fault, MINIMUM, Qd, weighted};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::OnceLock;

/// Halvings of the step in the transformed variable before giving up.
const LEVELS: usize = 10;
/// The transformed variable spans [-U_MAX, U_MAX]: further out, nodes would leave the range of f64.
const U_MAX: i64 = 6;
/// From this level on, a level that gains less than two digits hands over to the panels.
const PATIENCE: usize = 6;
/// Rounding left in a sum of `Qd`, relative to the magnitudes summed.
const NOISE: f64 = 1e-60;
/// Panels narrower than this are not split: their nodes would no longer differ.
const NARROWEST: f64 = 1e-45;
/// Panels aim this far below the tolerance: past a kink, a panel and its halves can agree better
/// than the halves agree with the integral, and a hundredfold covers what is seen.
const MARGIN: f64 = 1e-2;
/// Panels past which each doubling of their number must divide the error by 4 to go on toward the
/// tolerance: slower, as for an oscillation that decays like a power, 6 digits are kept as they are.
const STEADY: usize = 1 << 12;
/// Short of 6 digits, the doublings of the panels they may be away at the rate the error falls:
/// further, and they are out of reach.
const REACH: f64 = 5.0;

/// One node of a rule: where it sits (in the rule's own terms) and its weight.
type Node = (Qd, Qd);

/// Nodes per level: level 0 has the whole numbers, level k the odd multiples of 2^-k.
struct Rule {
    levels: Vec<Vec<(f64, Node)>>,
}

impl Rule {
    fn new(node: impl Fn(Qd) -> Node, symmetric: bool) -> Rule {
        let lowest = if symmetric { 0 } else { -U_MAX << LEVELS };
        let levels = (0..=LEVELS)
            .map(|level| {
                let step = 1i64 << (LEVELS - level);
                (lowest..=U_MAX << LEVELS)
                    .filter(|i| i % step == 0 && (level == 0 || (i / step) % 2 != 0))
                    .map(|i| {
                        let u = i as f64 / (1i64 << LEVELS) as f64;
                        (u, node(Qd::from_f64(u)))
                    })
                    .collect()
            })
            .collect();
        Rule { levels }
    }
}

/// tanh-sinh on [-1, 1], for u ≥ 0: distance of the node to the end, and weight.
fn finite() -> &'static Rule {
    static RULE: OnceLock<Rule> = OnceLock::new();
    RULE.get_or_init(|| Rule::new(Map::distance, true))
}

/// exp-sinh on [0, ∞): the node itself, and weight.
fn half_infinite() -> &'static Rule {
    static RULE: OnceLock<Rule> = OnceLock::new();
    RULE.get_or_init(|| {
        Rule::new(
            |u| {
                Map::Half(Qd::ZERO, Qd::ONE)
                    .at(u)
                    .unwrap_or((Qd::ZERO, Qd::ZERO))
            },
            false,
        )
    })
}

/// sinh-sinh on (-∞, ∞), for u ≥ 0: the node, and weight.
fn infinite() -> &'static Rule {
    static RULE: OnceLock<Rule> = OnceLock::new();
    RULE.get_or_init(|| Rule::new(|u| Map::Whole.at(u).unwrap_or((Qd::ZERO, Qd::ZERO)), true))
}

/// How the transformed variable u runs over the interval.
#[derive(Clone, Copy)]
enum Map {
    /// [a, b], by tanh-sinh.
    Finite(Qd, Qd),
    /// From an end toward +∞ (sign 1) or −∞ (sign −1), by exp-sinh.
    Half(Qd, Qd),
    /// (−∞, ∞), by sinh-sinh.
    Whole,
}

impl Map {
    /// For [a, b] and u ≥ 0: distance from the end in half-lengths, and dt/du in half-lengths.
    fn distance(u: Qd) -> Node {
        let (sinh, cosh) = sinh_cosh(u);
        // e^{2v}, and from it both the distance and cosh²v.
        let e = (Qd::PI * sinh).exp();
        let four = Qd::from_i64(4);
        (
            Qd::from_i64(2) / (e + Qd::ONE),
            Qd::FRAC_PI_2 * cosh * four / (e + Qd::from_i64(2) + e.recip()),
        )
    }

    /// The point at u and dt/du there; `None` where it rounds onto an end and adds nothing.
    fn at(self, u: Qd) -> Option<Node> {
        match self {
            Map::Finite(a, b) => {
                let half = (b - a) / Qd::from_i64(2);
                let (distance, weight) = Map::distance(u.abs());
                let t = if u.is_sign_negative() {
                    a + half * distance
                } else {
                    b - half * distance
                };
                (t != a && t != b).then_some((t, half * weight))
            }
            Map::Half(end, sign) => {
                let (sinh, cosh) = sinh_cosh(u);
                let offset = (Qd::FRAC_PI_2 * sinh).exp();
                let t = end + sign * offset;
                (t != end).then_some((t, Qd::FRAC_PI_2 * cosh * offset))
            }
            Map::Whole => {
                let (sinh, cosh) = sinh_cosh(u);
                let (v_sinh, v_cosh) = sinh_cosh(Qd::FRAC_PI_2 * sinh);
                Some((v_sinh, Qd::FRAC_PI_2 * cosh * v_cosh))
            }
        }
    }
}

/// sinh and cosh from one exponential: a node only needs sinh to an absolute 10^-63.
fn sinh_cosh(u: Qd) -> (Qd, Qd) {
    let e = u.exp();
    let inverse = e.recip();
    let two = Qd::from_i64(2);
    ((e - inverse) / two, (e + inverse) / two)
}

/// ∫ f from `a` to `b`, either of which may be infinite, aiming at a relative `tolerance`.
pub(super) fn integrate(
    f: &mut dyn FnMut(Qd) -> Result<Estimate, Fault>,
    a: Qd,
    b: Qd,
    tolerance: f64,
) -> Result<Estimate, Fault> {
    if a.is_nan() || b.is_nan() {
        return Err(Fault::Undefined);
    }
    if a == b {
        return Ok(Estimate::exact(Qd::ZERO));
    }
    if a > b {
        return Ok(integrate(f, b, a, tolerance)?.scaled(-Qd::ONE));
    }
    // Each node gives its weighted value and the error its own value carries.
    let mut point = |t: Qd, weight: Qd| -> Result<(Qd, f64), Fault> {
        let estimate = f(t)?;
        Ok((
            estimate.value * weight,
            weighted(weight.abs().to_f64(), estimate.error),
        ))
    };
    let map = match (a.is_finite(), b.is_finite()) {
        (true, true) => Map::Finite(a, b),
        (true, false) => Map::Half(a, Qd::ONE),
        (false, true) => Map::Half(b, -Qd::ONE),
        (false, false) => Map::Whole,
    };
    let pass = match map {
        Map::Finite(a, b) => {
            let (half, middle) = ((b - a) / Qd::from_i64(2), (a + b) / Qd::from_i64(2));
            refine(finite(), tolerance, &mut |u, (distance, weight)| {
                if u == 0.0 {
                    return point(middle, weight * half);
                }
                let offset = half * distance;
                let (mut value, mut error) = (Qd::ZERO, 0.0);
                for t in [a + offset, b - offset] {
                    // A node so close to an end that it rounds onto it adds nothing.
                    if t != a && t != b {
                        let (part, part_error) = point(t, weight * half)?;
                        value += part;
                        error += part_error;
                    }
                }
                Ok((value, error))
            })
        }
        Map::Half(end, sign) => refine(half_infinite(), tolerance, &mut |_, (offset, weight)| {
            let t = end + sign * offset;
            if t == end {
                return Ok((Qd::ZERO, 0.0));
            }
            point(t, weight)
        }),
        Map::Whole => refine(infinite(), tolerance, &mut |u, (t, weight)| {
            let (value, error) = point(t, weight)?;
            if u == 0.0 {
                return Ok((value, error));
            }
            let (mirror, mirror_error) = point(-t, weight)?;
            Ok((value + mirror, error + mirror_error))
        }),
    }?;
    let best = match pass {
        Pass::Done(estimate) => return Ok(estimate),
        Pass::Stalled(estimate, beyond) => {
            let panels = panels(map, &mut point, tolerance, beyond)?;
            if panels.error < estimate.error {
                panels
            } else {
                estimate
            }
        }
    };
    // Short of the tolerance, what has been reached still counts if it holds 6 digits.
    if best.relative() <= MINIMUM {
        Ok(best)
    } else {
        Err(Fault::Inaccurate)
    }
}

/// Where the levels of a rule got to.
enum Pass {
    Done(Estimate),
    /// Short of the tolerance, with the part of the error that lies beyond ±U_MAX.
    Stalled(Estimate, f64),
}

/// Halves the step until two levels agree to the tolerance; stops once 6 digits are out of reach.
fn refine(
    rule: &Rule,
    tolerance: f64,
    term: &mut dyn FnMut(f64, Node) -> Result<(Qd, f64), Fault>,
) -> Result<Pass, Fault> {
    let (mut total, mut previous) = (Qd::ZERO, None::<Qd>);
    let (mut change, mut errors) = (Qd::INFINITY, 0.0f64);
    // Weighted values at ±U_MAX, and at the node just inside each at the current level.
    let (mut ends, mut inside) = ([Qd::ZERO; 2], [Qd::ZERO; 2]);
    let last = U_MAX as f64;
    let mut step = 1.0;
    for (level, nodes) in rule.levels.iter().enumerate() {
        step = 1.0 / (1u64 << level) as f64;
        let mut sum = Qd::ZERO;
        for &(u, node) in nodes {
            let (value, error) = term(u, node)?;
            if !value.is_finite() {
                return Err(Fault::Undefined);
            }
            sum += value;
            errors += error;
            let side = usize::from(u < 0.0);
            if u.abs() == last {
                ends[side] = value.abs();
            }
            if u.abs() == last - step || (level == 0 && u.abs() == last - 1.0) {
                inside[side] = value.abs();
            }
        }
        total = match level {
            0 => sum,
            _ => total / Qd::from_i64(2) + sum * Qd::from_f64(step),
        };
        // Relative to the value itself: cancellation leaves fewer right digits, and that must show.
        let scale = total.abs();
        let difference = previous.map_or(Qd::INFINITY, |previous| (total - previous).abs());
        // Converging double exponentially, the change before was about the square root of this one.
        let steady = change <= scale * Qd::from_f64(tolerance.sqrt());
        let stalled = level >= PATIENCE && difference * Qd::from_i64(100) > change;
        change = difference;
        // Cut off at the far ends and carried by the integrand: no finer level changes either.
        let fixed = cut_off(ends, inside, step) + errors * step;
        if level >= 3 && fixed > scale.to_f64() * MINIMUM {
            return Err(Fault::Inaccurate);
        }
        if level >= 3 && difference <= scale * Qd::from_f64(tolerance) && steady {
            return Ok(Pass::Done(Estimate {
                value: total,
                error: difference.to_f64() + fixed,
            }));
        }
        previous = Some(total);
        // A kink or an oscillation the step does not resolve: panels place their nodes better.
        if stalled {
            break;
        }
    }
    let beyond = cut_off(ends, inside, step);
    let estimate = Estimate {
        value: total,
        error: change.to_f64() + beyond + errors * step,
    };
    Ok(Pass::Stalled(estimate, beyond))
}

/// The part of the integral beyond ±U_MAX, from how fast the weighted values shrink toward the ends.
fn cut_off(ends: [Qd; 2], inside: [Qd; 2], step: f64) -> f64 {
    ends.into_iter()
        .zip(inside)
        .map(|(end, inside)| match (end, inside) {
            (end, _) if end.is_zero() => 0.0,
            // Not shrinking toward the end: what lies beyond is unknown.
            (end, inside) if inside <= end => f64::INFINITY,
            // Doubled, so that the estimate errs on the side of fewer digits.
            (end, inside) => {
                let ratio = (end / inside).to_f64();
                2.0 * step * end.to_f64() * ratio / (1.0 - ratio)
            }
        })
        .sum()
}

/// Σ of weighted values over one panel, the error they carry, and Σ of their magnitudes.
#[derive(Clone, Copy, Default)]
struct Sum {
    value: Qd,
    error: f64,
    mass: f64,
}

impl Sum {
    fn plus(self, other: Sum) -> Sum {
        Sum {
            value: self.value + other.value,
            error: self.error + other.error,
            mass: self.mass + other.mass,
        }
    }
}

/// A panel of u, summed whole and in two halves: their disagreement is the error of the whole.
struct Panel {
    lo: Qd,
    hi: Qd,
    halves: [Sum; 2],
    gap: f64,
}

impl Panel {
    fn sum(&self) -> Sum {
        self.halves[0].plus(self.halves[1])
    }
}

impl PartialEq for Panel {
    fn eq(&self, other: &Panel) -> bool {
        self.gap == other.gap
    }
}

impl Eq for Panel {}

impl PartialOrd for Panel {
    fn partial_cmp(&self, other: &Panel) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Panel {
    fn cmp(&self, other: &Panel) -> Ordering {
        self.gap.total_cmp(&other.gap)
    }
}

/// u from -U_MAX to U_MAX in Gauss–Legendre panels, always splitting the one that disagrees most
/// with its halves, until the disagreements add up to the tolerance or are down to rounding.
fn panels(
    map: Map,
    point: &mut dyn FnMut(Qd, Qd) -> Result<(Qd, f64), Fault>,
    tolerance: f64,
    beyond: f64,
) -> Result<Estimate, Fault> {
    let rule = gauss(if tolerance < 1e-24 { 40 } else { 16 });
    let two = Qd::from_i64(2);
    let mut sum = |lo: Qd, hi: Qd| -> Result<Sum, Fault> {
        let (middle, half) = ((lo + hi) / two, (hi - lo) / two);
        let mut total = Sum::default();
        for &(x, w) in rule {
            if let Some((t, speed)) = map.at(middle + half * x) {
                let (value, error) = point(t, w * half * speed)?;
                if !value.is_finite() {
                    return Err(Fault::Undefined);
                }
                total.value += value;
                total.error += error;
                total.mass += value.abs().to_f64();
            }
        }
        Ok(total)
    };
    // A panel and its halves; the panel's own sum is known already once it is a half itself.
    let mut split = |lo: Qd, hi: Qd, whole: Option<Sum>| -> Result<Panel, Fault> {
        let whole = match whole {
            Some(whole) => whole,
            None => sum(lo, hi)?,
        };
        let middle = (lo + hi) / two;
        let halves = [sum(lo, middle)?, sum(middle, hi)?];
        let gap = (whole.value - halves[0].value - halves[1].value)
            .abs()
            .to_f64();
        Ok(Panel {
            lo,
            hi,
            halves,
            gap,
        })
    };
    // Totals over all panels, open or settled; in `Qd`, so that taking a panel out leaves no residue.
    let (mut value, mut errors, mut gaps) = (Qd::ZERO, Qd::ZERO, Qd::ZERO);
    let mut open = BinaryHeap::new();
    let width = Qd::from_f64(0.5);
    let mut panel = Qd::from_i64(-U_MAX);
    while panel < Qd::from_i64(U_MAX) {
        open.push(split(panel, panel + width, None)?);
        panel += width;
    }
    for panel in &open {
        value += panel.sum().value;
        errors += Qd::from_f64(panel.sum().error);
        gaps += Qd::from_f64(panel.gap);
    }
    // Panels so far, and the error when they were last half as many.
    let (mut count, mut checked, mut before) = (open.len(), open.len(), f64::INFINITY);
    loop {
        let fixed = errors.to_f64() + beyond;
        let error = gaps.to_f64() + fixed;
        let target = MARGIN * tolerance * value.abs().to_f64();
        // Done, or the values' own errors and what lies past ±U_MAX outweigh whatever splits could gain.
        if error <= target || (fixed > target && gaps.to_f64() <= fixed) {
            return Ok(Estimate { value, error });
        }
        if count >= 2 * checked {
            let (gain, six) = (before / error, MINIMUM * value.abs().to_f64());
            // Slow: 6 digits are kept as they are, and short of them they must be a few doublings away.
            if count >= STEADY
                && gain < 4.0
                && (error <= six || (error / six).ln() > REACH * gain.ln())
            {
                return Ok(Estimate { value, error });
            }
            (checked, before) = (count, error);
            // Panels whose gaps add up to a hundredth of the target will never be split: out of the
            // queue, so that memory stays bounded however long this runs; their gaps stay counted.
            open = prune(open, target / 100.0);
        }
        let Some(worst) = open.pop() else {
            return Ok(Estimate { value, error });
        };
        // Down to the rounding of its own values, or too narrow to split: settled as it is.
        let whole = worst.sum();
        if worst.gap <= NOISE * whole.mass + whole.error
            || (worst.hi - worst.lo).to_f64() < NARROWEST
        {
            continue;
        }
        value -= whole.value;
        errors -= Qd::from_f64(whole.error);
        gaps -= Qd::from_f64(worst.gap);
        let middle = (worst.lo + worst.hi) / two;
        let [left, right] = worst.halves;
        let halves = [
            split(worst.lo, middle, Some(left))?,
            split(middle, worst.hi, Some(right))?,
        ];
        for panel in halves {
            value += panel.sum().value;
            errors += Qd::from_f64(panel.sum().error);
            gaps += Qd::from_f64(panel.gap);
            open.push(panel);
        }
        count += 1;
    }
}

/// The panels worth splitting later: the ones with the smallest gaps go, up to `budget` in all.
fn prune(open: BinaryHeap<Panel>, budget: f64) -> BinaryHeap<Panel> {
    let mut panels = open.into_sorted_vec();
    let mut spent = 0.0;
    let kept = panels
        .iter()
        .position(|panel| {
            spent += panel.gap;
            spent > budget
        })
        .unwrap_or(panels.len());
    panels.drain(..kept);
    BinaryHeap::from(panels)
}

/// Gauss–Legendre nodes and weights on [-1, 1]: 16 for the curve's digits, 40 for 50 digits.
fn gauss(points: usize) -> &'static [(Qd, Qd)] {
    static SIXTEEN: OnceLock<Vec<(Qd, Qd)>> = OnceLock::new();
    static FORTY: OnceLock<Vec<(Qd, Qd)>> = OnceLock::new();
    let rule = if points == 16 { &SIXTEEN } else { &FORTY };
    rule.get_or_init(|| legendre(points))
}

/// The roots of the Legendre polynomial P_m by Newton's method, and their weights 2/((1-x²)P_m'(x)²).
fn legendre(m: usize) -> Vec<(Qd, Qd)> {
    // P_m(x) and P_m'(x) by the three-term recurrence.
    let at = |x: Qd| {
        let (mut p, mut q) = (x, Qd::ONE);
        for k in 1..m as i64 {
            let next =
                (Qd::from_i64(2 * k + 1) * x * p - Qd::from_i64(k) * q) / Qd::from_i64(k + 1);
            (p, q) = (next, p);
        }
        (p, Qd::from_i64(m as i64) * (x * p - q) / (x * x - Qd::ONE))
    };
    let mut rule = Vec::with_capacity(m);
    for i in 1..=m / 2 {
        let guess = std::f64::consts::PI * (i as f64 - 0.25) / (m as f64 + 0.5);
        let mut x = Qd::from_f64(guess.cos());
        for _ in 0..12 {
            let (p, derivative) = at(x);
            let step = p / derivative;
            x -= step;
            if step.abs().to_f64() < 1e-66 {
                break;
            }
        }
        let (_, derivative) = at(x);
        let weight = Qd::from_i64(2) / ((Qd::ONE - x * x) * derivative * derivative);
        rule.push((x, weight));
        rule.push((-x, weight));
    }
    rule
}

#[cfg(test)]
mod tests {
    use super::{Estimate, Fault, Qd, integrate};
    use crate::app::graph::numeric::{MINIMUM, TOLERANCE};

    fn close(value: Qd, expected: Qd) -> bool {
        (value - expected).abs() <= expected.abs() * Qd::from_f64(1e-50)
    }

    type Case = (fn(Qd) -> Qd, Qd, Qd, Qd);

    #[test]
    fn integrates_to_fifty_digits() {
        let two = Qd::from_i64(2);
        // Smooth, singular at an end, and infinite in one or both directions.
        let cases: [Case; 6] = [
            (|t| t * t, Qd::ZERO, Qd::ONE, Qd::ONE / Qd::from_i64(3)),
            (|t| t.sqrt().recip(), Qd::ZERO, Qd::ONE, two),
            (|t| t.ln(), Qd::ZERO, Qd::ONE, -Qd::ONE),
            (|t| (-t).exp(), Qd::ZERO, Qd::INFINITY, Qd::ONE),
            (
                |t| (t * t + Qd::ONE).recip(),
                Qd::ZERO,
                Qd::INFINITY,
                Qd::FRAC_PI_2,
            ),
            (
                |t| (-(t * t)).exp(),
                Qd::NEG_INFINITY,
                Qd::INFINITY,
                Qd::PI.sqrt(),
            ),
        ];
        for (f, a, b, expected) in cases {
            let estimate = integrate(&mut |t| Ok(Estimate::exact(f(t))), a, b, TOLERANCE).unwrap();
            assert!(close(estimate.value, expected), "{estimate:?} {expected:?}");
            assert!(estimate.relative() <= TOLERANCE, "{estimate:?}");
        }
        let reversed = integrate(
            &mut |t| Ok(Estimate::exact(t)),
            Qd::ONE,
            Qd::ZERO,
            TOLERANCE,
        );
        assert!(close(reversed.unwrap().value, -Qd::ONE / two));
    }

    #[test]
    fn gauss_legendre_is_exact_up_to_its_degree() {
        for points in [16, 40] {
            let rule = super::gauss(points);
            let moment = |k: i64| {
                rule.iter()
                    .fold(Qd::ZERO, |sum, &(x, w)| sum + w * x.powi(k))
            };
            // ∫ x^k over [-1, 1] is 2/(k+1) for even k, up to the degree 2m-1 of the rule.
            let k = 2 * points as i64 - 2;
            let two = Qd::from_i64(2);
            assert!(close(moment(0), two) && close(moment(k), two / Qd::from_i64(k + 1)));
        }
    }

    #[test]
    fn keeps_six_digits_and_stops_below() {
        // 1/t^0.95 puts its mass closer to 0 than f64 nodes reach: short of 50 digits, not of 6.
        let power = |p: f64| move |t: Qd| Ok(Estimate::exact(t.pow(Qd::from_f64(p))));
        let partial = integrate(&mut power(-0.95), Qd::ZERO, Qd::ONE, TOLERANCE).unwrap();
        assert!(partial.relative() > TOLERANCE && partial.relative() <= MINIMUM);
        let actual = (partial.value - Qd::from_i64(20)).abs().to_f64();
        assert!(
            actual <= partial.error,
            "actual {actual:e} estimated {:e}",
            partial.error
        );
        // 1/t^0.999: not even 6 digits.
        let hopeless = integrate(&mut power(-0.999), Qd::ZERO, Qd::ONE, TOLERANCE);
        assert_eq!(hopeless, Err(Fault::Inaccurate));
    }
}
