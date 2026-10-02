//! What CPUID says of the time-stamp counter: whether it is invariant, and its rate.

use raw_cpuid::{CpuId, CpuIdReader};

use crate::counter::CounterError;

/// Hertz in a kilohertz, the unit a hypervisor reports the counter's rate in.
const HERTZ_PER_KILOHERTZ: u64 = 1_000;

/// The counter's ticks a second, refused where the counter is not invariant.
///
/// The rate is the CPU's, from leaf `0x15`; else the hypervisor's, from leaf `0x4000_0010`; else
/// the one `measure` takes.
pub(super) fn ticks_per_second<R, M>(cpuid: &CpuId<R>, measure: M) -> Result<u64, CounterError>
where
    R: CpuIdReader,
    M: FnOnce() -> Option<u64>,
{
    let is_invariant =
        cpuid.get_advanced_power_mgmt_info().is_some_and(|power| power.has_invariant_tsc());
    if !is_invariant {
        return Err(CounterError::NotInvariant);
    }
    let hypervisor_rate = || {
        let kilohertz = cpuid.get_hypervisor_info()?.tsc_frequency()?;
        u64::from(kilohertz).checked_mul(HERTZ_PER_KILOHERTZ)
    };
    cpuid
        .get_tsc_info()
        .and_then(|leaf| leaf.tsc_frequency())
        .or_else(hypervisor_rate)
        .filter(|&ticks_per_second| ticks_per_second > 0)
        .or_else(measure)
        .ok_or(CounterError::NoRate)
}

#[cfg(test)]
mod tests {
    use raw_cpuid::{CpuId, CpuIdReader, CpuIdResult};

    use super::ticks_per_second;
    use crate::CounterError;

    /// The highest basic and extended leaves, and the power leaf saying the counter is invariant.
    const INVARIANT: [(u32, [u32; 4]); 3] = [
        (0, [0x15, 0, 0, 0]),
        (0x8000_0000, [0x8000_0007, 0, 0, 0]),
        (0x8000_0007, [0, 0, 0, 1 << 8]),
    ];

    /// A rate leaf reporting a 38.4 MHz crystal, and a ratio of 176 to 2 to it.
    const RATE_LEAF: (u32, [u32; 4]) = (0x15, [2, 176, 38_400_000, 0]);

    /// A CPU answering each of `leaves` with its EAX, EBX, ECX and EDX, and every other with zeros.
    fn cpu<const COUNT: usize>(leaves: [(u32, [u32; 4]); COUNT]) -> CpuId<impl CpuIdReader> {
        CpuId::with_cpuid_reader(move |leaf, _subleaf| {
            let answer = leaves.iter().find(|&&(number, _)| number == leaf);
            let [eax, ebx, ecx, edx] = answer.map_or([0; 4], |&(_, registers)| registers);
            CpuIdResult { eax, ebx, ecx, edx }
        })
    }

    /// No measurement, as without `std`.
    const fn no_measurement() -> Option<u64> {
        None
    }

    #[test]
    fn an_invariant_counter_runs_at_the_rate_its_leaf_reports() {
        let [highest, extended, power] = INVARIANT;
        let cpuid = cpu([highest, extended, power, RATE_LEAF]);
        assert_eq!(ticks_per_second(&cpuid, no_measurement), Ok(3_379_200_000));
    }

    #[test]
    fn a_counter_that_is_not_invariant_is_refused() {
        let [highest, extended, _] = INVARIANT;
        let variable_rate = cpu([highest, extended, RATE_LEAF]);
        assert_eq!(
            ticks_per_second(&variable_rate, no_measurement),
            Err(CounterError::NotInvariant)
        );

        let [highest, _, power] = INVARIANT;
        let old_cpu = cpu([highest, (0x8000_0000, [0x8000_0001, 0, 0, 0]), power, RATE_LEAF]);
        assert_eq!(ticks_per_second(&old_cpu, no_measurement), Err(CounterError::NotInvariant));
    }

    #[test]
    fn a_virtual_counter_runs_at_the_rate_its_hypervisor_reports() {
        let [_, extended, power] = INVARIANT;
        let cpuid = cpu([
            (0, [0x0D, 0, 0, 0]),
            (1, [0, 0, 1 << 31, 0]),
            extended,
            power,
            (0x4000_0000, [0x4000_0010, 0, 0, 0]),
            (0x4000_0010, [2_100_000, 0, 0, 0]),
        ]);
        assert_eq!(ticks_per_second(&cpuid, no_measurement), Ok(2_100_000_000));
    }

    #[test]
    fn a_counter_whose_cpu_reports_no_rate_is_measured_or_refused() {
        let silent_leaf = cpu(INVARIANT);
        assert_eq!(ticks_per_second(&silent_leaf, || Some(2_100_000_000)), Ok(2_100_000_000));
        assert_eq!(ticks_per_second(&silent_leaf, no_measurement), Err(CounterError::NoRate));

        let [highest, extended, power] = INVARIANT;
        let crystal_unknown = cpu([highest, extended, power, (0x15, [2, 168, 0, 0])]);
        assert_eq!(ticks_per_second(&crystal_unknown, no_measurement), Err(CounterError::NoRate));
    }
}
